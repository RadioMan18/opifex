import { reactive, ref } from 'vue';
import type {
  Agent,
  AgentListItem,
  AgentPlan,
  Conversation,
  Message,
  OllamaModel,
  OllamaMessage,
  PlanVersion,
} from '@/types/agent';
import {
  createAgent as ipcCreateAgent,
  getAgent as ipcGetAgent,
  listAgents as ipcListAgents,
  updateAgent as ipcUpdateAgent,
  deleteAgent as ipcDeleteAgent,
  exportAgent as ipcExportAgent,
  importAgent as ipcImportAgent,
  renameAgent as ipcRenameAgent,
  duplicateAgent as ipcDuplicateAgent,
  seedStarterAgents as ipcSeedStarterAgents,
  ollamaListModels as ipcOllamaListModels,
  ollamaPullModel as ipcOllamaPullModel,
  ollamaChatStream as ipcOllamaChatStream,
  ollamaGetDefaultModel as ipcOllamaGetDefaultModel,
  ollamaSetDefaultModel as ipcOllamaSetDefaultModel,
  ollamaGetProvider as ipcOllamaGetProvider,
  ollamaSetProvider as ipcOllamaSetProvider,
  ollamaGetApiKey as ipcOllamaGetApiKey,
  ollamaSetApiKey as ipcOllamaSetApiKey,
  createConversation as ipcCreateConversation,
  listConversations as ipcListConversations,
  saveMessage as ipcSaveMessage,
  savePlanVersion as ipcSavePlanVersion,
  listPlanVersions as ipcListPlanVersions,
  restorePlanVersion as ipcRestorePlanVersion,
} from '@/services/ipc';

interface AgentState {
  agents: AgentListItem[];
  currentAgent: Agent | null;
  currentPlan: AgentPlan | null;
  messages: Message[];
  conversations: Conversation[];
  currentConversationId: string | null;
  planVersions: PlanVersion[];
  models: OllamaModel[];
  defaultModel: string | null;
  provider: string | null;
  apiKeys: Record<string, string>;
  loading: boolean;
  streaming: boolean;
  streamingMsgId: string | null;
  interviewing: boolean;
  buildError: string | null;
  rawStreamingContent: string | null;
  error: string | null;
}

const state = reactive<AgentState>({
  agents: [],
  currentAgent: null,
  currentPlan: null,
  messages: [],
  conversations: [],
  currentConversationId: null,
  planVersions: [],
  models: [],
  defaultModel: null,
  provider: null,
  apiKeys: {},
  loading: false,
  streaming: false,
  streamingMsgId: null,
  interviewing: true,
  buildError: null,
  rawStreamingContent: null,
  error: null,
});

const isInitialized = ref(false);

let streamingMessageId: string | null = null;
let streamingContent = '';
let streamUnlistens: (() => void)[] = [];

