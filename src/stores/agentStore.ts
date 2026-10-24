import { reactive, ref } from 'vue';
import type {
  Agent,
  AgentListItem,
  AgentPlan,
  ApprovalRequest,
  ApprovalStatus,
  Conversation,
  CreateApprovalInput,
  InstalledTemplate,
  InstallTemplateResult,
  Message,
  OllamaModel,
  OllamaMessage,
  PlanDiffResult,
  PlanVersion,
  TemplateEntry,
  TemplateSource,
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
  loadUserAgents as ipcLoadUserAgents,
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
  requestApproval as ipcRequestApproval,
  pollApprovalStatus as ipcPollApprovalStatus,
  cancelApproval as ipcCancelApproval,
  listPendingApprovals as ipcListPendingApprovals,
  addTemplateSource as ipcAddTemplateSource,
  listTemplateSources as ipcListTemplateSources,
  fetchTemplates as ipcFetchTemplates,
  installTemplate as ipcInstallTemplate,
  listInstalledTemplates as ipcListInstalledTemplates,
  diffPlanVersions as ipcDiffPlanVersions,
} from '@/services/ipc';

interface AgentState {
  agents: AgentListItem[];
  currentAgent: Agent | null;
  currentPlan: AgentPlan | null;
  messages: Message[];
  conversations: Conversation[];
  currentConversationId: string | null;
  planVersions: PlanVersion[];
  pendingApprovals: ApprovalRequest[];
  templateSources: TemplateSource[];
  installedTemplates: InstalledTemplate[];
  marketplaceTemplates: TemplateEntry[];
  marketplaceLoading: boolean;
  models: OllamaModel[];
  defaultModel: string | null;
  provider: string | null;
  apiKeys: Record<string, string>;
  loading: boolean;
  streaming: boolean;
  streamingMsgId: string | null;
  interviewing: boolean;
  modelWarmedUp: boolean;
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
  pendingApprovals: [],
  templateSources: [],
  installedTemplates: [],
  marketplaceTemplates: [],
  marketplaceLoading: false,
  models: [],
  defaultModel: null,
  provider: null,
  modelWarmedUp: false,
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
let failedAssistantMessageId: string | null = null;
let streamingContent = '';
let streamCleanup: (() => void) | null = null;

function parseStringArray(value: unknown): (string | { action: string })[] | undefined {
  if (value === undefined) return undefined;
  if (!Array.isArray(value)) {
    throw new Error('Oppy returned an invalid agent plan');
  }
  for (const item of value) {
    if (typeof item !== 'string' && (!item || typeof item !== 'object' || !('action' in item))) {
      throw new Error('Oppy returned an invalid agent plan');
    }
  }
  return value as (string | { action: string })[];
}

function repairRepeatedOutput(content: string): string {
  return content
    // Strip interleaved newline+quote artifacts: "\n"key": → "key":
    .replace(/"\s*\n\s*"/g, '"')
    .replace(/"\s*"/g, '"')
    .replace(/":\s*":/g, '":')
    .replace(/([.,!?])",\s*\1"/g, '$1"')
    .replace(/(['']ve)(?:\1)+/gi, '$1')
    .replace(/\bII(?=[''])/g, 'I')
    // Collapse zero-space adjacent duplicates (e.g. planplan → plan, goalgoal → goal)
    .replace(/\b([\p{L}\p{N}_-]{2,})\1\b/giu, '$1')
    // Collapse space-separated word repetitions (e.g. "plan plan")
    .replace(/\b([\p{L}\p{N}_-]+)(?:\s+\1\b)+/giu, '$1')
    .replace(/\b(\d)\s+\1(\d+)\b/g, '$1$2')
    .replace(/([.,!?])\1+/g, '$1')
    .replace(/([.!?])"\s*\1"/g, '$1"')
    .replace(/_([a-z])_\1/gi, '_$1')
    .replace(/\/([A-Z])\/\1(?=[a-z])/g, '/$1')
    .replace(/\b([A-Z][a-z]+)(?=[A-Z])/g, word =>
      word.replace(/([\p{L}\p{N}_-]{2,})\1+/giu, '$1')
    )
    // Greedy catch-all for any remaining adjacent duplicate substrings (3+ chars)
    .replace(/([\p{L}\p{N}_-]{3,})\1+/giu, '$1');
}

function repairCorruptedFieldNames(content: string): string {
  const fieldNames = ['response', 'plan', 'goal', 'steps', 'tools', 'knowledgeSources', 'triggers', 'outputs'];
  return content.replace(/"([^"]+)"(?=\s*:)/g, (match, rawName: string) => {
    const name = repairRepeatedOutput(rawName).replace(/\s/g, '').toLowerCase();
    const fieldName = fieldNames.find(candidate => {
      const normalized = candidate.toLowerCase();
      return name === normalized || (name.includes(normalized) && name.length <= normalized.length + 4);
    });
    return fieldName ? `"${fieldName}"` : match;
  });
}

