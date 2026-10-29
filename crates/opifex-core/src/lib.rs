//! lib.rs for opifex-core — public API surface.

pub mod expression;
pub mod migrations;
pub mod validator;

pub use expression::{parse_expression, Expr, Literal, BinaryOp, ParseError};
pub use migrations::run_v2_migrations;
pub use validator::{
    AccountSelection, AgentGraph, BlockDefinition, BlockKind, BlockOrigin, BlockParameter, BlockPort,
    ConnectorBlock, ConnectorGrant, EdgeInstance, GraphValidator, NodeInstance, ParameterType,
    PortConformanceVector, PortDirection, PortType, Position, Severity, ToolApprovalMode,
    ToolManagementConfig, ToolOverride, ToolOverrideMode, ValidationError, ValidationResult,
    Viewport, can_connect,
};
