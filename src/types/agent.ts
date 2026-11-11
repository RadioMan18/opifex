/**
 * Agent plan schema v2 — every chain is trigger → action → result → gate.
 *
 * Triggers and results are *declared ports* on the plan; a chain references them
 * by name. That is what makes the spec composable: a chain starts when a trigger
 * fires or when an upstream result lands, so results feed triggers instead of
 * being decorative strings. A chain that starts from an upstream result simply
 * names that result in `trigger`.
 */

/** Derived view of a chain, kept for the flat plan pane and version diff. */
export interface AgentPlanStep {
  action: string;
  inputs: string[];
  requiresApproval: boolean;
  approvalContext?: string;
}

export type TriggerKind = 'event' | 'schedule' | 'intent';

/** Something that can start the agent. */
export interface TriggerPort {
  name: string;
  kind: TriggerKind;
  description?: string;
  /** Cron or interval expression; required when kind is 'schedule'. */
  schedule?: string;
}

/** A named artifact. `terminal` marks a deliverable rather than an internal handoff. */
export interface ResultPort {
  name: string;
  terminal: boolean;
  description?: string;
  /** Expected shape, e.g. 'markdown_table[Rank,Tool,Stars]'. */
  schema?: string;
}

export type BindingSource = 'trigger' | 'result' | 'parameter' | 'literal';

/** How one input of an action is filled. */
export interface InputBinding {
  port: string;
  from: BindingSource;
  /** Trigger or result port name for 'trigger' / 'result' bindings. */
  ref?: string;
  value?: string;
}

export type GateMode = 'auto' | 'approve';
export type FailurePolicy = 'stop' | 'retry' | 'escalate';

/** Who authorises the action, and what happens when the result is not acceptable. */
export interface ChainGate {
  mode: GateMode;
  approver?: string;
  onFailure: FailurePolicy;
  maxRetries?: number;
}

export interface AgentChain {
  id: string;
  /** Trigger port name, or the name of an upstream result port. */
  trigger: string;
  /** Callable tool or action id. */
  capability: string;
  inputs: InputBinding[];
  /** Result port this chain produces. */
  result: string;
  /** Reasoning contract: what the model may decide, and what it must not. */
  decision?: string;
  gate: ChainGate;
}

export interface AgentPlan {
  version: 2;
  goal?: string;
  triggers: TriggerPort[];
  results: ResultPort[];
  chains: AgentChain[];
  tools: string[];
  knowledgeSources: string[];
}

export interface PlanSummary {
  goal: string;
  steps: string;
  triggers: string;
  outputs: string;
  tools: string;
  knowledgeSources: string;
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
