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

export interface ApprovalRequest {
  id: string;
  agentId: string;
  taskId: string;
  command: string;
  context: string;
  status: 'pending' | 'approved' | 'denied' | 'cancelled';
  response: string | null;
  requestedAt: string;
  respondedAt: string | null;
}

export interface CreateApprovalInput {
  agentId: string;
  command: string;
  context: string;
}

export interface ApprovalStatus {
  taskId: string;
  status: 'pending' | 'approved' | 'denied' | 'cancelled';
  response: string | null;
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

export interface TemplateSource {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  addedAt: string;
}

export interface AddTemplateSourceInput {
  name: string;
  url: string;
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

export interface InstallTemplateInput {
  templateId: string;
  version: string;
  sourceId: string;
}

export interface InstallTemplateResult {
  agentId: string;
  name: string;
  templateId: string;
  version: string;
}

export interface PlanDiff {
  field: string;
  diffType: string;
  before: unknown | null;
  after: unknown | null;
}

export interface PlanDiffResult {
  versionAId: string;
  versionBId: string;
  diffs: PlanDiff[];
  similarity: number;
}

export interface DiffPlanVersionsInput {
  versionIdA: string;
  versionIdB: string;
}
