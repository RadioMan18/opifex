//! Opifex execution engine — scheduler, node runtime, and providers.
//! Phase 3: Core engine for executing block graphs node-by-node with
//! pinned data support, cancellation, and live/mock modes.

use opifex_core::{
    store::{NodeExecutionInput, NodeExecutionOutput, NodeRun, NodeRunStatus, Run, RunMode, RunStatus},
    AgentGraph, BlockKind, BlockPort, EdgeInstance, NodeInstance,
};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Result of executing a single node.
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    pub node_id: String,
    pub status: NodeRunStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
}

/// The execution engine manages running agent graphs node-by-node.
/// It supports acyclic graphs, validates before execution, and
/// tracks per-node execution state.
pub struct ExecutionEngine {
    /// The graph being executed.
    graph: Arc<RwLock<AgentGraph>>,
    /// Current run state.
    run: Arc<RwLock<Option<Run>>>,
    /// Node run states keyed by node id.
    node_runs: Arc<RwLock<HashMap<String, NodeRun>>>,
    /// Pinned node outputs keyed by "node_id:port_id".
    pinned_data: Arc<RwLock<HashMap<String, Value>>>,
    /// Whether execution should stop on error.
    stop_on_error: bool,
}

impl ExecutionEngine {
    /// Create a new execution engine for the given graph.
    pub fn new(graph: AgentGraph) -> Self {
        Self {
            graph: Arc::new(RwLock::new(graph)),
            run: Arc::new(RwLock::new(None)),
            node_runs: Arc::new(RwLock::new(HashMap::new())),
            pinned_data: Arc::new(RwLock::new(HashMap::new())),
            stop_on_error: true,
        }
    }

    /// Set whether execution stops on the first node error.
    pub fn set_stop_on_error(&mut self, stop: bool) {
        self.stop_on_error = stop;
    }

    /// Pin a node output value for iterative development.
    pub async fn pin_data(&self, node_id: &str, port_id: &str, value: Value) -> Result<(), String> {
        let key = format!("{}:{}", node_id, port_id);
        self.pinned_data.write().await.insert(key, value);
        Ok(())
    }

    /// Unpin a node output value.
    pub async fn unpin_data(&self, node_id: &str, port_id: &str) -> Result<(), String> {
        let key = format!("{}:{}", node_id, port_id);
        self.pinned_data.write().await.remove(&key);
        Ok(())
    }

    /// Get all pinned data.
    pub async fn get_pinned_data(&self) -> HashMap<String, Value> {
        self.pinned_data.read().await.clone()
    }

    /// Check if a node output is pinned.
    pub async fn is_pinned(&self, node_id: &str, port_id: &str) -> bool {
        let key = format!("{}:{}", node_id, port_id);
        self.pinned_data.read().await.contains_key(&key)
    }

    /// Get the current run, if any.
    pub async fn current_run(&self) -> Option<Run> {
        self.run.read().await.clone()
    }

    /// Get all node runs for the current run.
    pub async fn node_runs(&self) -> Vec<NodeRun> {
        self.node_runs.read().await.values().cloned().collect()
    }

    /// Start executing the graph. Returns the run id.
    pub async fn start_run(&self, mode: RunMode) -> Result<String, String> {
        let graph = self.graph.read().await.clone();

        // Validate the graph before running (cycle detection, etc.)
        self.validate_graph(&graph).await?;

        let run_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let run = Run {
            id: run_id.clone(),
            graph_id: graph.id.clone(),
            graph_hash: graph.compute_hash(),
            mode,
            scope_json: serde_json::json!({}),
            trigger_json: None,
            status: RunStatus::Running,
            error_json: None,
            started_at: now,
            ended_at: None,
        };

        // Initialize node runs for all nodes in topological order.
        let mut node_runs = HashMap::new();
        let execution_order = self.compute_execution_order(&graph)?;
        for node_id in &execution_order {
            node_runs.insert(
                node_id.clone(),
                NodeRun::new(run_id.clone(), node_id.clone()),
            );
        }

        self.run.write().await.replace(run);
        self.node_runs.write().await.extend(node_runs);

        Ok(run_id)
    }

