//! Core types and validation for Opifex v2.0.0 block graphs.
//! Single source of truth for graph schema, port compatibility, and validator authority.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Port data types used for connection compatibility checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    String,
    Number,
    Boolean,
    Object,
    Any,
    Message,
    LLM,
    Tool,
    Json,
}

impl PortType {
    pub fn is_compatible_with(&self, other: &PortType) -> bool {
        if self == &PortType::Any || other == &PortType::Any {
            return true;
        }
        self == other
    }
}

/// Block kind classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockKind {
    Trigger,
    LLM,
    Agent,
    Tool,
    Control,
    Output,
    Utility,
    Subgraph,
}

/// A port definition on a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPort {
    pub id: String,
    pub name: String,
    pub r#type: PortType,
    pub direction: PortDirection,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

/// A block definition (reusable template).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub id: String,
    pub version: u32,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub kind: BlockKind,
    pub origin: BlockOrigin,
    pub inputs: Vec<BlockPort>,
    pub outputs: Vec<BlockPort>,
    pub parameters: Vec<BlockParameter>,
    pub spec_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockOrigin {
    User,
    Imported,
}

/// A configurable parameter on a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParameter {
    pub id: String,
    pub name: String,
    pub param_type: ParameterType,
    pub label: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub options: Option<Vec<String>>,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Select,
    MultiSelect,
    Textarea,
    Code,
    Credential,
}

/// A node instance placed on the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInstance {
    pub id: String,
    pub block_id: String,
    pub block_version: u32,
    pub position: Position,
    pub parameters: HashMap<String, serde_json::Value>,
    pub sub_graph: Option<Box<AgentGraph>>,
    pub is_collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// An edge connecting two node ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInstance {
    pub id: String,
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

/// The full agent graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGraph {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub schema_version: u32,
    pub nodes: Vec<NodeInstance>,
    pub edges: Vec<EdgeInstance>,
    pub viewport: Option<Viewport>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl AgentGraph {
    pub fn compute_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        serde_json::to_string(self).unwrap().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Validation error from the graph validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub node_path: Option<String>,
    pub port_id: Option<String>,
    pub edge_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Validation result from the graph validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub graph_hash: String,
}

/// Shared conformance vector for port compatibility testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConformanceVector {
    pub source_type: PortType,
    pub target_type: PortType,
    pub expected_result: bool,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("block not found: {0}")]
    BlockNotFound(String),
    #[error("port not found: {0}")]
    PortNotFound(String),
    #[error("invalid connection: {0}")]
    InvalidConnection(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

/// Check if a source port type can connect to a target port type.
pub fn can_connect(source_type: &PortType, target_type: &PortType) -> bool {
    source_type.is_compatible_with(target_type)
}

/// The GraphValidator is the single source of validation truth.
pub struct GraphValidator<'a> {
    graph: &'a AgentGraph,
    definitions: &'a HashMap<(String, u32), BlockDefinition>,
}

impl<'a> GraphValidator<'a> {
    pub fn new(
        graph: &'a AgentGraph,
        definitions: &'a HashMap<(String, u32), BlockDefinition>,
    ) -> Self {
        Self { graph, definitions }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Structure check: exactly one trigger
        let trigger_count = self
            .graph
            .nodes
            .iter()
            .filter(|n| self.get_kind(n) == Some(BlockKind::Trigger))
            .count();

        if trigger_count == 0 {
            errors.push(ValidationError {
                severity: Severity::Error,
                code: "NO_TRIGGER".to_string(),
                message: "Graph lacks an Input block. Every graph requires exactly one trigger.".to_string(),
                suggestion: Some("Drag an 'Input' block onto the canvas.".to_string()),
                node_path: None,
                port_id: None,
                edge_id: None,
            });
        } else if trigger_count > 1 {
            errors.push(ValidationError {
                severity: Severity::Error,
                code: "MULTIPLE_TRIGGERS".to_string(),
                message: "Multiple trigger blocks detected. Rev 3 graphs support exactly one input.".to_string(),
                suggestion: Some("Remove extra trigger blocks.".to_string()),
                node_path: None,
                port_id: None,
                edge_id: None,
            });
        }

        // 2. Cycle detection
        self.check_cycles(&mut errors);

        // 3. Port type matching & required connections
        self.check_ports_and_edges(&mut errors, &mut warnings);

        let all_errors: Vec<ValidationError> = errors.into_iter().chain(warnings).collect();
        let valid = all_errors.iter().all(|e| e.severity == Severity::Warning);

        ValidationResult {
            valid,
            errors: all_errors,
            graph_hash: self.graph.compute_hash(),
        }
    }