function repairDuplicateDelimiters(content: string): string {
  // Pre-pass: collapse interleaved newline+quote artifacts before structural repair
  const prepass = content
    .replace(/"\s*\n\s*"/g, '"')
    .replace(/\}\s*\n\s*\}/g, '}')
    .replace(/\]\s*\n\s*\]/g, ']');
  const normalized = repairCorruptedFieldNames(repairRepeatedOutput(prepass))
    .replace(/^(\s*)\{\s*\{+/, '$1{')
    .replace(/:\s*\{\s*\{+/g, ':{')
    .replace(/:\s*\[\s*\[+/g, ':[')
    .replace(/\[\s*\[/g, '[')
    .replace(/\]\s*\]+(?=\s*[,}])/g, ']')
    .replace(/\[\s*\]\s*\[\s*\]/g, '[]')
    .replace(/,\s*\[\s*\](?=\s*[,}])/g, '')
    .replace(/,\s*,+/g, ',')
    .replace(/,\s*([}\]])/g, '$1')
    .replace(/",\s*",/g, '",')
    .replace(/(?:\s*\}){2,}\s*,\s*"response"\s*:/g, '}, "response":');

  const stack: string[] = [];
  let inString = false;
  let escaped = false;
  const balanced = [...normalized].filter(character => {
    if (inString) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === '"') inString = false;
      return true;
    }
    if (character === '"') {
      inString = true;
      return true;
    }
    if (character === '{' || character === '[') {
      stack.push(character);
      return true;
    }
    if (character === '}' || character === ']') {
      const expected = character === '}' ? '{' : '[';
      if (stack.at(-1) !== expected) return false;
      stack.pop();
    }
    return true;
  }).join('');
  return balanced + stack.reverse().map(open => open === '{' ? '}' : ']').join('');
}

