import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  Agent,
  AgentListItem,
  AgentPlan,
  Conversation,
  CreateAgentInput,
  CreateConversationInput,
  DbMessage,
  InstalledTemplate,
  OllamaModel,
  OllamaMessage,
  OllamaChatOutput,
  PlanVersion,
  SaveMessageInput,
  SavePlanVersionInput,
  TemplateEntry,
  TemplateSource,
  UpdateAgentInput,
} from '@/types/agent';

// Connector IPC types (snake_case to match Rust backend)
interface CreateConnectorInput {
  name: string;
  category: string;
  account_selection: 'personal-default' | 'specific-account' | 'team-default';
  tool_management: {
    connector_id: string;
    preset: 'always-allow' | 'ask-each-time' | 'ask-for-writes-deletes' | 'custom';
    overrides: { tool_id: string; mode: 'allow' | 'ask' | 'deny' }[];
  };
}

interface UpdateConnectorInput {
  id: string;
  name?: string;
  category?: string;
  account_selection?: 'personal-default' | 'specific-account' | 'team-default';
  tool_management?: {
    connector_id: string;
    preset: 'always-allow' | 'ask-each-time' | 'ask-for-writes-deletes' | 'custom';
    overrides: { tool_id: string; mode: 'allow' | 'ask' | 'deny' }[];
  };
}

interface GrantConnectorInput {
  agent_id: string;
  connector_id: string;
  account_id?: string;
}



interface ConnectorBlock {
  id: string;
  name: string;
  category: string;
  account_selection: 'personal-default' | 'specific-account' | 'team-default';
  tool_management: {
    connector_id: string;
    preset: 'always-allow' | 'ask-each-time' | 'ask-for-writes-deletes' | 'custom';
    overrides: { tool_id: string; mode: 'allow' | 'ask' | 'deny' }[];
  };
  is_builtin: boolean;
  is_custom: boolean;
  created_at: string;
  updated_at: string;
}

interface ConnectorGrant {
  agent_id: string;
  connector_id: string;
  account_id: string | null;
  granted_at: string;
}

export async function createAgent(input: CreateAgentInput): Promise<Agent> {
  return invoke('create_agent', { input });
}

export async function getAgent(id: string): Promise<Agent | null> {
  return invoke('get_agent', { id });
}

export async function listAgents(includeStarters?: boolean): Promise<AgentListItem[]> {
  return invoke('list_agents', { includeStarters: includeStarters ?? true });
}

export async function updateAgent(id: string, input: UpdateAgentInput): Promise<Agent> {
  return invoke('update_agent', { id, input });
}

export async function deleteAgent(id: string): Promise<boolean> {
  return invoke('delete_agent', { id });
}

export async function exportAgent(id: string): Promise<string> {
  return invoke('export_agent', { id });
}

export async function importAgent(agentJson: string): Promise<Agent> {
  return invoke('import_agent', { agent_json: agentJson });
}

export async function renameAgent(id: string, newName: string): Promise<Agent> {
  return invoke('rename_agent', { id, newName });
}

export async function duplicateAgent(id: string, newName?: string): Promise<Agent> {
  return invoke('duplicate_agent', { id, newName: newName ?? null });
}

export async function seedStarterAgents(): Promise<string[]> {
  return invoke('seed_starter_agents');
}

export async function ollamaListModels(): Promise<OllamaModel[]> {
  return invoke('ollama_list_models');
}

export async function ollamaPullModel(model: string): Promise<string> {
  return invoke('ollama_pull_model', { model });
}

export async function ollamaChat(
  provider: string,
  model: string,
  messages: OllamaMessage[],
): Promise<OllamaChatOutput> {
  return invoke('ollama_chat', { provider, model, messages });
}

export interface OllamaStreamCallbacks {
  onChunk: (content: string) => void;
  onEnd: () => void;
  onError: (error: string) => void;
}

export type StreamCleanup = () => void;

export async function ollamaChatStream(
  provider: string,
  model: string,
  apiKey: string | null,
  messages: OllamaMessage[],
  callbacks: OllamaStreamCallbacks,
): Promise<StreamCleanup> {
  let isDone = false;
  const unlistens: UnlistenFn[] = [];

  const cleanup: StreamCleanup = () => {
    if (isDone) return;
    isDone = true;
    while (unlistens.length > 0) {
      const u = unlistens.pop();
      try {
        u?.();
      } catch {}
    }
  };

  const chunkUnlisten = await listen<{ content: string }>('ollama-stream-chunk', (event) => {
    if (!isDone) {
      callbacks.onChunk(event.payload.content);
    }
  });
  if (isDone) {
    chunkUnlisten();
  } else {
    unlistens.push(chunkUnlisten);
  }

  const endUnlisten = await listen<{}>('ollama-stream-end', () => {
    if (!isDone) {
      cleanup();
      callbacks.onEnd();
    }
  });
  if (isDone) {
    endUnlisten();
  } else {
    unlistens.push(endUnlisten);
  }

  const errorUnlisten = await listen<{ error: string }>('ollama-stream-error', (event) => {
    if (!isDone) {
      cleanup();
      callbacks.onError(event.payload.error);
    }
  });
  if (isDone) {
    errorUnlisten();
  } else {
    unlistens.push(errorUnlisten);
  }

  // Trigger the Tauri command in the background; do not block returning cleanup
  invoke('ollama_chat_stream', { provider, model, apiKey, messages }).catch((err: any) => {
    if (!isDone) {
      cleanup();
      callbacks.onError(err?.message || String(err));
    }
  });

  return cleanup;
}

export async function ollamaGetDefaultModel(): Promise<string | null> {
  return invoke('ollama_get_default_model');
}