    fn check_cycles(&self, errors: &mut Vec<ValidationError>) {
        // Build adjacency list
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let node_ids: HashSet<String> = self.graph.nodes.iter().map(|n| n.id.clone()).collect();

        for edge in &self.graph.edges {
            adj.entry(edge.source.clone()).or_default().push(edge.target.clone());
        }

        // Tarjan's SCC or simple DFS cycle detection
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();

        fn dfs(
            node: &str,
            adj: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            errors: &mut Vec<ValidationError>,
            path: &mut Vec<String>,
        ) {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if rec_stack.contains(neighbor) {
                        // Found a cycle
                        errors.push(ValidationError {
                            severity: Severity::Error,
                            code: "CYCLE_DETECTED".to_string(),
                            message: format!("Cycle detected in graph involving node '{}'. Agent graphs must be acyclic.", neighbor),
                            suggestion: Some("Remove the edge that creates the cycle.".to_string()),
                            node_path: Some(neighbor.clone()),
                            port_id: None,
                            edge_id: None,
                        });
                    } else if !visited.contains(neighbor) {
                        dfs(neighbor, adj, visited, rec_stack, errors, path);
                    }
                }
            }

            path.pop();
            rec_stack.remove(node);
        }

        for node_id in &node_ids {
            if !visited.contains(node_id) {
                let mut path = Vec::new();
                dfs(node_id, &adj, &mut visited, &mut rec_stack, &mut *errors, &mut path);
            }
        }
    }

    fn check_ports_and_edges(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>) {
        // Build node lookup
        let node_lookup: HashMap<&str, &NodeInstance> = self
            .graph
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // Build port definitions lookup
        let def_lookup: HashMap<&str, &BlockDefinition> = self
            .definitions
            .iter()
            .map(|((id, _version), def)| (id.as_str(), def))
            .collect();

        for edge in &self.graph.edges {
            let source_node = match node_lookup.get(edge.source.as_str()) {
                Some(n) => n,
                None => {
                    errors.push(ValidationError {
                        severity: Severity::Error,
                        code: "DANGLING_EDGE".to_string(),
                        message: format!("Edge references missing source node '{}'.", edge.source),
                        suggestion: None,
                        node_path: Some(edge.source.clone()),
                        port_id: Some(edge.source_handle.clone()),
                        edge_id: Some(edge.id.clone()),
                    });
                    continue;
                }
            };

            let target_node = match node_lookup.get(edge.target.as_str()) {
                Some(n) => n,
                None => {
                    errors.push(ValidationError {
                        severity: Severity::Error,
                        code: "DANGLING_EDGE".to_string(),
                        message: format!("Edge references missing target node '{}'.", edge.target),
                        suggestion: None,
                        node_path: Some(edge.target.clone()),
                        port_id: Some(edge.target_handle.clone()),
                        edge_id: Some(edge.id.clone()),
                    });
                    continue;
                }
            };

            // Look up port types
            let source_def = def_lookup.get(source_node.block_id.as_str());
            let target_def = def_lookup.get(target_node.block_id.as_str());

            let source_port_type = source_def.and_then(|d| {
                d.outputs.iter().find(|p| p.id == edge.source_handle).map(|p| &p.r#type)
            });
            let target_port_type = target_def.and_then(|d| {
                d.inputs.iter().find(|p| p.id == edge.target_handle).map(|p| &p.r#type)
            });

            match (source_port_type, target_port_type) {
                (Some(st), Some(tt)) => {
                    if !can_connect(st, tt) {
                        errors.push(ValidationError {
                            severity: Severity::Error,
                            code: "TYPE_MISMATCH".to_string(),
                            message: format!(
                                "Output '{}' (type {:?}) cannot connect to input '{}' (type {:?}).",
                                edge.source_handle, st, edge.target_handle, tt
                            ),
                            suggestion: Some("Insert a type adapter block or change the connection.".to_string()),
                            node_path: Some(edge.target.clone()),
                            port_id: Some(edge.target_handle.clone()),
                            edge_id: Some(edge.id.clone()),
                        });
                    }
                }
                _ => {
                    warnings.push(ValidationError {
                        severity: Severity::Warning,
                        code: "UNKNOWN_PORT".to_string(),
                        message: format!("Could not resolve port types for edge {}.", edge.id),
                        suggestion: None,
                        node_path: None,
                        port_id: None,
                        edge_id: Some(edge.id.clone()),
                    });
                }
            }
        }

        // Check required inputs are connected
        for node in &self.graph.nodes {
            let def = def_lookup.get(node.block_id.as_str());
            if let Some(def) = def {
                for input in &def.inputs {
                    if input.required {
                        let connected = self.graph.edges.iter().any(|e| {
                            e.target == node.id && e.target_handle == input.id
                        });
                        if !connected {
                            errors.push(ValidationError {
                                severity: Severity::Error,
                                code: "UNCONNECTED_INPUT".to_string(),
                                message: format!("Required input '{}' on node '{}' is not connected.", input.name, node.id),
                                suggestion: Some(format!("Connect a block output to the '{}' input on node '{}'.", input.name, node.id)),
                                node_path: Some(node.id.clone()),
                                port_id: Some(input.id.clone()),
                                edge_id: None,
                            });
                        }
                    }
                }
            }
        }
    }

    fn get_kind(&self, node: &NodeInstance) -> Option<BlockKind> {
        self.definitions
            .get(&(node.block_id.clone(), node.block_version))
            .map(|d| d.kind.clone())
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_def(id: &str, kind: BlockKind) -> BlockDefinition {
        BlockDefinition {
            id: id.to_string(),
            version: 1,
            name: id.to_string(),
            description: None,
            category: "test".to_string(),
            kind,
            origin: BlockOrigin::User,
            inputs: vec![],
            outputs: vec![],
            parameters: vec![],
            spec_json: serde_json::json!({}),
        }
    }

    #[test]
    fn test_valid_graph_single_trigger() {
        let mut nodes = vec![NodeInstance {
            id: "n1".to_string(),
            block_id: "builtin:input".to_string(),
            block_version: 1,
            position: Position { x: 0.0, y: 0.0 },
            parameters: HashMap::new(),
            sub_graph: None,
            is_collapsed: false,
        }];
        let mut defs = HashMap::new();
        defs.insert(("builtin:input".to_string(), 1), make_def("builtin:input", BlockKind::Trigger));

        let graph = AgentGraph {
            id: "g1".to_string(),
            name: "test".to_string(),
            description: None,
            schema_version: 1,
            nodes,
            edges: vec![],
            viewport: None,
            metadata: None,
        };

        let validator = GraphValidator::new(&graph, &defs);
        let result = validator.validate();
        assert!(result.valid, "expected valid graph, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_no_trigger() {
        let graph = AgentGraph {
            id: "g1".to_string(),
            name: "test".to_string(),
            description: None,
            schema_version: 1,
            nodes: vec![],
            edges: vec![],
            viewport: None,
            metadata: None,
        };
        let defs = HashMap::new();
        let validator = GraphValidator::new(&graph, &defs);
        let result = validator.validate();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "NO_TRIGGER"));
    }

    #[test]
    fn test_cycle_detected() {
        let mut nodes = vec![
            NodeInstance {
                id: "n1".to_string(),
                block_id: "builtin:input".to_string(),
                block_version: 1,
                position: Position { x: 0.0, y: 0.0 },
                parameters: HashMap::new(),
                sub_graph: None,
                is_collapsed: false,
            },
            NodeInstance {
                id: "n2".to_string(),
                block_id: "builtin:llm".to_string(),
                block_version: 1,
                position: Position { x: 100.0, y: 0.0 },
                parameters: HashMap::new(),
                sub_graph: None,
                is_collapsed: false,
            },
        ];
        let edges = vec![
            EdgeInstance {
                id: "e1".to_string(),
                source: "n1".to_string(),
                source_handle: "output".to_string(),
                target: "n2".to_string(),
                target_handle: "input".to_string(),
            },
            EdgeInstance {
                id: "e2".to_string(),
                source: "n2".to_string(),
                source_handle: "output".to_string(),
                target: "n1".to_string(),
                target_handle: "input".to_string(),
            },
        ];
        let mut defs = HashMap::new();
        defs.insert(("builtin:input".to_string(), 1), make_def("builtin:input", BlockKind::Trigger));
        defs.insert(("builtin:llm".to_string(), 1), make_def("builtin:llm", BlockKind::LLM));

        let graph = AgentGraph {
            id: "g1".to_string(),
            name: "test".to_string(),
            description: None,
            schema_version: 1,
            nodes,
            edges,
            viewport: None,
            metadata: None,
        };
        let validator = GraphValidator::new(&graph, &defs);
        let result = validator.validate();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "CYCLE_DETECTED"));
    }

    #[test]
    fn test_type_mismatch() {
        let mut nodes = vec![
            NodeInstance {
                id: "n1".to_string(),
                block_id: "builtin:llm".to_string(),
                block_version: 1,
                position: Position { x: 0.0, y: 0.0 },
                parameters: HashMap::new(),
                sub_graph: None,
                is_collapsed: false,
            },
            NodeInstance {
                id: "n2".to_string(),
                block_id: "builtin:control".to_string(),
                block_version: 1,
                position: Position { x: 100.0, y: 0.0 },
                parameters: HashMap::new(),
                sub_graph: None,
                is_collapsed: false,
            },
        ];
        // LLM outputs String, Control expects Number — mismatch
        let mut defs = HashMap::new();
        defs.insert(
            ("builtin:llm".to_string(), 1),
            BlockDefinition {
                id: "builtin:llm".to_string(),
                version: 1,
                name: "LLM".to_string(),
                description: None,
                category: "llm".to_string(),
                kind: BlockKind::LLM,
                origin: BlockOrigin::User,
                inputs: vec![],
                outputs: vec![BlockPort {
                    id: "completion".to_string(),
                    name: "completion".to_string(),
                    r#type: PortType::String,
                    direction: PortDirection::Output,
                    required: false,
                    default_value: None,
                }],
                parameters: vec![],
                spec_json: serde_json::json!({}),
            },
        );
        defs.insert(
            ("builtin:control".to_string(), 1),
            BlockDefinition {
                id: "builtin:control".to_string(),
                version: 1,
                name: "Control".to_string(),
                description: None,
                category: "control".to_string(),
                kind: BlockKind::Control,
                origin: BlockOrigin::User,
                inputs: vec![BlockPort {
                    id: "count".to_string(),
                    name: "count".to_string(),
                    r#type: PortType::Number,
                    direction: PortDirection::Input,
                    required: true,
                    default_value: None,
                }],
                outputs: vec![],
                parameters: vec![],
                spec_json: serde_json::json!({}),
            },
        );

        let edges = vec![EdgeInstance {
            id: "e1".to_string(),
            source: "n1".to_string(),
            source_handle: "completion".to_string(),
            target: "n2".to_string(),
            target_handle: "count".to_string(),
        }];

        let graph = AgentGraph {
            id: "g1".to_string(),
            name: "test".to_string(),
            description: None,
            schema_version: 1,
            nodes,
            edges,
            viewport: None,
            metadata: None,
        };
        let validator = GraphValidator::new(&graph, &defs);
        let result = validator.validate();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "TYPE_MISMATCH"));
    }

