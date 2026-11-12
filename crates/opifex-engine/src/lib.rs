//! Opifex execution engine — scheduler, node runtime, and providers.
//! Phase 3: Core engine for executing block graphs node-by-node with
//! pinned data support, cancellation, and live/mock modes.

use opifex_core::{
    store::{NodeRun, NodeRunStatus, Run, RunMode, RunStatus},
    AgentGraph, NodeInstance,
};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod providers;
pub mod runtime;
pub use runtime::{
    BoxFuture, ChatMessage, EngineRuntime, LlmRequest, LlmResponse, ToolRequest,
    UnavailableRuntime,
};

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
    /// Node outputs keyed by "node_id:port_id", the wires between blocks.
    outputs: Arc<RwLock<HashMap<String, Value>>>,
    /// Host capabilities: providers and tools.
    runtime: Arc<dyn EngineRuntime>,
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
            outputs: Arc::new(RwLock::new(HashMap::new())),
            runtime: Arc::new(UnavailableRuntime),
        }
    }

    /// Attach the host that can reach providers and tools.
    pub fn with_runtime(graph: AgentGraph, runtime: Arc<dyn EngineRuntime>) -> Self {
        let mut engine = Self::new(graph);
        engine.runtime = runtime;
        engine
    }

    /// Every node output produced so far, keyed by "node_id:port_id".
    pub async fn node_outputs(&self) -> HashMap<String, Value> {
        self.outputs.read().await.clone()
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
        let _run_id = self.start_run(mode.clone()).await?;
        let graph = self.graph.read().await.clone();
        let execution_order = self.compute_execution_order(&graph)?;

        for node_id in &execution_order {
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| format!("Node not found: {}", node_id))?;

            let result = self.execute_node(node, mode.clone()).await;

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
            // Keep this node's outputs so downstream blocks can read them by port.
            if let Some(Value::Object(ports)) = &result.output {
                let mut outputs = self.outputs.write().await;
                for (port, value) in ports {
                    outputs.insert(format!("{}:{}", node_id, port), value.clone());
                }
            }

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
    async fn execute_node(&self, node: &NodeInstance, mode: RunMode) -> NodeExecutionResult {
        let inputs = self.resolve_inputs(node).await;

        if mode == RunMode::Mock {
            return self.execute_mock(node);
        }

        match node.block_id.as_str() {
            "builtin:input" => self.execute_input(node, inputs).await,
            "builtin:llm-prompt" => self.execute_llm_prompt(node, inputs).await,
            "builtin:structured-llm" => self.execute_structured_llm(node, inputs).await,
            "builtin:agent" => self.execute_agent(node, inputs).await,
            "builtin:http-request" => self.execute_http_request(node, inputs).await,
            "builtin:condition" => self.execute_condition(node, inputs).await,
            "builtin:loop-for-each" => self.execute_loop(node),
            "builtin:merge" => self.execute_merge(node, inputs),
            "builtin:code-js" => self.execute_code_js(node, inputs).await,
            "builtin:final-answer" => self.execute_final_answer(node, inputs),
            block_id => failed(node, format!("Unknown block type: {}", block_id)),
        }
    }

    /// Values arriving on a node's input ports: upstream outputs, with pinned
    /// values taking precedence so a pinned port survives a re-run.
    async fn resolve_inputs(&self, node: &NodeInstance) -> HashMap<String, Value> {
        let graph = self.graph.read().await;
        let outputs = self.outputs.read().await;
        let pinned = self.pinned_data.read().await;

        let mut inputs = HashMap::new();
        for edge in graph.edges.iter().filter(|edge| edge.target == node.id) {
            let key = format!("{}:{}", edge.source, edge.source_handle);
            if let Some(value) = pinned.get(&key).or_else(|| outputs.get(&key)) {
                inputs.insert(edge.target_handle.clone(), value.clone());
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

    /// Placeholder outputs for a dry run: no provider is called.
    fn execute_mock(&self, node: &NodeInstance) -> NodeExecutionResult {
        let output = match node.block_id.as_str() {
            "builtin:input" => serde_json::json!({ "message": "", "payload": {} }),
            "builtin:llm-prompt" => serde_json::json!({ "completion": "LLM response placeholder" }),
            "builtin:structured-llm" => serde_json::json!({ "result": {} }),
            "builtin:agent" => serde_json::json!({ "answer": "Agent response placeholder", "trace": {} }),
            "builtin:http-request" => serde_json::json!({ "response": {}, "tool": {} }),
            "builtin:condition" => serde_json::json!({ "true": null, "false": null }),
            "builtin:loop-for-each" => serde_json::json!({ "results": [] }),
            "builtin:merge" => serde_json::json!({ "merged": [] }),
            "builtin:code-js" => serde_json::json!({ "output": null }),
            "builtin:final-answer" => serde_json::json!({ "message": null }),
            _ => return failed(node, format!("Unknown block type: {}", node.block_id)),
        };
        succeeded(node, output, None, None)
    }

    /// The entry point: hands the run input to the first block.
    async fn execute_input(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let message = input_str(&inputs, "message")
            .or_else(|| param_str(node, "message"))
            .unwrap_or_default();
        succeeded(
            node,
            serde_json::json!({ "message": message, "payload": {} }),
            None,
            None,
        )
    }

    /// A single completion from the configured provider.
    async fn execute_llm_prompt(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let message = input_str(&inputs, "message").or_else(|| param_str(node, "message"));
        let Some(message) = message.filter(|value| !value.trim().is_empty()) else {
            return failed(
                node,
                "Prompt Weaver has nothing to send: connect a block to its Message port or set a run input.",
            );
        };

        let request = LlmRequest {
            model: param_str(node, "model"),
            system: input_str(&inputs, "system").or_else(|| param_str(node, "system")),
            messages: vec![ChatMessage::user(message)],
            temperature: param_f64(node, "temperature"),
            json: false,
        };

        match self.runtime.run_llm(request).await {
            Ok(response) => succeeded(
                node,
                serde_json::json!({ "completion": response.text }),
                response.tokens_in,
                response.tokens_out,
            ),
            Err(error) => failed(node, error),
        }
    }

    /// A completion whose text is required to be JSON.
    async fn execute_structured_llm(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let message = input_str(&inputs, "message")
            .or_else(|| param_str(node, "message"))
            .or_else(|| param_str(node, "schema"));
        let Some(message) = message.filter(|value| !value.trim().is_empty()) else {
            return failed(node, "Structured LLM block has no prompt to send.");
        };

        let instruction = match param_str(node, "schema") {
            Some(schema) => format!("{}\n\nReply with JSON matching this schema: {}", message, schema),
            None => message,
        };

        let request = LlmRequest {
            model: param_str(node, "model"),
            system: Some(
                "Reply with a single JSON value and nothing else: no prose, no code fences.".to_string(),
            ),
            messages: vec![ChatMessage::user(instruction)],
            temperature: param_f64(node, "temperature"),
            json: true,
        };

        match self.runtime.run_llm(request).await {
            Ok(response) => match parse_json_value(&response.text) {
                Ok(value) => succeeded(
                    node,
                    serde_json::json!({ "result": value }),
                    response.tokens_in,
                    response.tokens_out,
                ),
                Err(error) => failed(
                    node,
                    format!("Model did not return JSON: {} (got: {})", error, truncate(&response.text, 200)),
                ),
            },
            Err(error) => failed(node, error),
        }
    }

    /// One completion with the agent's instructions. Tool use inside the loop is
    /// not wired up yet, so this makes exactly one call and reports it as one step.
    async fn execute_agent(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let prompt = input_str(&inputs, "prompt")
            .or_else(|| input_str(&inputs, "message"))
            .or_else(|| param_str(node, "prompt"));
        let Some(prompt) = prompt.filter(|value| !value.trim().is_empty()) else {
            return failed(node, "Task Orchestrator has no prompt to work from.");
        };

        let system = param_str(node, "system").unwrap_or_else(|| {
            "You are an autonomous agent. Answer the task directly and concisely.".to_string()
        });

        let request = LlmRequest {
            model: param_str(node, "model"),
            system: Some(system),
            messages: vec![ChatMessage::user(prompt)],
            temperature: param_f64(node, "temperature"),
            json: false,
        };

        match self.runtime.run_llm(request).await {
            Ok(response) => succeeded(
                node,
                serde_json::json!({
                    "answer": response.text,
                    "trace": {
                        "iterations": 1,
                        "maxIterations": param_f64(node, "max_iterations")
                            .map(|value| value as u64)
                            .unwrap_or(8),
                        "tools": inputs.get("tools").cloned().unwrap_or(Value::Null),
                    },
                }),
                response.tokens_in,
                response.tokens_out,
            ),
            Err(error) => failed(node, error),
        }
    }

    /// A real HTTP request made by the host, so credentials and the network stay
    /// out of the engine.
    async fn execute_http_request(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let url = input_str(&inputs, "url").or_else(|| param_str(node, "url"));
        let Some(url) = url.filter(|value| !value.trim().is_empty()) else {
            return failed(
                node,
                "Web Courier needs a URL: connect one to its URL port or set the url parameter.",
            );
        };
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return failed(node, format!("Web Courier only speaks http(s): {}", url));
        }

        let method = param_str(node, "method").unwrap_or_else(|| "GET".to_string());
        let request = ToolRequest {
            kind: "http-request".to_string(),
            url: Some(url.clone()),
            method: Some(method.clone()),
            headers: inputs.get("headers").cloned(),
            body: inputs.get("body").cloned(),
            code: None,
            input: None,
        };

        match self.runtime.run_tool(request).await {
            Ok(response) => succeeded(
                node,
                serde_json::json!({
                    "response": response,
                    "tool": { "kind": "http-request", "method": method, "url": url },
                }),
                None,
                None,
            ),
            Err(error) => failed(node, error),
        }
    }

    /// Routes the incoming value to the true or false port.
    async fn execute_condition(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let value = inputs.get("value").cloned().unwrap_or(Value::Null);
        let output = if is_truthy(&value) {
            serde_json::json!({ "true": value, "false": null })
        } else {
            serde_json::json!({ "true": null, "false": value })
        };
        succeeded(node, output, None, None)
    }

    /// Loops are not executed: a run walks the graph once, and pretending to loop
    /// would report work that never happened.
    fn execute_loop(&self, node: &NodeInstance) -> NodeExecutionResult {
        failed(
            node,
            "Loop blocks are not implemented: a run executes each block once.",
        )
    }

    fn execute_merge(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let merged: Vec<Value> = inputs.into_values().collect();
        succeeded(node, serde_json::json!({ "merged": merged }), None, None)
    }

    /// JavaScript blocks need the sandbox, which ships with the host.
    async fn execute_code_js(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let request = ToolRequest {
            kind: "code-js".to_string(),
            url: None,
            method: None,
            headers: None,
            body: None,
            code: param_str(node, "code"),
            input: inputs.get("input").cloned(),
        };

        match self.runtime.run_tool(request).await {
            Ok(value) => succeeded(node, serde_json::json!({ "output": value }), None, None),
            Err(error) => failed(node, error),
        }
    }

    /// The terminal block: passes the answer through so the run has a result.
    fn execute_final_answer(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, Value>,
    ) -> NodeExecutionResult {
        let message = input_str(&inputs, "message").or_else(|| param_str(node, "message"));
        let Some(message) = message.filter(|value| !value.trim().is_empty()) else {
            return failed(node, "Result Terminal has no message to return.");
        };
        succeeded(node, serde_json::json!({ "message": message }), None, None)
    }
}

fn succeeded(
    node: &NodeInstance,
    output: Value,
    tokens_in: Option<u64>,
    tokens_out: Option<u64>,
) -> NodeExecutionResult {
    NodeExecutionResult {
        node_id: node.id.clone(),
        status: NodeRunStatus::Succeeded,
        output: Some(output),
        error: None,
        tokens_in,
        tokens_out,
    }
}

fn failed(node: &NodeInstance, error: impl Into<String>) -> NodeExecutionResult {
    NodeExecutionResult {
        node_id: node.id.clone(),
        status: NodeRunStatus::Failed,
        output: None,
        error: Some(error.into()),
        tokens_in: None,
        tokens_out: None,
    }
}

fn param_str(node: &NodeInstance, key: &str) -> Option<String> {
    match node.parameters.get(key) {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

fn param_f64(node: &NodeInstance, key: &str) -> Option<f64> {
    match node.parameters.get(key) {
        Some(Value::Number(number)) => number.as_f64(),
        Some(Value::String(text)) => text.trim().parse().ok(),
        _ => None,
    }
}

/// Reads a value arriving on an input port as text. Objects produced by other
/// blocks carry their text in a conventional field, so those are unwrapped.
fn input_str(inputs: &HashMap<String, Value>, key: &str) -> Option<String> {
    match inputs.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Object(map) => ["content", "text", "message", "answer", "completion", "output"]
            .iter()
            .find_map(|field| map.get(*field).and_then(|value| value.as_str()))
            .map(|text| text.to_string()),
        Value::Array(items) => Some(
            items
                .iter()
                .map(|item| match item {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        _ => None,
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|float| float != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
    }
}

/// Accepts the JSON a model actually returns: fences and surrounding prose are
/// stripped before parsing.
fn parse_json_value(text: &str) -> Result<Value, String> {
    let trimmed = text.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let unfenced = unfenced.strip_suffix("```").unwrap_or(unfenced).trim();

    if let Ok(value) = serde_json::from_str::<Value>(unfenced) {
        return Ok(value);
    }

    let first = unfenced.find(['{', '[']);
    let last = unfenced.rfind(['}', ']']);
    if let (Some(first), Some(last)) = (first, last) {
        if last > first {
            return serde_json::from_str::<Value>(&unfenced[first..=last])
                .map_err(|error| error.to_string());
        }
    }

    Err("no JSON value found".to_string())
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let clipped: String = text.chars().take(limit).collect();
    format!("{}…", clipped)
}


#[cfg(test)]
mod tests {
    use super::*;
    use opifex_core::EdgeInstance;

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

    /// The canvas writes camelCase JSON into agents.spec_json; a run has to
    /// parse that payload and walk the graph without any translation step.
    #[tokio::test]
    async fn test_runs_webview_camel_case_graph() {
        let json = serde_json::json!({
            "id": "canvas-graph",
            "name": "Canvas graph",
            "schemaVersion": 1,
            "nodes": [
                {
                    "id": "n1",
                    "blockId": "builtin:input",
                    "blockVersion": 1,
                    "position": { "x": 0.0, "y": 0.0 },
                    "parameters": {},
                    "isCollapsed": false
                },
                {
                    "id": "n2",
                    "blockId": "builtin:llm-prompt",
                    "blockVersion": 1,
                    "position": { "x": 200.0, "y": 0.0 },
                    "parameters": {},
                    "isCollapsed": false
                }
            ],
            "edges": [{
                "id": "e1",
                "source": "n1",
                "sourceHandle": "message",
                "target": "n2",
                "targetHandle": "message"
            }],
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        });

        let graph: AgentGraph = serde_json::from_value(json).expect("canvas graph must parse");
        let run = ExecutionEngine::new(graph)
            .run_graph(RunMode::Mock)
            .await
            .expect("mock run should walk the graph");

        assert_eq!(run.status, RunStatus::Succeeded);
        assert!(run.ended_at.is_some());
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

    struct StubRuntime {
        reply: String,
        llm_calls: std::sync::atomic::AtomicUsize,
        tool_calls: std::sync::atomic::AtomicUsize,
    }

    impl StubRuntime {
        fn new(reply: &str) -> Self {
            Self {
                reply: reply.to_string(),
                llm_calls: std::sync::atomic::AtomicUsize::new(0),
                tool_calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    impl EngineRuntime for StubRuntime {
        fn run_llm<'a>(&'a self, _request: LlmRequest) -> BoxFuture<'a, Result<LlmResponse, String>> {
            self.llm_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let reply = self.reply.clone();
            Box::pin(async move {
                Ok(LlmResponse { text: reply, tokens_in: Some(7), tokens_out: Some(11) })
            })
        }

        fn run_tool<'a>(&'a self, _request: ToolRequest) -> BoxFuture<'a, Result<Value, String>> {
            self.tool_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async move { Ok(serde_json::json!({ "status": 200, "body": "ok" })) })
        }
    }

    /// trigger(message) -> llm-prompt(message -> completion) -> final-answer(message)
    fn make_chain_graph(trigger_message: &str) -> AgentGraph {
        let mut graph = make_test_graph();
        graph.nodes[0].parameters.insert(
            "message".to_string(),
            Value::String(trigger_message.to_string()),
        );
        graph.nodes[1].parameters.insert(
            "model".to_string(),
            Value::String("test-model".to_string()),
        );
        // builtin:final-answer expects its input on "message".
        graph.edges[1].target_handle = "message".to_string();
        graph
    }

    #[tokio::test]
    async fn live_run_calls_the_provider_and_passes_data_downstream() {
        let runtime = Arc::new(StubRuntime::new("hello from the model"));
        let engine = ExecutionEngine::with_runtime(make_chain_graph("say hello"), runtime.clone());

        let run = engine.run_graph(RunMode::Live).await.expect("live run");
        assert_eq!(run.status, RunStatus::Succeeded);
        assert_eq!(runtime.llm_calls.load(std::sync::atomic::Ordering::SeqCst), 1);

        let outputs = engine.node_outputs().await;
        assert_eq!(
            outputs.get("n2:completion").and_then(|value| value.as_str()),
            Some("hello from the model")
        );
        assert_eq!(
            outputs.get("n3:message").and_then(|value| value.as_str()),
            Some("hello from the model")
        );
    }

    #[tokio::test]
    async fn llm_block_fails_without_a_message() {
        let runtime = Arc::new(StubRuntime::new("unused"));
        let engine = ExecutionEngine::with_runtime(make_chain_graph(""), runtime.clone());

        let error = engine.run_graph(RunMode::Live).await.expect_err("must fail");
        assert!(error.contains("nothing to send"), "unexpected error: {}", error);
        assert_eq!(runtime.llm_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }


    fn make_structured_graph() -> AgentGraph {
        let mut graph = make_test_graph();
        graph.nodes[0].parameters.insert(
            "message".to_string(),
            Value::String("summarise this".to_string()),
        );
        graph.nodes[1].block_id = "builtin:structured-llm".to_string();
        // The structured block is the last block here: it publishes "result",
        // not the "completion" the final-answer block reads.
        graph.nodes.pop();
        graph.edges.pop();
        graph
    }

    #[tokio::test]
    async fn structured_block_parses_fenced_json() {
        let reply = "```json\n{\"score\": 3}\n```";
        let runtime = Arc::new(StubRuntime::new(reply));
        let engine = ExecutionEngine::with_runtime(make_structured_graph(), runtime);

        engine.run_graph(RunMode::Live).await.expect("structured run");
        let outputs = engine.node_outputs().await;
        assert_eq!(
            outputs
                .get("n2:result")
                .and_then(|value| value.get("score"))
                .and_then(|score| score.as_i64()),
            Some(3)
        );
    }

    #[tokio::test]
    async fn structured_block_fails_when_the_model_ignores_json() {
        let runtime = Arc::new(StubRuntime::new("I am afraid I cannot do that."));
        let engine = ExecutionEngine::with_runtime(make_structured_graph(), runtime);

        let error = engine.run_graph(RunMode::Live).await.expect_err("must fail");
        assert!(error.contains("did not return JSON"), "unexpected error: {}", error);
    }

    #[tokio::test]
    async fn mock_mode_never_reaches_the_provider() {
        let runtime = Arc::new(StubRuntime::new("unused"));
        let engine = ExecutionEngine::with_runtime(make_chain_graph("say hello"), runtime.clone());

        engine.run_graph(RunMode::Mock).await.expect("mock run");
        assert_eq!(runtime.llm_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(runtime.tool_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn http_block_goes_through_the_runtime() {
        let runtime = Arc::new(StubRuntime::new("unused"));
        let graph = AgentGraph {
            id: "http-graph".to_string(),
            name: "HTTP".to_string(),
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
                    block_id: "builtin:http-request".to_string(),
                    block_version: 1,
                    position: opifex_core::Position { x: 200.0, y: 0.0 },
                    parameters: HashMap::new(),
                    sub_graph: None,
                    is_collapsed: false,
                },
            ],
            edges: vec![EdgeInstance {
                id: "e1".to_string(),
                source: "n1".to_string(),
                source_handle: "message".to_string(),
                target: "n2".to_string(),
                target_handle: "url".to_string(),
            }],
            viewport: None,
            metadata: None,
        };

        let mut graph = graph;
        graph.nodes[0].parameters.insert(
            "message".to_string(),
            Value::String("ftp://example.com/data".to_string()),
        );

        let engine = ExecutionEngine::with_runtime(graph, runtime.clone());
        let error = engine.run_graph(RunMode::Live).await.expect_err("must fail");
        assert!(error.contains("http(s)"), "unexpected error: {}", error);
        assert_eq!(runtime.tool_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