export async function ollamaSetDefaultModel(model: string): Promise<void> {
  return invoke('ollama_set_default_model', { model });
}

export async function ollamaGetProvider(): Promise<string | null> {
  return invoke('ollama_get_provider');
}

export async function ollamaSetProvider(provider: string): Promise<void> {
  return invoke('ollama_set_provider', { provider });
}

export async function ollamaGetApiKey(provider: string): Promise<string | null> {
  return invoke('ollama_get_api_key', { provider });
}

export async function ollamaSetApiKey(provider: string, apiKey: string): Promise<void> {
  return invoke('ollama_set_api_key', { provider, apiKey });
}

export async function createConversation(input: CreateConversationInput): Promise<Conversation> {
  return invoke('create_conversation', { input });
}

export async function loadUserAgents(): Promise<string[]> {
  return invoke('load_user_agents');
}

export async function getConversation(id: string): Promise<Conversation | null> {
  return invoke('get_conversation', { id });
}

export async function listConversations(agentId: string): Promise<Conversation[]> {
  return invoke('list_conversations', { agentId });
}

export async function saveMessage(input: SaveMessageInput): Promise<DbMessage> {
  return invoke('save_message', { input });
}

export async function loadMessages(conversationId: string): Promise<DbMessage[]> {
  return invoke('load_messages', { conversationId });
}

export async function savePlanVersion(input: SavePlanVersionInput): Promise<PlanVersion> {
  return invoke('save_plan_version', { input });
}

export async function listPlanVersions(agentId: string): Promise<PlanVersion[]> {
  return invoke('list_plan_versions', { agentId });
}

export async function restorePlanVersion(versionId: string): Promise<AgentPlan> {
  return invoke('restore_plan_version', { versionId });
}

export async function writeFile(path: string, content: string): Promise<void> {
  return invoke('write_file', { path, content });
}

export async function addTemplateSource(input: { name: string; url: string }): Promise<TemplateSource> {
  return invoke('add_template_source', { input });
}

export async function listTemplateSources(): Promise<TemplateSource[]> {
  return invoke('list_template_sources');
}

export async function fetchTemplates(sourceId: string): Promise<TemplateEntry[]> {
  return invoke('fetch_templates', { sourceId });
}

export async function installTemplate(input: { templateId: string; version: string; sourceId: string }): Promise<{
  agentId: string;
  name: string;
  templateId: string;
  version: string;
}> {
  return invoke('install_template', { input });
}

export async function listInstalledTemplates(): Promise<InstalledTemplate[]> {
  return invoke('list_installed_templates');
}

// Connector IPC
export async function createConnector(input: CreateConnectorInput): Promise<ConnectorBlock> {
  return invoke('create_connector', { input });
}

export async function getConnector(id: string): Promise<ConnectorBlock | null> {
  return invoke('get_connector', { id });
}

export async function listConnectors(category?: string): Promise<ConnectorBlock[]> {
  return invoke('list_connectors', { category });
}

export async function updateConnector(input: UpdateConnectorInput): Promise<ConnectorBlock> {
  return invoke('update_connector', { input });
}

export async function deleteConnector(id: string): Promise<boolean> {
  return invoke('delete_connector', { id });
}

export async function grantConnector(input: GrantConnectorInput): Promise<ConnectorGrant> {
  return invoke('grant_connector', { input });
}

export async function revokeConnector(agentId: string, connectorId: string): Promise<boolean> {
  return invoke('revoke_connector', { agent_id: agentId, connector_id: connectorId });
}

export async function listConnectorGrants(agentId: string): Promise<ConnectorGrant[]> {
  return invoke('list_connector_grants', { agent_id: agentId });
}

export async function checkToolApproval(
  toolId: string,
  connectorId: string,
  preset: string,
  overrides?: { toolId: string; mode: 'allow' | 'ask' | 'deny' }[],
): Promise<{ allowed: boolean; requiresApproval: boolean }> {
  const result = await invoke<{ allowed: boolean; requires_approval: boolean }>(
    'check_tool_approval',
    { tool_id: toolId, connector_id: connectorId, preset, overrides },
  );
  return { allowed: result.allowed, requiresApproval: result.requires_approval };
}

// Run IPC types (snake_case to match Rust backend)
interface RunAgentGraphInput {
  agent_id: string;
  mode: 'live' | 'mock';
}

interface PinDataInput {
  agent_id: string;
  node_id: string;
  port_id: string;
  value: any;
}

interface UnpinDataInput {
  agent_id: string;
  node_id: string;
  port_id: string;
}

interface GetPinnedDataInput {
  agent_id: string;
}

interface PinnedDataDto {
  node_id: string;
  port_id: string;
  value: any;
  created_at: string;
}

interface RunStatusDto {
  run_id: string;
  agent_id: string;
  status: string;
  started_at: string;
  ended_at: string | null;
  node_count: number;
}

interface Run {
  id: string;
  graph_id: string;
  graph_hash: string;
  mode: string;
  scope_json: any;
  trigger_json: any | null;
  status: string;
  error_json: any | null;
  started_at: string;
  ended_at: string | null;
}

// Run IPC
export async function runAgentGraph(input: RunAgentGraphInput): Promise<Run> {
  return invoke('run_agent_graph', { input });
}

export async function pinAgentData(input: PinDataInput): Promise<void> {
  return invoke('pin_agent_data', { input });
}

export async function unpinAgentData(input: UnpinDataInput): Promise<void> {
  return invoke('unpin_agent_data', { input });
}

export async function getPinnedData(input: GetPinnedDataInput): Promise<PinnedDataDto[]> {
  return invoke('get_pinned_data', { input });
}

export async function getRunStatus(input: GetPinnedDataInput): Promise<RunStatusDto | null> {
  return invoke('get_run_status', { input });
}