#[test]
fn test_can_connect_any() {
    assert!(can_connect(&PortType::Any, &PortType::String));
    assert!(can_connect(&PortType::String, &PortType::Any));
    assert!(can_connect(&PortType::String, &PortType::String));
    assert!(!can_connect(&PortType::String, &PortType::Number));
}

// ============================================================================
// Connector / Tool Management tests
// ============================================================================

#[test]
fn test_tool_management_always_allow() {
    let config = ToolManagementConfig {
        connector_id: "conn-1".to_string(),
        preset: ToolApprovalMode::AlwaysAllow,
        overrides: vec![],
    };

    assert_eq!(config.check_tool("read_data"), (true, false));
    assert_eq!(config.check_tool("delete_record"), (true, false));
    assert_eq!(config.check_tool("create_record"), (true, false));
}

#[test]
fn test_tool_management_ask_each_time() {
    let config = ToolManagementConfig {
        connector_id: "conn-1".to_string(),
        preset: ToolApprovalMode::AskEachTime,
        overrides: vec![],
    };

    assert_eq!(config.check_tool("read_data"), (false, true));
    assert_eq!(config.check_tool("delete_record"), (false, true));
    assert_eq!(config.check_tool("create_record"), (false, true));
}

#[test]
fn test_tool_management_ask_for_writes_deletes() {
    let config = ToolManagementConfig {
        connector_id: "conn-1".to_string(),
        preset: ToolApprovalMode::AskForWritesDeletes,
        overrides: vec![],
    };

    // Read-only tools are allowed without approval
    assert_eq!(config.check_tool("read_data"), (true, false));
    assert_eq!(config.check_tool("get_records"), (true, false));
    assert_eq!(config.check_tool("list_items"), (true, false));

    // Write/delete tools require approval
    assert_eq!(config.check_tool("delete_record"), (false, true));
    assert_eq!(config.check_tool("create_record"), (false, true));
    assert_eq!(config.check_tool("update_record"), (false, true));
    assert_eq!(config.check_tool("post_data"), (false, true));
    assert_eq!(config.check_tool("put_item"), (false, true));
    assert_eq!(config.check_tool("patch_record"), (false, true));
    assert_eq!(config.check_tool("modify_data"), (false, true));
}