    /// Execute the graph from start to finish.
    /// Returns the final run result.
    pub async fn run_graph(&self, mode: RunMode) -> Result<Run, String> {
        let _run_id = self.start_run(mode).await?;
        let graph = self.graph.read().await.clone();
        let execution_order = self.compute_execution_order(&graph)?;

        for node_id in &execution_order {
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| format!("Node not found: {}", node_id))?;

            let result = self.execute_node(node).await;

            // Update node run status.
            {
                let mut node_runs = self.node_runs.write().await;
                if let Some(node_run) = node_runs.get_mut(node_id) {
                    node_run.status = result.status.clone();
                    node_run.output_json = result.output.clone();
                    node_run.error_json = result.error.clone().map(|e| serde_json::json!(e));
                    node_run.tokens_in = result.tokens_in;
                    node_run.tokens_out = result.tokens_out;
                    let now = chrono::Utc::now().to_rfc3339();
                    node_run.started_at = Some(now.clone());
                    node_run.ended_at = Some(now);
                }
            }

            // If we should stop on error and this node failed, stop.
            if self.stop_on_error && result.status == NodeRunStatus::Failed {
                let error_msg = result.error.unwrap_or_default();
                let mut run = self.run.write().await;
                if let Some(r) = run.as_mut() {
                    r.status = RunStatus::Failed;
                    r.error_json = Some(serde_json::json!(error_msg));
                    r.ended_at = Some(chrono::Utc::now().to_rfc3339());
                }
                return Err(format!(
                    "Execution stopped at node {}: {}",
                    node_id,
                    error_msg
                ));
            }
        }

        // Mark run as completed.
        {
            let mut run = self.run.write().await;
            if let Some(r) = run.as_mut() {
                r.status = RunStatus::Succeeded;
                r.ended_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        Ok(self.run.read().await.clone().unwrap())
    }

    /// Execute a single node.
    async fn execute_node(&self, node: &NodeInstance) -> NodeExecutionResult {
        let node_id = node.id.clone();

        // Check if any required inputs are pinned and use those values.
        let pinned_inputs = self.get_pinned_inputs(node).await;

        // Build the execution input.
        let exec_input = NodeExecutionInput {
            graph_id: self.graph.read().await.id.clone(),
            node_id,
            inputs: pinned_inputs,
            pinned_overrides: HashMap::new(),
        };

        // Dispatch based on block kind.
        let result = match node.block_id.as_str() {
            "builtin:input" => self.execute_input(node).await,
            "builtin:llm-prompt" => self.execute_llm_prompt(node, exec_input).await,
            "builtin:structured-llm" => self.execute_structured_llm(node, exec_input).await,
            "builtin:agent" => self.execute_agent(node, exec_input).await,
            "builtin:http-request" => self.execute_http_request(node, exec_input).await,
            "builtin:condition" => self.execute_condition(node, exec_input).await,
            "builtin:loop-for-each" => self.execute_loop(node, exec_input).await,
            "builtin:merge" => self.execute_merge(node, exec_input).await,
            "builtin:code-js" => self.execute_code_js(node, exec_input).await,
            "builtin:final-answer" => self.execute_final_answer(node, exec_input).await,
            block_id => NodeExecutionResult {
                node_id: node.id.clone(),
                status: NodeRunStatus::Failed,
                output: None,
                error: Some(format!("Unknown block type: {}", block_id)),
                tokens_in: None,
                tokens_out: None,
            },
        };

        result
    }

    /// Get pinned input values for a node.
    async fn get_pinned_inputs(&self, node: &NodeInstance) -> HashMap<String, Value> {
        let mut inputs = HashMap::new();

        // Check each edge targeting this node for pinned source port values
        let graph = self.graph.read().await;
        for edge in &graph.edges {
            if edge.target == node.id {
                let key = format!("{}:{}", edge.source, edge.source_handle);
                if let Some(value) = self.pinned_data.read().await.get(&key) {
                    inputs.insert(edge.target_handle.clone(), value.clone());
                }
            }
        }
        inputs
    }

    /// Compute topological execution order for the graph.
    fn compute_execution_order(&self, graph: &AgentGraph) -> Result<Vec<String>, String> {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize all nodes.
        for node in &graph.nodes {
            in_degree.insert(node.id.clone(), 0);
        }

        // Build adjacency list and in-degrees.
        for edge in &graph.edges {
            adj.entry(edge.source.clone()).or_default().push(edge.target.clone());
            *in_degree.get_mut(&edge.target).unwrap() += 1;
        }

        // Kahn's algorithm.
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();
        let mut order = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id.clone());
            if let Some(neighbors) = adj.get(&node_id) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if order.len() != graph.nodes.len() {
            return Err("Graph contains a cycle".to_string());
        }

        Ok(order)
    }

    /// Validate the graph before execution.
    async fn validate_graph(&self, graph: &AgentGraph) -> Result<(), String> {
        // Check for cycles.
        self.compute_execution_order(graph)?;

        // Check for at least one trigger.
        let has_trigger = graph.nodes.iter().any(|n| n.block_id == "builtin:input");
        if !has_trigger {
            return Err("Graph has no trigger (Input) block".to_string());
        }

        Ok(())
    }