export function useAgentStore() {
  async function init() {
    if (isInitialized.value) return;
    state.loading = true;
    try {
      await ipcSeedStarterAgents();
      await loadAgents();
      await loadModels();
      await loadDefaultModel();
      await loadProvider();
      isInitialized.value = true;
    } catch (err: any) {
      state.error = err?.message || String(err);
    } finally {
      state.loading = false;
    }
  }

  async function loadAgents() {
    try {
      state.agents = await ipcListAgents(true);
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function loadModels() {
    try {
      state.models = await ipcOllamaListModels();
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function loadDefaultModel() {
    try {
      state.defaultModel = await ipcOllamaGetDefaultModel();
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function setDefaultModel(model: string) {
    try {
      await ipcOllamaSetDefaultModel(model);
      state.defaultModel = model;
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function loadProvider() {
    try {
      state.provider = await ipcOllamaGetProvider();
      if (!state.provider) {
        await ipcOllamaSetProvider('ollama');
        state.provider = 'ollama';
      }
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function setProvider(provider: string) {
    try {
      await ipcOllamaSetProvider(provider);
      state.provider = provider;
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function getApiKey(provider: string): Promise<string | null> {
    try {
      return await ipcOllamaGetApiKey(provider);
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function setApiKey(provider: string, apiKey: string) {
    try {
      await ipcOllamaSetApiKey(provider, apiKey);
      state.apiKeys[provider] = apiKey;
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function pullModel(modelName: string) {
    state.loading = true;
    try {
      await ipcOllamaPullModel(modelName);
      await loadModels();
    } catch (err: any) {
      state.error = err?.message || String(err);
    } finally {
      state.loading = false;
    }
  }

  async function selectAgent(id: string) {
    cleanupStream();
    try {
      state.currentAgent = await ipcGetAgent(id);
      state.currentPlan = state.currentAgent?.specJson ?? null;

      // Always start fresh — don't load old conversations automatically.
      // Conversation history can be loaded explicitly if the user chooses a previous conversation.
      state.conversations = await ipcListConversations(id);
      state.currentConversationId = null;
      state.messages = [];
      state.planVersions = await ipcListPlanVersions(id);
      state.interviewing = true;
      state.buildError = null;
      state.rawStreamingContent = null;
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function createNewConversation(title?: string): Promise<string> {
    if (!state.currentAgent) {
      throw new Error('No agent selected');
    }
    const conv = await ipcCreateConversation({
      agentId: state.currentAgent.id,
      title: title ?? `Conversation with ${state.currentAgent.name}`,
    });
    state.currentConversationId = conv.id;
    state.conversations = [conv, ...state.conversations.filter(c => c.id !== conv.id)];
    state.messages = [];
    return conv.id;
  }

  async function createNewAgent(
    name: string,
    description: string | null,
    model?: string,
  ) {
    state.loading = true;
    cleanupStream();
    try {
      const agent = await ipcCreateAgent({
        name,
        description,
        specJson: {
          goal: '',
          steps: [],
          tools: [],
          knowledgeSources: [],
          triggers: [],
          outputs: [],
        },
        model: model ?? state.defaultModel ?? 'ollama',
        isStarter: false,
      });
      state.currentAgent = agent;
      state.currentPlan = agent.specJson;
      state.messages = [];
      state.conversations = [];
      state.currentConversationId = null;
      state.planVersions = [];
      state.interviewing = true;
      state.buildError = null;
      state.rawStreamingContent = null;
      await loadAgents();
      return agent;
    } catch (err: any) {
      state.error = err?.message || String(err);
    } finally {
      state.loading = false;
    }
  }

  async function exportAgent(id: string): Promise<string> {
    try {
      return await ipcExportAgent(id);
    } catch (err: any) {
      state.error = err?.message || String(err);
      throw err;
    }
  }

  async function importAgent(agentJson: string): Promise<Agent | null> {
    try {
      const agent = await ipcImportAgent(agentJson);
      await loadAgents();
      return agent;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function renameAgent(id: string, newName: string): Promise<Agent | null> {
    try {
      const agent = await ipcRenameAgent(id, newName);
      await loadAgents();
      if (state.currentAgent?.id === id) {
        state.currentAgent = agent;
      }
      return agent;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function duplicateAgent(id: string, newName?: string): Promise<Agent | null> {
    try {
      const agent = await ipcDuplicateAgent(id, newName);
      await loadAgents();
      return agent;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function sendMessage(content: string) {
    if (!state.currentAgent) {
      state.error = 'No agent selected';
      return;
    }
    if (!content.trim()) return;

    cleanupStream();

    if (!state.currentConversationId) {
      await createNewConversation();
    }

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: content.trim(),
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(userMessage);

    try {
      await ipcSaveMessage({
        conversationId: state.currentConversationId!,
        role: 'user',
        content: userMessage.content,
        turnPlanJson: null,
      });
    } catch (err: any) {
      state.error = err?.message || String(err);
    }

    streamingMessageId = crypto.randomUUID();
    state.streamingMsgId = streamingMessageId;
    streamingContent = '';
    const assistantMessage: Message = {
      id: streamingMessageId,
      role: 'assistant',
      content: '',
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(assistantMessage);

    state.streaming = true;
    state.error = null;

    const provider = state.provider ?? 'ollama';
    const model = state.defaultModel ?? state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages
        .filter(m => m.id !== streamingMessageId)
        .map(m => ({ role: m.role, content: m.content })),
    ];

    try {
      const unlistens = await ipcOllamaChatStream(
        provider,
        model,
        apiKey,
        ollamaMessages,
        {
          onChunk: (chunk: string) => {
            streamingContent += chunk;
            const msg = state.messages.find(m => m.id === streamingMessageId);
            if (msg) {
              msg.content = streamingContent;
            }
          },
          onEnd: () => {
            handleStreamEnd(streamingMessageId!);
          },
          onError: (err: string) => {
            state.streaming = false;
            state.error = err;
            cleanupStream();
          },
        },
      );
      streamUnlistens = unlistens;
    } catch (err: any) {
      state.streaming = false;
      state.error = err?.message || String(err);
      cleanupStream();
    }
  }

  function buildSystemPrompt(): string {
    const agentName = state.currentAgent?.name || 'agent';
    const currentPlan = state.currentPlan
      ? JSON.stringify(state.currentPlan, null, 2)
      : '{}';

    if (state.interviewing) {
      return `You are Oppy, an expert AI agent builder for Opifex.

Current agent: "${agentName}"
Current plan (JSON): ${currentPlan}

INTERVIEW MODE: You are interviewing the user to understand what they want their agent to do.

Your job:
1. Have a natural conversation and ask clarifying questions about the agent's goal, steps, tools, knowledge sources, triggers, and outputs
2. Ask questions one at a time or in small related batches — don't overwhelm the user
3. Be helpful, concise, and professional
4. Do NOT produce any JSON plan yet — we are still in the interview phase
5. When you have enough information to build a complete plan, ask: "Should I build this agent now?"
6. Wait for the user to confirm before switching to plan generation mode

The user sees your messages on the left side of the screen. The plan panel on the right will remain empty until the interview is complete.`;
    } else {
      return `You are Oppy, an expert AI agent builder for Opifex. You help users design AI agents for business use through conversation.

Current agent: "${agentName}"
Current plan (JSON):
${currentPlan}

PLAN MODE: The interview is complete. Your job is to produce and refine a structured agent plan.

Your job:
1. Produce a COMPLETE updated plan as a JSON object in this exact format:
{
  "goal": "string - what the agent accomplishes",
  "steps": ["step 1", "step 2", "step 3"],
  "tools": ["tool1", "tool2"],
  "knowledgeSources": ["source1.md", "source2.pdf"],
  "triggers": ["trigger1", "trigger2"],
  "outputs": ["output1", "output2"]
}

 2. Return ONLY a raw JSON object — no markdown code blocks, no explanation, no prose before or after
3. Update the plan incrementally based on the conversation — don't repeat the same plan unless nothing changed
4. Be helpful, concise, and professional
5. Ensure valid JSON syntax — double-quoted keys and string values only

The user sees the plan on the right side of the screen updating in real-time. Make each turn count.`;
    }
  }

  async function confirmBuild() {
    if (!state.currentAgent) return;
    state.interviewing = false;
    state.buildError = null;
    state.rawStreamingContent = null;

    const buildMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: 'Please generate the complete agent plan now based on our conversation.',
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(buildMessage);

    try {
      await ipcSaveMessage({
        conversationId: state.currentConversationId!,
        role: 'user',
        content: buildMessage.content,
        turnPlanJson: null,
      });
    } catch (err: any) {
      state.error = err?.message || String(err);
    }

    const provider = state.provider ?? 'ollama';
    const model = state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages.map(m => ({ role: m.role, content: m.content })),
    ];

    state.streaming = true;
    state.error = null;

    streamingMessageId = crypto.randomUUID();
    state.streamingMsgId = streamingMessageId;
    streamingContent = '';
    const assistantMessage: Message = {
      id: streamingMessageId,
      role: 'assistant',
      content: '',
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(assistantMessage);

    try {
      const unlistens = await ipcOllamaChatStream(
        provider,
        model,
        apiKey,
        ollamaMessages,
        {
          onChunk: (chunk: string) => {
            streamingContent += chunk;
            const msg = state.messages.find(m => m.id === streamingMessageId);
            if (msg) {
              msg.content = streamingContent;
            }
          },
          onEnd: () => {
            handleStreamEnd(streamingMessageId!);
          },
          onError: (err: string) => {
            state.streaming = false;
            state.error = err;
            cleanupStream();
          },
        },
      );
      streamUnlistens = unlistens;
    } catch (err: any) {
      state.streaming = false;
      state.error = err?.message || String(err);
      cleanupStream();
    }
  }

  async function retryBuild() {
    if (!state.currentAgent) return;
    state.buildError = null;
    state.rawStreamingContent = null;
    state.interviewing = false;

    // Remove the last failed assistant message if it exists
    const lastMsg = state.messages[state.messages.length - 1];
    if (lastMsg && lastMsg.role === 'assistant' && lastMsg.id === streamingMessageId) {
      state.messages = state.messages.slice(0, -1);
    }

    const buildMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: 'Please generate the complete agent plan now based on our conversation.',
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(buildMessage);

    try {
      await ipcSaveMessage({
        conversationId: state.currentConversationId!,
        role: 'user',
        content: buildMessage.content,
        turnPlanJson: null,
      });
    } catch (err: any) {
      state.error = err?.message || String(err);
    }

    const provider = state.provider ?? 'ollama';
    const model = state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages.map(m => ({ role: m.role, content: m.content })),
    ];

    state.streaming = true;
    state.error = null;

    streamingMessageId = crypto.randomUUID();
    state.streamingMsgId = streamingMessageId;
    streamingContent = '';
    const assistantMessage: Message = {
      id: streamingMessageId,
      role: 'assistant',
      content: '',
      turnPlanJson: null,
      createdAt: new Date().toISOString(),
    };
    state.messages.push(assistantMessage);

    try {
      const unlistens = await ipcOllamaChatStream(
        provider,
        model,
        apiKey,
        ollamaMessages,
        {
          onChunk: (chunk: string) => {
            streamingContent += chunk;
            const msg = state.messages.find(m => m.id === streamingMessageId);
            if (msg) {
              msg.content = streamingContent;
            }
          },
          onEnd: () => {
            handleStreamEnd(streamingMessageId!);
          },
          onError: (err: string) => {
            state.streaming = false;
            state.error = err;
            cleanupStream();
          },
        },
      );
      streamUnlistens = unlistens;
    } catch (err: any) {
      state.streaming = false;
      state.error = err?.message || String(err);
      cleanupStream();
    }
  }

  function handleStreamEnd(messageId: string) {
    state.streaming = false;
    state.streamingMsgId = null;
    cleanupStream();

    const msg = state.messages.find(m => m.id === messageId);
    if (!msg) return;

    // If no content was streamed, show an error instead of trying to parse
    if (!streamingContent || streamingContent.trim().length === 0) {
      state.buildError = "Oppy didn't return a response. Click Retry to try again.";
      return;
    }

    try {
      // Strip markdown code block fences if present
      let cleaned = streamingContent.trim();
      // Remove ```json ... ``` or ``` ... ``` wrappers
      cleaned = cleaned.replace(/^```json\s*/i, '').replace(/^```\s*/i, '').replace(/\s*```$/, '').trim();

      // Try to find JSON object within the text (between first { and last })
      const firstBrace = cleaned.indexOf('{');
      const lastBrace = cleaned.lastIndexOf('}');
      if (firstBrace >= 0 && lastBrace > firstBrace) {
        cleaned = cleaned.substring(firstBrace, lastBrace + 1);
      }

      // Remove any trailing commas before } or ] (common LLM mistake)
      cleaned = cleaned.replace(/,(\s*[}\]])/g, '$1');

      const plan = JSON.parse(cleaned) as AgentPlan;
      msg.turnPlanJson = plan;
      state.currentPlan = plan;

      if (state.currentConversationId) {
        ipcSaveMessage({
          conversationId: state.currentConversationId,
          role: 'assistant',
          content: msg.content,
          turnPlanJson: plan,
        }).catch(() => {});
      }

      if (state.currentAgent) {
        ipcUpdateAgent(state.currentAgent.id, {
          name: null,
          description: null,
          specJson: plan,
          model: null,
          isStarter: null,
        }).then(updated => {
          state.currentAgent = {
            ...updated,
            specJson: plan,
            updatedAt: new Date().toISOString(),
          };
        }).catch(() => {});
      }
    } catch {
      state.buildError = "Oppy returned a response but it wasn't valid JSON. Try qwen3-coder:30b or llama3.2 for better JSON output. You can retry or continue the interview.";
      state.rawStreamingContent = streamingContent;
      if (state.currentConversationId) {
        ipcSaveMessage({
          conversationId: state.currentConversationId,
          role: 'assistant',
          content: msg.content,
          turnPlanJson: null,
        }).catch(() => {});
      }
    }
  }

  function cleanupStream() {
    streamUnlistens.forEach(unlisten => unlisten());
    streamUnlistens = [];
    streamingMessageId = null;
    streamingContent = '';
    state.streaming = false;
    state.streamingMsgId = null;
  }

  async function updatePlanField(
    section: keyof AgentPlan,
    value: string | string[],
  ) {
    if (!state.currentAgent || !state.currentPlan) return;

    const newPlan = { ...state.currentPlan };
    if (section === 'goal') {
      newPlan.goal = value as string;
    } else {
      newPlan[section] = Array.isArray(value)
        ? value.filter(Boolean)
        : value.split(',').map(s => s.trim()).filter(Boolean);
    }

    state.currentPlan = newPlan;

    try {
      await ipcUpdateAgent(state.currentAgent.id, {
        name: null,
        description: null,
        specJson: newPlan,
        model: null,
        isStarter: null,
      });
      if (state.currentAgent) {
        state.currentAgent = {
          ...state.currentAgent,
          specJson: newPlan,
          updatedAt: new Date().toISOString(),
        };
      }
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function savePlanVersion(label?: string): Promise<PlanVersion | null> {
    if (!state.currentAgent || !state.currentPlan) return null;
    try {
      const version = await ipcSavePlanVersion({
        agentId: state.currentAgent.id,
        label: label ?? null,
        specJson: state.currentPlan,
      });
      state.planVersions = [version, ...state.planVersions];
      return version;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function loadPlanVersions(agentId: string) {
    try {
      state.planVersions = await ipcListPlanVersions(agentId);
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  async function restorePlanVersion(versionId: string): Promise<boolean> {
    try {
      const plan = await ipcRestorePlanVersion(versionId);
      state.currentPlan = plan;
      if (state.currentAgent) {
        state.currentAgent = {
          ...state.currentAgent,
          specJson: plan,
          updatedAt: new Date().toISOString(),
        };
      }
      return true;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return false;
    }
  }

  async function deleteCurrentAgent() {
    if (!state.currentAgent) return;
    cleanupStream();
    try {
      await ipcDeleteAgent(state.currentAgent.id);
      state.currentAgent = null;
      state.currentPlan = null;
      state.messages = [];
      state.conversations = [];
      state.currentConversationId = null;
      state.planVersions = [];
      await loadAgents();
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }

  return {
    state,
    init,
    loadAgents,
    loadModels,
    loadDefaultModel,
    loadProvider,
    setProvider,
    setDefaultModel,
    getApiKey,
    setApiKey,
    pullModel,
    selectAgent,
    createNewAgent,
    createNewConversation,
    sendMessage,
    confirmBuild,
    retryBuild,
    updatePlanField,
    savePlanVersion,
    loadPlanVersions,
    restorePlanVersion,
    exportAgent,
    importAgent,
    renameAgent,
    duplicateAgent,
    deleteCurrentAgent,
  };
}
