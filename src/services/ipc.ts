import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  Agent,
  AgentListItem,
  AgentPlan,
  AddTemplateSourceInput,
  ApprovalRequest,
  ApprovalStatus,
  Conversation,
  CreateAgentInput,
  CreateApprovalInput,
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
  PlanDiffResult,
} from '@/types/agent';

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

export async function ollamaWarmupModel(model: string): Promise<void> {
  return invoke('ollama_warmup_model', { model });
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

export async function requestApproval(input: CreateApprovalInput): Promise<{ taskId: string; approveUrl: string; denyUrl: string; cancelUrl: string; status: string }> {
  return invoke('request_approval', { input });
}

export async function pollApprovalStatus(input: { taskId: string }): Promise<ApprovalStatus> {
  return invoke('poll_approval_status', input);
}

export async function cancelApproval(input: { taskId: string }): Promise<boolean> {
  return invoke('cancel_approval', input);
}

export async function listPendingApprovals(input?: { agentId?: string }): Promise<ApprovalRequest[]> {
  return invoke('list_pending_approvals', { input: input ?? undefined });
}

export async function addTemplateSource(input: AddTemplateSourceInput): Promise<TemplateSource> {
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

export async function diffPlanVersions(versionIdA: string, versionIdB: string): Promise<PlanDiffResult> {
  return invoke('diff_plan_versions', { versionIdA, versionIdB });
}