#[test]
fn test_tool_management_custom_overrides() {
    let config = ToolManagementConfig {
        connector_id: "conn-1".to_string(),
        preset: ToolApprovalMode::Custom,
        overrides: vec![
            ToolOverride {
                tool_id: "read_data".to_string(),
                mode: ToolOverrideMode::Allow,
            },
            ToolOverride {
                tool_id: "delete_record".to_string(),
                mode: ToolOverrideMode::Deny,
            },
            ToolOverride {
                tool_id: "create_record".to_string(),
                mode: ToolOverrideMode::Ask,
            },
        ],
    };

    // Overridden tools follow their override
    assert_eq!(config.check_tool("read_data"), (true, false));
    assert_eq!(config.check_tool("delete_record"), (false, false));
    assert_eq!(config.check_tool("create_record"), (false, true));

    // Non-overridden tools default to ask under Custom preset
    assert_eq!(config.check_tool("list_items"), (false, true));
}

#[test]
fn test_tools_requiring_approval() {
    let config = ToolManagementConfig {
        connector_id: "conn-1".to_string(),
        preset: ToolApprovalMode::AskForWritesDeletes,
        overrides: vec![],
    };

    let all_tools = vec![
        "read_data".to_string(),
        "get_records".to_string(),
        "delete_record".to_string(),
        "create_record".to_string(),
        "list_items".to_string(),
    ];

    let requiring = config.tools_requiring_approval(&all_tools);
    assert_eq!(requiring.len(), 2);
    assert!(requiring.contains(&"delete_record".to_string()));
    assert!(requiring.contains(&"create_record".to_string()));
}