export function useAgentStore() {
  async function init() {
    if (isInitialized.value) return;
    state.loading = true;
    try {
      await ipcSeedStarterAgents();
      await ipcLoadUserAgents();
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

  async function setModelWarmedUp(warmed: boolean) {
    state.modelWarmedUp = warmed;
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
      state.currentPlan = null;

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
    state.currentPlan = null;
    state.interviewing = true;
    state.buildError = null;
    state.rawStreamingContent = null;
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
      state.currentPlan = null;
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
    if (!state.modelWarmedUp) {
      state.error = 'Model is warming up. Please wait a moment and try again.';
      return;
    }

    if (failedAssistantMessageId) {
      state.messages = state.messages.filter(message => message.id !== failedAssistantMessageId);
      failedAssistantMessageId = null;
    }
    cleanupStream();

    if (!state.currentConversationId) {
      try {
        await createNewConversation();
      } catch (err: any) {
        state.error = err?.message || String(err);
        return;
      }
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
    state.buildError = null;

    const provider = state.provider ?? 'ollama';
    const model = state.defaultModel ?? state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages
        .filter(m => m.id !== streamingMessageId && m.id !== failedAssistantMessageId)
        .map(m => ({ role: m.role, content: m.content })),
    ];

    try {
      const cleanup = await ipcOllamaChatStream(
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
            if (!state.interviewing) {
              state.buildError = err;
              failedAssistantMessageId = streamingMessageId;
            }
            cleanupStream();
          },
        },
      );
      streamCleanup = cleanup;
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
      return `You are Oppy, an AI Agent Architect for Opifex. Design a production-grade AI agent specification.

Current agent: "${agentName}"
Current plan (JSON): ${currentPlan}

INTERVIEW MODE — 4 Pillars:
1. GOAL & SCOPE: Precise objective, domain, constraints, time horizons.
2. DETERMINISTIC LOGIC: Scoring formulas, filtering cutoffs, exact criteria.
3. TOOLS vs KNOWLEDGE: Tools = callable actions (web_search, github_api). KnowledgeSources = data stores (github_repos, docs.md).
4. OUTPUT DELIVERABLES: Exact artifacts and schema (markdown table, report, citations).

Rules:
- Ask 1-2 focused clarifying questions per turn.
- Offer recommendations or multiple-choice options.
- Update the "plan" JSON with specific, detailed steps. No vague placeholders.
- Keep "response" conversational and concise.
- When ready, ask: "Should I build this agent now?"

Return ONLY valid JSON:
{
  "response": "conversational reply with 1-2 clarifying questions",
  "plan": {
    "goal": "scoped objective",
    "steps": ["specific actionable steps"],
    "tools": ["tool1", "tool2"],
    "knowledgeSources": ["source1"],
    "triggers": ["trigger1"],
    "outputs": ["output1"]
  }
}
Use empty strings/arrays for undetermined fields.`;
    } else {
      return `You are Oppy. Compile the conversation into a final, production-ready agent specification.

Current agent: "${agentName}"
Current plan (JSON): ${currentPlan}

PLAN MODE:
1. "goal": Clear, precise objective with domain and constraints.
2. "steps": Ordered, actionable steps with explicit criteria and inputs. No vague placeholders.
3. "tools": Callable actions only (web_search, github_api, etc.).
4. "knowledgeSources": Data stores and docs only. No tools listed here.
5. "triggers": Events that invoke the agent.
6. "outputs": Concrete artifacts with schema/format.

Return ONLY valid JSON:
{
  "response": "concise summary of scope, logic, outputs",
  "plan": {
    "goal": "string",
    "steps": ["step 1", "step 2"],
    "tools": ["tool1"],
    "knowledgeSources": ["source1"],
    "triggers": ["trigger1"],
    "outputs": ["output1"]
  }
}
Double-quoted keys, valid JSON only.`;
    }
  }

  async function confirmBuild() {
    if (!state.currentAgent) return;
    cleanupStream();
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
    const model = state.defaultModel ?? state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages
        .filter(m => m.id !== failedAssistantMessageId)
        .map(m => ({ role: m.role, content: m.content })),
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
      const cleanup = await ipcOllamaChatStream(
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
            if (!state.interviewing) {
              state.buildError = err;
              failedAssistantMessageId = streamingMessageId;
            }
            cleanupStream();
          },
        },
      );
      streamCleanup = cleanup;
    } catch (err: any) {
      state.streaming = false;
      state.error = err?.message || String(err);
      cleanupStream();
    }
  }

  async function retryBuild() {
    if (!state.currentAgent) return;
    cleanupStream();
    state.buildError = null;
    state.rawStreamingContent = null;
    state.interviewing = false;

    if (failedAssistantMessageId) {
      state.messages = state.messages.filter(message => message.id !== failedAssistantMessageId);
      failedAssistantMessageId = null;
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
    const model = state.defaultModel ?? state.currentAgent.model;
    const apiKey = state.apiKeys[provider] ?? null;
    const systemPrompt = buildSystemPrompt();

    const ollamaMessages: OllamaMessage[] = [
      { role: 'system', content: systemPrompt },
      ...state.messages
        .filter(m => m.id !== failedAssistantMessageId)
        .map(m => ({ role: m.role, content: m.content })),
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
      const cleanup = await ipcOllamaChatStream(
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
            if (!state.interviewing) {
              state.buildError = err;
              failedAssistantMessageId = streamingMessageId;
            }
            cleanupStream();
          },
        },
      );
      streamCleanup = cleanup;
    } catch (err: any) {
      state.streaming = false;
      state.error = err?.message || String(err);
      cleanupStream();
    }
  }

  function handleStreamEnd(messageId: string) {
    const completedContent = streamingContent;
    state.streaming = false;
    state.streamingMsgId = null;
    cleanupStream();

    const msg = state.messages.find(m => m.id === messageId);
    if (!msg) return;

    // If no content was streamed, show an error instead of trying to parse
    if (!completedContent || completedContent.trim().length === 0) {
      failedAssistantMessageId = messageId;
      state.buildError = "Oppy didn't return a response. Click Retry to try again.";
      return;
    }

    try {
      let cleaned = completedContent.trim()
        .replace(/^```json\s*/i, '')
        .replace(/^```\s*/i, '')
        .replace(/\s*```$/, '')
        .trim();
      const firstBrace = cleaned.indexOf('{');
      const lastBrace = cleaned.lastIndexOf('}');
      if (firstBrace >= 0 && lastBrace > firstBrace) {
        cleaned = cleaned.substring(firstBrace, lastBrace + 1);
      }
      cleaned = cleaned.replace(/,(\s*[}\]])/g, '$1');

      let parsed: unknown;
      try {
        parsed = JSON.parse(cleaned);
      } catch {
        cleaned = repairDuplicateDelimiters(cleaned)
          .replace(/,(\s*[}\]])/g, '$1');
        parsed = JSON.parse(cleaned);
      }
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        throw new Error('Oppy response must be a JSON object');
      }
      const parsedObject = parsed as Record<string, unknown>;
      const wrappedPlan = parsedObject.plan;
      const planValue = wrappedPlan ?? parsedObject;
      if (!planValue || typeof planValue !== 'object' || Array.isArray(planValue)) {
        throw new Error('Oppy response did not include an agent plan');
      }
      const planObject = planValue as Record<string, unknown>;
      if (planObject.goal !== undefined && typeof planObject.goal !== 'string') {
        throw new Error('Oppy returned an invalid agent plan');
      }
      const plan: AgentPlan = {
        goal: typeof planObject.goal === 'string' ? planObject.goal : undefined,
        steps: parseStringArray(planObject.steps)?.map((item) => {
          const action = typeof item === 'string' ? item : item.action;
          return { action, inputs: [], requiresApproval: false, approvalContext: undefined };
        }),
        tools: parseStringArray(planObject.tools)?.map((item) => (typeof item === 'string' ? item : item.action)) ?? [],
        knowledgeSources: parseStringArray(planObject.knowledgeSources)?.map((item) => (typeof item === 'string' ? item : item.action)) ?? [],
        triggers: parseStringArray(planObject.triggers)?.map((item) => (typeof item === 'string' ? item : item.action)) ?? [],
        outputs: parseStringArray(planObject.outputs)?.map((item) => (typeof item === 'string' ? item : item.action)) ?? [],
      };
      const response = typeof parsedObject.response === 'string'
        ? parsedObject.response
        : 'I updated the agent plan based on our conversation.';
      msg.content = response;
      msg.turnPlanJson = plan;
      state.currentPlan = plan;
      state.buildError = null;
      failedAssistantMessageId = null;

      if (state.currentConversationId) {
        ipcSaveMessage({
          conversationId: state.currentConversationId,
          role: 'assistant',
          content: msg.content,
          turnPlanJson: plan,
        }).catch((err: unknown) => {
          state.error = err instanceof Error ? err.message : String(err);
        });
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
        }).catch((err: unknown) => {
          state.error = err instanceof Error ? err.message : String(err);
        });
      }
    } catch (error: unknown) {
      failedAssistantMessageId = messageId;
      const detail = error instanceof Error ? ` ${error.message}` : '';
      state.buildError = `Oppy returned a response but it wasn't valid JSON.${detail} Try qwen3-coder:30b or llama3.2 for better JSON output. You can retry or continue the interview.`;
      state.rawStreamingContent = completedContent;
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
    if (streamCleanup) {
      try {
        streamCleanup();
      } catch {}
      streamCleanup = null;
    }
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
    } else if (section === 'steps') {
      const stepStrings = Array.isArray(value)
        ? value.filter(Boolean)
        : value.split(',').map(s => s.trim()).filter(Boolean);
      newPlan.steps = stepStrings.map((action) => ({
        action,
        inputs: [],
        requiresApproval: false,
        approvalContext: undefined,
      }));
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

  async function requestApproval(input: CreateApprovalInput): Promise<{ taskId: string; approveUrl: string; denyUrl: string; cancelUrl: string; status: string } | null> {
    try {
      const result = await ipcRequestApproval(input);
      const approval: ApprovalRequest = {
        id: result.taskId,
        agentId: input.agentId,
        taskId: result.taskId,
        command: input.command,
        context: input.context,
        status: 'pending',
        response: null,
        requestedAt: new Date().toISOString(),
        respondedAt: null,
      };
      state.pendingApprovals = [approval, ...state.pendingApprovals];
      return result;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function pollApprovalStatus(input: { taskId: string }): Promise<ApprovalStatus | null> {
    try {
      const status = await ipcPollApprovalStatus(input);
      state.pendingApprovals = state.pendingApprovals.map(a =>
        a.taskId === input.taskId
          ? { ...a, status: status.status as any, response: status.response ?? null, respondedAt: new Date().toISOString() }
          : a
      );
      return status;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function cancelApproval(input: { taskId: string }): Promise<boolean> {
    try {
      const cancelled = await ipcCancelApproval(input);
      if (cancelled) {
        state.pendingApprovals = state.pendingApprovals.map(a =>
          a.taskId === input.taskId
            ? { ...a, status: 'cancelled', response: 'cancelled', respondedAt: new Date().toISOString() }
            : a
        );
      }
      return cancelled;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return false;
    }
  }

  async function listPendingApprovals(agentId?: string): Promise<ApprovalRequest[]> {
    try {
      const approvals = await ipcListPendingApprovals(agentId ? { agentId } : undefined);
      state.pendingApprovals = approvals;
      return approvals;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return [];
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

  async function addTemplateSource(name: string, url: string): Promise<TemplateSource | null> {
    try {
      const source = await ipcAddTemplateSource({ name, url });
      state.templateSources = [source, ...state.templateSources];
      return source;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function loadTemplateSources(): Promise<TemplateSource[]> {
    try {
      const sources = await ipcListTemplateSources();
      state.templateSources = sources;
      return sources;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return [];
    }
  }

  async function loadMarketplaceTemplates(sourceId: string): Promise<TemplateEntry[]> {
    state.marketplaceLoading = true;
    state.error = null;
    try {
      const templates = await ipcFetchTemplates(sourceId);
      state.marketplaceTemplates = templates;
      return templates;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return [];
    } finally {
      state.marketplaceLoading = false;
    }
  }

  async function installMarketplaceTemplate(
    templateId: string,
    version: string,
    sourceId: string,
  ): Promise<InstallTemplateResult | null> {
    try {
      const result = await ipcInstallTemplate({ templateId, version, sourceId });
      state.installedTemplates = [
        { id: result.agentId, templateId: result.templateId, version: result.version, installedAt: new Date().toISOString() },
        ...state.installedTemplates,
      ];
      await loadAgents();
      return result;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
    }
  }

  async function loadInstalledTemplates(): Promise<InstalledTemplate[]> {
    try {
      const templates = await ipcListInstalledTemplates();
      state.installedTemplates = templates;
      return templates;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return [];
    }
  }

  async function diffPlanVersions(versionIdA: string, versionIdB: string): Promise<PlanDiffResult | null> {
    try {
      const result = await ipcDiffPlanVersions(versionIdA, versionIdB);
      return result;
    } catch (err: any) {
      state.error = err?.message || String(err);
      return null;
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
    requestApproval,
    pollApprovalStatus,
    cancelApproval,
    listPendingApprovals,
    addTemplateSource,
    loadTemplateSources,
    loadMarketplaceTemplates,
    installMarketplaceTemplate,
    loadInstalledTemplates,
    diffPlanVersions,
    setModelWarmedUp,
  };
}
