export interface AgentPlan {
  goal?: string;
  steps?: string[];
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