#[test]
fn test_connector_block_creation() {
    let config = ToolManagementConfig {
        connector_id: "gmail-1".to_string(),
        preset: ToolApprovalMode::AskForWritesDeletes,
        overrides: vec![],
    };

    let connector = ConnectorBlock {
        id: "conn-gmail".to_string(),
        name: "Gmail".to_string(),
        category: "email".to_string(),
        account_selection: AccountSelection::PersonalDefault,
        tool_management: config,
        is_builtin: true,
        is_custom: false,
        created_at: "2026-10-29T00:00:00Z".to_string(),
        updated_at: "2026-10-29T00:00:00Z".to_string(),
    };

    assert_eq!(connector.name, "Gmail");
    assert_eq!(connector.category, "email");
    assert!(connector.is_builtin);
    assert!(!connector.is_custom);
    assert!(matches!(
        connector.account_selection,
        AccountSelection::PersonalDefault
    ));
}

#[test]
fn test_connector_grant() {
    let grant = ConnectorGrant {
        agent_id: "agent-1".to_string(),
        connector_id: "conn-gmail".to_string(),
        account_id: Some("user@gmail.com".to_string()),
        granted_at: "2026-10-29T00:00:00Z".to_string(),
    };

    assert_eq!(grant.agent_id, "agent-1");
    assert_eq!(grant.connector_id, "conn-gmail");
    assert_eq!(grant.account_id, Some("user@gmail.com".to_string()));
}
}

