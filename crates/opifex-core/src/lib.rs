//! lib.rs for opifex-core — public API surface.

pub mod expression;
pub mod migrations;
pub mod store;
pub mod validator;

pub use expression::{parse_expression, Expr, Literal, BinaryOp, ParseError};
pub use migrations::run_v2_migrations;
pub use store::{
    NodeExecutionInput, NodeExecutionOutput, NodeRun, NodeRunStatus, PinnedData,
    PinnedDataWithContext, Run, RunMode, RunStatus,
};
pub use validator::{
    AccountSelection, AgentGraph, BlockKind, BlockOrigin, BlockParameter, BlockPort,
    Channel, ChannelKind, ConnectorBlock, ConnectorGrant, EdgeInstance, GraphValidator,
    InstructionChange, InstructionChangeStatus, KnowledgeSource, NodeInstance, ParameterType,
    PortConformanceVector, PortDirection, PortType, Position, Severity, Skill, SkillKind,
    SubagentConfig, ToolApprovalMode, ToolManagementConfig, ToolOverride, ToolOverrideMode,
    Trigger, TriggerKind, ValidationError, ValidationResult, Viewport, can_connect,
};
