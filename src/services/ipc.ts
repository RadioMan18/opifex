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
  OllamaModel,
  OllamaMessage,
  OllamaChatOutput,
  PlanVersion,
  SaveMessageInput,
  SavePlanVersionInput,
  UpdateAgentInput,
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
