//! Run and pinned data models for Opifex v2.0.0 execution engine.
//! Provides types for tracking run history and pinned node outputs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a run (whole graph execution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Status of an individual node run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRunStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}

/// A top-level run record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub graph_id: String,
    pub graph_hash: String,
    pub mode: RunMode,
    pub scope_json: serde_json::Value,
    pub trigger_json: Option<serde_json::Value>,
    pub status: RunStatus,
    pub error_json: Option<serde_json::Value>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

/// Execution mode for a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    Live,
    Mock,
}

/// An individual node run within a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRun {
    pub id: String,
    pub run_id: String,
    pub parent_node_run_id: Option<String>,
    pub node_path: String,
    pub attempt: u32,
    pub status: NodeRunStatus,
    pub input_json: Option<serde_json::Value>,
    pub output_json: Option<serde_json::Value>,
    pub error_json: Option<serde_json::Value>,
    pub truncated: bool,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

/// A pinned node output for iterative development.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedData {
    pub graph_id: String,
    pub node_id: String,
    pub port_id: String,
    pub value_json: serde_json::Value,
    pub source_node_run_id: Option<String>,
    pub created_at: String,
}

/// Pinned data with node context for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedDataWithContext {
    pub graph_id: String,
    pub node_id: String,
    pub port_id: String,
    pub value: serde_json::Value,
    pub source_node_run_id: Option<String>,
    pub created_at: String,
    pub node_name: Option<String>,
    pub port_name: Option<String>,
}

/// Input data for executing a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionInput {
    pub graph_id: String,
    pub node_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub pinned_overrides: HashMap<String, serde_json::Value>,
}

/// Output data from executing a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionOutput {
    pub status: NodeRunStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub is_pinned: bool,
}

impl Default for RunStatus {
    fn default() -> Self {
        RunStatus::Running
    }
}

impl Default for NodeRunStatus {
    fn default() -> Self {
        NodeRunStatus::Queued
    }
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Mock
    }
}

impl Default for Run {
    fn default() -> Self {
        Self {
            id: String::new(),
            graph_id: String::new(),
            graph_hash: String::new(),
            mode: RunMode::Mock,
            scope_json: serde_json::json!({}),
            trigger_json: None,
            status: RunStatus::Running,
            error_json: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: None,
        }
    }
}

impl NodeRun {
    pub fn new(run_id: String, node_path: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            run_id,
            parent_node_run_id: None,
            node_path,
            attempt: 1,
            status: NodeRunStatus::Queued,
            input_json: None,
            output_json: None,
            error_json: None,
            truncated: false,
            tokens_in: None,
            tokens_out: None,
            started_at: None,
            ended_at: None,
        }
    }
}