// ============================================================================
// Connector types (Gumloop model)
// ============================================================================

/// Account selection for connector blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountSelection {
    PersonalDefault,
    SpecificAccount(String),
    TeamDefault,
}

/// Tool Management approval preset modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolApprovalMode {
    AlwaysAllow,
    AskEachTime,
    AskForWritesDeletes,
    Custom,
}

/// Per-tool override when preset is Custom.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOverride {
    pub tool_id: String,
    pub mode: ToolOverrideMode,
}

/// Override mode for a specific tool (simpler than preset modes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolOverrideMode {
    Allow,
    Ask,
    Deny,
}

/// Tool Management configuration for a connector block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolManagementConfig {
    pub connector_id: String,
    pub preset: ToolApprovalMode,
    pub overrides: Vec<ToolOverride>,
}

/// A connector block definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorBlock {
    pub id: String,
    pub name: String,
    pub category: String,
    pub account_selection: AccountSelection,
    pub tool_management: ToolManagementConfig,
    pub is_builtin: bool,
    pub is_custom: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A connector grant (which account an agent uses for a connector).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorGrant {
    pub agent_id: String,
    pub connector_id: String,
    pub account_id: Option<String>,
    pub granted_at: String,
}

impl ToolManagementConfig {
    /// Check if a specific tool is allowed to run without approval.
    /// Returns (allowed, requires_approval).
    pub fn check_tool(&self, tool_id: &str) -> (bool, bool) {
        // Check overrides first
        if let Some(ov) = self.overrides.iter().find(|o| o.tool_id == tool_id) {
            match ov.mode {
                ToolOverrideMode::Allow => return (true, false),
                ToolOverrideMode::Ask => return (false, true),
                ToolOverrideMode::Deny => return (false, false),
            }
        }

        // Fall back to preset
        match self.preset {
            ToolApprovalMode::AlwaysAllow => (true, false),
            ToolApprovalMode::AskEachTime => (false, true),
            ToolApprovalMode::AskForWritesDeletes => {
                let is_write = self.is_write_tool(tool_id);
                if is_write {
                    (false, true)
                } else {
                    (true, false)
                }
            }
            ToolApprovalMode::Custom => {
                // Custom with no matching override defaults to ask
                (false, true)
            }
        }
    }

    /// Heuristic: determine if a tool is a write/delete operation based on name.
    fn is_write_tool(&self, tool_id: &str) -> bool {
        let lower = tool_id.to_lowercase();
        lower.contains("delete")
            || lower.contains("remove")
            || lower.contains("create")
            || lower.contains("update")
            || lower.contains("write")
            || lower.contains("post")
            || lower.contains("put")
            || lower.contains("patch")
            || lower.contains("modify")
    }

    /// Get all tool IDs that require approval.
    pub fn tools_requiring_approval(&self, all_tool_ids: &[String]) -> Vec<String> {
        all_tool_ids
            .iter()
            .filter(|tid| {
                let (allowed, requires_approval) = self.check_tool(tid);
                requires_approval && !allowed
            })
            .cloned()
            .collect()
    }
}

impl Default for ToolManagementConfig {
    fn default() -> Self {
        Self {
            connector_id: String::new(),
            preset: ToolApprovalMode::AskForWritesDeletes,
            overrides: Vec::new(),
        }
    }
}

/// Skill definition — teaches an agent how to do specific work "your way".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub kind: SkillKind,
    pub ai_editing_enabled: bool,
    pub instructions: String,
    pub attached_to: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Skill creation kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillKind {
    CreateWithAI,
    UploadFiles,
    WriteInstructions,
    AddExisting,
}

/// Knowledge source for RAG with citations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub source_ids: Vec<String>,
    pub citations_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Trigger definition — how an agent starts on its own.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trigger {
    pub id: String,
    pub kind: TriggerKind,
    pub config: serde_json::Value,
    pub ai_managed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Trigger kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerKind {
    AppTrigger,
    Webhook,
    Scheduled,
    OneTime,
    CreateWithAI,
}

// End of file
