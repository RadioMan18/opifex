export interface AgentPlanStep {
  action: string;
  inputs: string[];
  requiresApproval: boolean;
  approvalContext?: string;
}

export interface AgentPlan {
  goal?: string;
  steps?: AgentPlanStep[];
  tools?: string[];
  knowledgeSources?: string[];
  triggers?: string[];
  outputs?: string[];
}

export interface Agent {
  id: string;
  name: string;
  description: string | null;
  specJson: AgentPlan;
  model: string;
  isStarter: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AgentListItem {
  id: string;
  name: string;
  description: string | null;
  model: string;
  isStarter: boolean;
  updatedAt: string;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  turnPlanJson: AgentPlan | null;
  createdAt: string;
}

export interface Conversation {
  id: string;
  agentId: string;
  title: string;
  createdAt: string;
}

export interface DbMessage {
  id: string;
  conversationId: string;
  role: string;
  content: string;
  turnPlanJson: AgentPlan | null;
  createdAt: string;
}

export interface PlanVersion {
  id: string;
  agentId: string;
  label: string | null;
  specJson: AgentPlan;
  createdAt: string;
}

export interface CreateAgentInput {
  name: string;
  description: string | null;
  specJson: AgentPlan;
  model: string | null;
  isStarter: boolean | null;
}

export interface UpdateAgentInput {
  name: string | null;
  description: string | null;
  specJson: AgentPlan | null;
  model: string | null;
  isStarter: boolean | null;
}

export interface CreateConversationInput {
  agentId: string;
  title: string;
}

export interface SaveMessageInput {
  conversationId: string;
  role: string;
  content: string;
  turnPlanJson: AgentPlan | null;
}

export interface SavePlanVersionInput {
  agentId: string;
  label: string | null;
  specJson: AgentPlan;
}

export interface RestorePlanVersionInput {
  versionId: string;
}

export interface OllamaModel {
  name: string;
  size: number | null;
  modifiedAt: string | null;
  digest: string | null;
}

export interface TemplateSource {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  addedAt: string;
}

export interface TemplateEntry {
  id: string;
  name: string;
  description: string | null;
  version: string;
  author: string | null;
  downloadUrl: string | null;
  checksum: string | null;
  specJson: unknown | null;
  model: string | null;
  isStarter: boolean;
}

export interface InstalledTemplate {
  id: string;
  templateId: string;
  version: string;
  installedAt: string;
}

export interface OllamaMessage {
  role: string;
  content: string;
}

export interface OllamaChatInput {
  model: string;
  messages: Array<{ role: string; content: string }>;
}

export interface OllamaChatOutput {
  content: string;
}

export type LlmProvider = 'ollama' | 'openai' | 'anthropic';

export type PortType =
  | 'string'
  | 'number'
  | 'boolean'
  | 'object'
  | 'any'
  | 'message'
  | 'llm'
  | 'tool'
  | 'json';

export type PortDirection = 'input' | 'output';

export type BlockKind =
  | 'trigger'
  | 'llm'
  | 'agent'
  | 'tool'
  | 'control'
  | 'output'
  | 'utility'
  | 'subgraph';

export type BlockOrigin = 'user' | 'imported';

export interface BlockPort {
  id: string;
  name: string;
  type: PortType;
  direction: PortDirection;
  required: boolean;
  defaultValue?: unknown;
  description?: string;
}

export interface BlockParameter {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'select' | 'multi-select' | 'textarea' | 'code' | 'credential';
  label: string;
  required: boolean;
  defaultValue?: unknown;
  options?: string[];
  description?: string;
  sensitive: boolean;
}

export interface BlockDefinition {
  id: string;
  version: number;
  name: string;
  description?: string;
  category: string;
  kind: BlockKind;
  origin: BlockOrigin;
  inputs: BlockPort[];
  outputs: BlockPort[];
  parameters: BlockParameter[];
  specJson: unknown;
  createdAt: string;
  updatedAt: string;
  // Connector-specific fields (populated when kind === 'tool' and block is a connector)
  account_selection?: AccountSelection;
  tool_management?: ToolManagementConfig;
}

export interface NodeInstance {
  id: string;
  blockId: string;
  blockVersion: number;
  position: { x: number; y: number };
  parameters: Record<string, unknown>;
  subGraph?: AgentGraph;
  isCollapsed: boolean;
}

export interface EdgeInstance {
  id: string;
  source: string;
  target: string;
}

export interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

export interface AgentGraph {
  id: string;
  name: string;
  description?: string;
  schemaVersion: number;
  nodes: NodeInstance[];
  edges: EdgeInstance[];
  viewport?: Viewport;
  metadata?: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export type Severity = 'error' | 'warning';

export interface ValidationError {
  severity: Severity;
  code: string;
  message: string;
  suggestion?: string;
  nodePath?: string;
  portId?: string;
  edgeId?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  graphHash: string;
}

export interface PortConformanceVector {
  sourceType: PortType;
  targetType: PortType;
  expectedResult: boolean;
}

export interface ConformanceCheckResult {
  allowed: boolean;
  reason?: string;
}

// Connector account selection (Gumloop model)
export type AccountSelection = 'personal-default' | 'specific-account' | 'team-default';

// Tool Management approval presets
export type ToolApprovalMode = 'always-allow' | 'ask-each-time' | 'ask-for-writes-deletes' | 'custom';

// Per-tool override when preset is 'custom'
export interface ToolOverride {
  tool_id: string;
  mode: 'allow' | 'ask' | 'deny';
}

export interface ToolManagementConfig {
  connector_id: string;
  preset: ToolApprovalMode;
  overrides: ToolOverride[];
}

export interface ConnectorBlock {
  id: string;
  name: string;
  category: string;
  account_selection: AccountSelection;
  tool_management: ToolManagementConfig;
  is_builtin: boolean;
  is_custom: boolean;
  created_at: string;
  updated_at: string;
}

export interface ConnectorGrant {
  agent_id: string;
  connector_id: string;
  account_id: string | null;
  granted_at: string;
}

// Skill types (Phase 4)
export type SkillKind = 'create-with-ai' | 'upload-files' | 'write-instructions' | 'add-existing';

export interface Skill {
  id: string;
  name: string;
  description?: string;
  kind: SkillKind;
  ai_editing_enabled: boolean;
  instructions: string;
  attached_to: string[];
  created_at: string;
  updated_at: string;
}

// Knowledge Source types (Phase 4)
export interface KnowledgeSource {
  id: string;
  name: string;
  provider: string;
  source_ids: string[];
  citations_enabled: boolean;
  created_at: string;
  updated_at: string;
}

// Trigger types (Phase 4)
export type TriggerKind = 'app-trigger' | 'webhook' | 'scheduled' | 'one-time' | 'create-with-ai';

export interface Trigger {
  id: string;
  kind: TriggerKind;
  config: any;
  ai_managed: boolean;
  created_at: string;
  updated_at: string;
}

// Subagent types (Phase 5)
export interface SubagentConfig {
  agent_id: string;
  task: string;
  max_concurrent: number;
  wait_for_results: boolean;
  depth_limit: number;
}

export interface SubagentDto {
  id: string;
  agent_id: string;
  task: string;
  max_concurrent: number;
  wait_for_results: boolean;
  depth_limit: number;
  created_at: string;
  updated_at: string;
}

// Ability types (Phase 5)
export type AbilityKind =
  | 'web-search'
  | 'web-fetch'
  | 'image-generation'
  | 'chat-widgets'
  | 'search-past-conversations'
  | 'ask-question'
  | 'tool-discovery'
  | 'rules-creation';

export interface AbilityDto {
  kind: AbilityKind;
  enabled: boolean;
}

// Channel types (Phase 5)
export type ChannelKind =
  | 'email-inbox'
  | 'slack'
  | 'teams'
  | 'hosted-page'
  | 'developer'
  | 'mcp';

export interface ChannelDto {
  id: string;
  kind: ChannelKind;
  config: any;
  created_at: string;
  updated_at: string;
}