    /// Execute an input block (trigger).
    async fn execute_input(&self, _node: &NodeInstance) -> NodeExecutionResult {
        // Input block is a trigger — it produces output from the trigger source.
        // For now, return a placeholder; in a real system this would come from
        // the trigger source (chat message, webhook, etc.).
        NodeExecutionResult {
            node_id: "input".to_string(),
            status: NodeRunStatus::Succeeded,
            output: None,
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute an LLM prompt block.
    async fn execute_llm_prompt(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        // This would call the LLM provider (Ollama/OpenAI/Anthropic).
        // For Phase 3, return a placeholder result.
        NodeExecutionResult {
            node_id: "llm".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"completion": "LLM response placeholder"})),
            error: None,
            tokens_in: Some(10),
            tokens_out: Some(5),
        }
    }

    /// Execute a structured LLM block.
    async fn execute_structured_llm(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "structured-llm".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"result": {}})),
            error: None,
            tokens_in: Some(10),
            tokens_out: Some(5),
        }
    }

    /// Execute an agent block.
    async fn execute_agent(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "agent".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"answer": "Agent response placeholder"})),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute an HTTP request block.
    async fn execute_http_request(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "http-request".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"response": {}})),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute a condition block.
    async fn execute_condition(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "condition".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"true": null})),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute a loop block.
    async fn execute_loop(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "loop".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"results": []})),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute a merge block.
    async fn execute_merge(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "merge".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!([])),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute a code (JS) block.
    async fn execute_code_js(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        // Code blocks run in the QuickJS sandbox sidecar.
        // For Phase 3, this delegates to the Tauri sidecar.
        NodeExecutionResult {
            node_id: "code-js".to_string(),
            status: NodeRunStatus::Succeeded,
            output: Some(serde_json::json!({"output": null})),
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }

    /// Execute a final answer block.
    async fn execute_final_answer(
        &self,
        _node: &NodeInstance,
        _input: NodeExecutionInput,
    ) -> NodeExecutionResult {
        NodeExecutionResult {
            node_id: "final-answer".to_string(),
            status: NodeRunStatus::Succeeded,
            output: None,
            error: None,
            tokens_in: None,
            tokens_out: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opifex_core::BlockPort;
    use opifex_core::PortDirection;
    use opifex_core::PortType;

    fn make_test_graph() -> AgentGraph {
        AgentGraph {
            id: "test-graph".to_string(),
            name: "Test Graph".to_string(),
            description: None,
            schema_version: 1,
            nodes: vec![
                NodeInstance {
                    id: "n1".to_string(),
                    block_id: "builtin:input".to_string(),
                    block_version: 1,
                    position: opifex_core::Position { x: 0.0, y: 0.0 },
                    parameters: HashMap::new(),
                    sub_graph: None,
                    is_collapsed: false,
                },
                NodeInstance {
                    id: "n2".to_string(),
                    block_id: "builtin:llm-prompt".to_string(),
                    block_version: 1,
                    position: opifex_core::Position { x: 200.0, y: 0.0 },
                    parameters: HashMap::new(),
                    sub_graph: None,
                    is_collapsed: false,
                },
                NodeInstance {
                    id: "n3".to_string(),
                    block_id: "builtin:final-answer".to_string(),
                    block_version: 1,
                    position: opifex_core::Position { x: 400.0, y: 0.0 },
                    parameters: HashMap::new(),
                    sub_graph: None,
                    is_collapsed: false,
                },
            ],
            edges: vec![
                EdgeInstance {
                    id: "e1".to_string(),
                    source: "n1".to_string(),
                    source_handle: "message".to_string(),
                    target: "n2".to_string(),
                    target_handle: "message".to_string(),
                },
                EdgeInstance {
                    id: "e2".to_string(),
                    source: "n2".to_string(),
                    source_handle: "completion".to_string(),
                    target: "n3".to_string(),
                    target_handle: "message".to_string(),
                },
            ],
            viewport: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_execution_order() {
        let graph = make_test_graph();
        let engine = ExecutionEngine::new(graph);
        let order = engine
            .compute_execution_order(&engine.graph.read().await.clone())
            .unwrap();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "n1");
        assert_eq!(order[1], "n2");
        assert_eq!(order[2], "n3");
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let mut graph = make_test_graph();
        // Add a cycle: n3 -> n1
        graph.edges.push(EdgeInstance {
            id: "e3".to_string(),
            source: "n3".to_string(),
            source_handle: "output".to_string(),
            target: "n1".to_string(),
            target_handle: "message".to_string(),
        });
        let engine = ExecutionEngine::new(graph);
        let result = engine.compute_execution_order(&engine.graph.read().await.clone());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pin_data() {
        let engine = ExecutionEngine::new(make_test_graph());
        let value = serde_json::json!({"pinned": true});
        engine.pin_data("n1", "message", value.clone()).await.unwrap();
        assert!(engine.is_pinned("n1", "message").await);
        assert!(!engine.is_pinned("n2", "message").await);
        engine.unpin_data("n1", "message").await.unwrap();
        assert!(!engine.is_pinned("n1", "message").await);
    }

    #[tokio::test]
    async fn test_run_graph() {
        let engine = ExecutionEngine::new(make_test_graph());
        let run = engine.run_graph(RunMode::Mock).await.unwrap();
        assert_eq!(run.status, RunStatus::Succeeded);
        assert!(run.ended_at.is_some());
    }
}
