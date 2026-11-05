<script setup lang="ts">
import { onMounted, ref, computed, onBeforeUnmount } from 'vue';
import { save as tauriSaveDialog, open as tauriOpenDialog } from '@tauri-apps/plugin-dialog';
import { readFile as tauriReadFile } from '@tauri-apps/plugin-fs';
import { useAgentStore } from '@/stores/agentStore';
import { useGraphStore } from '@/stores/graphStore';
import ChatPane from '@/components/ChatPane.vue';
import PlanPane from '@/components/PlanPane.vue';
import AgentLibrary from '@/components/AgentLibrary.vue';
import ModelSelector from '@/components/ModelSelector.vue';
import AgentCanvas from '@/components/AgentCanvas.vue';
import BlockPalette from '@/components/BlockPalette.vue';
import NodeInspector from '@/components/NodeInspector.vue';
import AgentToolDrawer from '@/components/inspector/AgentToolDrawer.vue';
import PinPanel from '@/components/PinPanel.vue';
import SkillEditor from '@/components/inspector/SkillEditor.vue';
import KnowledgeSourceEditor from '@/components/inspector/KnowledgeSourceEditor.vue';
import TriggerEditor from '@/components/inspector/TriggerEditor.vue';
import { writeFile as ipcWriteFile } from '@/services/ipc';
import type { AgentPlan, InstalledTemplate, TemplateSource, BlockDefinition } from '@/types/agent';

const store = useAgentStore();
const graphStore = useGraphStore();
const {
  state,
  init,
  selectAgent,
  createNewAgent,
  createNewConversation,
  sendMessage,
  confirmBuild,
  retryBuild,
  updatePlanField,
  deleteCurrentAgent,
  pullModel,
  setDefaultModel,
  setApiKey,
  savePlanVersion,
  loadPlanVersions,
  restorePlanVersion,
  exportAgent,
  importAgent,
  renameAgent,
  duplicateAgent,
  addTemplateSource,
  loadTemplateSources,
  fetchMarketplaceTemplates,
  installMarketplaceTemplate,
  loadInstalledTemplates,
} = store;

const newAgentName = ref('');
const newAgentDescription = ref('');
const showCreateDialog = ref(false);
const showImportDialog = ref(false);
const showRenameDialog = ref(false);
const renameId = ref<string | null>(null);
const renameName = ref('');
const showProviderDialog = ref(false);
const newVersionLabel = ref('');
const darkMode = ref(localStorage.getItem('opifex-theme') === 'dark');
const viewMode = ref<'canvas' | 'oppy'>('canvas');
const showUtilityMenu = ref(false);
const utilityMenuRef = ref<HTMLElement | null>(null);

function toggleUtilityMenu() {
  showUtilityMenu.value = !showUtilityMenu.value;
}

function closeUtilityMenu() {
  showUtilityMenu.value = false;
}

function handleUtilityAction(action: () => void) {
  action();
  closeUtilityMenu();
}

function handleClickOutside(e: MouseEvent) {
  if (utilityMenuRef.value && !utilityMenuRef.value.contains(e.target as Node)) {
    closeUtilityMenu();
  }
}

const defaultBlocks = ref<BlockDefinition[]>([
  {
    id: 'builtin:input',
    version: 1,
    name: 'Entry Point',
    description: 'Entry point trigger — every graph requires exactly one.',
    category: 'Triggers',
    kind: 'trigger',
    origin: 'user',
    inputs: [],
    outputs: [
      { id: 'message', name: 'Message', type: 'string', direction: 'output', required: false },
      { id: 'payload', name: 'Payload', type: 'json', direction: 'output', required: false },
    ],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:llm-prompt',
    version: 1,
    name: 'Prompt Weaver',
    description: 'Send a message to the LLM provider and return completion.',
    category: 'LLM',
    kind: 'llm',
    origin: 'user',
    inputs: [
      { id: 'message', name: 'Message', type: 'string', direction: 'input', required: true },
      { id: 'system', name: 'System', type: 'string', direction: 'input', required: false },
    ],
    outputs: [{ id: 'completion', name: 'Completion', type: 'string', direction: 'output', required: false }],
    parameters: [
      { id: 'model', name: 'model', type: 'string', label: 'Model', required: false, defaultValue: 'llama3.2', sensitive: false },
      { id: 'temperature', name: 'temperature', type: 'number', label: 'Temperature', required: false, defaultValue: 0.7, sensitive: false },
    ],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:agent',
    version: 1,
    name: 'Task Orchestrator',
    description: 'Autonomous agent that can invoke attached tools.',
    category: 'Agents',
    kind: 'agent',
    origin: 'user',
    inputs: [
      { id: 'prompt', name: 'Prompt', type: 'string', direction: 'input', required: true },
      { id: 'tools', name: 'Tools', type: 'tool', direction: 'input', required: false },
    ],
    outputs: [
      { id: 'answer', name: 'Answer', type: 'string', direction: 'output', required: false },
      { id: 'trace', name: 'Trace', type: 'json', direction: 'output', required: false },
    ],
    parameters: [
      { id: 'max_iterations', name: 'max_iterations', type: 'number', label: 'Max Iterations', required: false, defaultValue: 8, sensitive: false },
    ],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:http-request',
    version: 1,
    name: 'Web Courier',
    description: 'Generic HTTP request tool for agents.',
    category: 'Tools',
    kind: 'tool',
    origin: 'user',
    inputs: [
      { id: 'url', name: 'URL', type: 'string', direction: 'input', required: true },
      { id: 'body', name: 'Body', type: 'any', direction: 'input', required: false },
      { id: 'headers', name: 'Headers', type: 'object', direction: 'input', required: false },
    ],
    outputs: [
      { id: 'response', name: 'Response', type: 'json', direction: 'output', required: false },
      { id: 'tool', name: 'Tool', type: 'tool', direction: 'output', required: false },
    ],
    parameters: [
      { id: 'method', name: 'method', type: 'select', label: 'Method', required: false, defaultValue: 'GET', options: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'], sensitive: false },
    ],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:condition',
    version: 1,
    name: 'Logic Gate',
    description: 'Branch execution based on a boolean condition.',
    category: 'Control Flow',
    kind: 'control',
    origin: 'user',
    inputs: [{ id: 'value', name: 'Value', type: 'any', direction: 'input', required: true }],
    outputs: [
      { id: 'true', name: 'True', type: 'any', direction: 'output', required: false },
      { id: 'false', name: 'False', type: 'any', direction: 'output', required: false },
    ],
    parameters: [
      { id: 'condition', name: 'condition', type: 'string', label: 'Condition', required: false, defaultValue: '{{ input.count > 5 }}', sensitive: false },
    ],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:merge',
    version: 1,
    name: 'Stream Merger',
    description: 'Merge multiple inputs into a single output.',
    category: 'Control Flow',
    kind: 'control',
    origin: 'user',
    inputs: [{ id: 'inputs', name: 'Inputs', type: 'any', direction: 'input', required: false }],
    outputs: [{ id: 'merged', name: 'Merged', type: 'any', direction: 'output', required: false }],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:code-js',
    version: 1,
    name: 'Script Vault',
    description: 'Sandboxed JavaScript execution (Developer mode).',
    category: 'Utilities',
    kind: 'utility',
    origin: 'user',
    inputs: [{ id: 'input', name: 'Input', type: 'any', direction: 'input', required: true }],
    outputs: [
      { id: 'output', name: 'Output', type: 'any', direction: 'output', required: false },
      { id: 'tool', name: 'Tool', type: 'tool', direction: 'output', required: false },
    ],
    parameters: [
      { id: 'code', name: 'code', type: 'code', label: 'Code', required: false, sensitive: false },
    ],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'builtin:final-answer',
    version: 1,
    name: 'Result Terminal',
    description: 'Terminal output block — returns result to user.',
    category: 'Outputs',
    kind: 'output',
    origin: 'user',
    inputs: [{ id: 'message', name: 'Message', type: 'string', direction: 'input', required: true }],
    outputs: [],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
]);

const selectedCanvasNodeId = computed(() => graphStore.selectedNodeId ?? undefined);

// One rail, one place to look: the canvas owns the workspace, and everything
// that configures a node or an agent lives in a tab here instead of stacking
// panels underneath the canvas.
type InspectorTab = 'node' | 'tools' | 'pins' | 'skills' | 'knowledge' | 'triggers';
const inspectorTabs: Array<{ id: InspectorTab; label: string }> = [
  { id: 'node', label: 'Node' },
  { id: 'tools', label: 'Tools' },
  { id: 'pins', label: 'Pins' },
  { id: 'skills', label: 'Skills' },
  { id: 'knowledge', label: 'Knowledge' },
  { id: 'triggers', label: 'Triggers' },
];
const inspectorTab = ref<InspectorTab>('node');

// Both rails can be dismissed so the canvas can use the full width.
const showPalette = ref(true);
const showInspector = ref(true);

onMounted(async () => {
  init();
  applyDarkMode();
  setupKeyboardShortcuts();
  // The canvas resolves node labels/ports from the store, so the palette's
  // block definitions must be registered there before a drop can render one.
  graphStore.blocks = defaultBlocks.value;
  // Dropping a block is a no-op against a null graph; ensure one exists.
  if (!graphStore.graph) {
    graphStore.createGraph('Untitled Graph');
  }
  await loadTemplateSources();
  await loadInstalledTemplates();
  document.addEventListener('click', handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside);
});

function toggleDarkMode() {
  darkMode.value = !darkMode.value;
  applyDarkMode();
}

function applyDarkMode() {
  const isDark = darkMode.value;
  const root = document.documentElement;
  root.setAttribute('data-theme', isDark ? 'dark' : 'light');

  if (isDark) {
    root.style.setProperty('--surface-base', '#0F172A');
    root.style.setProperty('--surface-card', '#1E293B');
    root.style.setProperty('--surface-hover', '#334155');
    root.style.setProperty('--surface-muted', '#0F172A');
    root.style.setProperty('--text-primary', '#F8FAFC');
    root.style.setProperty('--text-secondary', '#CBD5E1');
    root.style.setProperty('--text-muted', '#D1D5DB');
    root.style.setProperty('--border', '#475569');
    root.style.setProperty('--border-focus', '#0EA5E9');
    root.style.setProperty('--border-hover', '#64748B');
    root.style.setProperty('--primary', '#0EA5E9');
    root.style.setProperty('--primary-hover', '#0284C7');
    root.style.setProperty('--primary-fg', '#FFFFFF');
    root.style.setProperty('--primary-bg', 'rgba(14, 165, 233, 0.15)');
    root.style.setProperty('--success', '#10B981');
    root.style.setProperty('--success-fg', '#FFFFFF');
    root.style.setProperty('--success-bg', 'rgba(16, 185, 129, 0.15)');
    root.style.setProperty('--danger', '#EF4444');
    root.style.setProperty('--danger-fg', '#FFFFFF');
    root.style.setProperty('--danger-bg', 'rgba(239, 68, 68, 0.15)');
    root.style.setProperty('--warning', '#EAB308');
    root.style.setProperty('--warning-fg', '#FFFFFF');
    root.style.setProperty('--warning-bg', 'rgba(234, 179, 8, 0.15)');
    root.style.setProperty('--shadow-sm', '0 1px 2px rgba(0, 0, 0, 0.4)');
    root.style.setProperty('--shadow-md', '0 4px 6px -1px rgb(0 0 0 / 0.5), 0 2px 4px -2px rgb(0 0 0 / 0.5)');
    root.style.setProperty('--shadow-lg', '0 10px 15px -3px rgb(0 0 0 / 0.6), 0 4px 6px -4px rgb(0 0 0 / 0.6)');
  } else {
    root.style.setProperty('--surface-base', '#F8FAFC');
    root.style.setProperty('--surface-card', '#FFFFFF');
    root.style.setProperty('--surface-hover', '#E2E8F0');
    root.style.setProperty('--surface-muted', '#F1F5F9');
    root.style.setProperty('--text-primary', '#0F172A');
    root.style.setProperty('--text-secondary', '#334155');
    root.style.setProperty('--text-muted', '#475569');
    root.style.setProperty('--border', '#CBD5E1');
    root.style.setProperty('--border-focus', '#0B63D9');
    root.style.setProperty('--border-hover', '#94A3B8');
    root.style.setProperty('--primary', '#0B63D9');
    root.style.setProperty('--primary-hover', '#084FB3');
    root.style.setProperty('--primary-fg', '#FFFFFF');
    root.style.setProperty('--primary-bg', 'rgba(11, 99, 217, 0.12)');
    root.style.setProperty('--success', '#0F8F5F');
    root.style.setProperty('--success-fg', '#FFFFFF');
    root.style.setProperty('--success-bg', 'rgba(15, 143, 95, 0.12)');
    root.style.setProperty('--danger', '#B42318');
    root.style.setProperty('--danger-fg', '#FFFFFF');
    root.style.setProperty('--danger-bg', 'rgba(180, 35, 24, 0.12)');
    root.style.setProperty('--warning', '#B86A00');
    root.style.setProperty('--warning-fg', '#FFFFFF');
    root.style.setProperty('--warning-bg', 'rgba(184, 106, 0, 0.12)');
    root.style.setProperty('--shadow-sm', '0 1px 2px rgba(15, 23, 42, 0.08)');
    root.style.setProperty('--shadow-md', '0 6px 16px rgba(15, 23, 42, 0.10), 0 2px 6px rgba(15, 23, 42, 0.06)');
    root.style.setProperty('--shadow-lg', '0 18px 40px rgba(15, 23, 42, 0.12), 0 8px 18px rgba(15, 23, 42, 0.08)');
  }

  localStorage.setItem('opifex-theme', isDark ? 'dark' : 'light');
}

const currentAgent = computed(() => state.currentAgent);
const currentPlan = computed(() => state.currentPlan);
const messages = computed(() => state.messages);
const models = computed(() => state.models);
const defaultModel = computed(() => state.defaultModel);
const loading = computed(() => state.loading);
const streaming = computed(() => state.streaming);
const streamingMsgId = computed(() => state.streamingMsgId);
const error = computed(() => state.error);
const apiKeys = computed(() => state.apiKeys);
const planVersions = computed(() => state.planVersions);
const interviewing = computed(() => state.interviewing);
const buildError = computed(() => state.buildError);
const isDark = computed(() => darkMode.value);
const compareVersionId = ref<string | null>(null);
const compareVersion = computed(() => state.planVersions.find(v => v.id === compareVersionId.value));
const templateSources = computed(() => state.templateSources);
const installedTemplates = computed(() => state.installedTemplates);
const marketplaceTemplates = computed(() => state.marketplaceTemplates);
const marketplaceLoading = computed(() => state.marketplaceLoading);
const selectedSourceId = ref<string | null>(null);

const setCompareVersion = (id: string | null) => {
  compareVersionId.value = id;
};

async function handleSelectAgent(id: string) {
  await selectAgent(id);
}

async function handleSendMessage(content: string) {
  await sendMessage(content);
}

async function handleNewConversation() {
  if (currentAgent.value) {
    await createNewConversation();
  }
}

function onBlockDragStart(block: BlockDefinition) {
  // Block drag start is handled by BlockPalette's native drag events.
  // The canvas drop handler receives the block via dataTransfer, with this as
  // a fallback in case the webview drops the payload.
  graphStore.setDraggingBlock(block);
}

async function handleCreateAgent() {
  if (!newAgentName.value.trim()) return;
  await createNewAgent(
    newAgentName.value,
    newAgentDescription.value || null,
    defaultModel.value ?? undefined
  );
  newAgentName.value = '';
  newAgentDescription.value = '';
  showCreateDialog.value = false;
}

async function handleDeleteAgent(id: string) {
  if (!confirm(`Delete agent "${id}"?`)) return;
  await deleteCurrentAgent();
}

async function handleGoHome() {
  state.currentAgent = null;
  state.currentPlan = null;
  state.messages = [];
  state.conversations = [];
  state.currentConversationId = null;
  state.planVersions = [];
  state.interviewing = true;
}

async function handlePullModel(modelName: string) {
  await pullModel(modelName);
}

async function handleSetDefaultModel(modelName: string) {
  await setDefaultModel(modelName);
}

async function handleSetApiKey(prov: string, key: string) {
  await setApiKey(prov, key);
}

async function handleApiKeyChange(prov: string, e: Event) {
  const target = e.target as HTMLInputElement;
  state.apiKeys[prov as keyof typeof state.apiKeys] = target.value;
}

async function handlePlanUpdate(section: keyof AgentPlan, value: string | string[]) {
  await updatePlanField(section, value);
}

async function handleAddTemplateSource(name: string, url: string) {
  await addTemplateSource(name, url);
  await loadTemplateSources();
}

async function handleLoadMarketplaceTemplates(sourceId: string) {
  await fetchMarketplaceTemplates(sourceId);
}

async function handleInstallMarketplaceTemplate(templateId: string, version: string, sourceId: string) {
  const result = await installMarketplaceTemplate(templateId, version, sourceId);
  if (result) {
    await loadInstalledTemplates();
  }
}

async function handleImportAgent() {
  const filePath = await tauriOpenDialog({
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  });

  if (filePath && filePath.length > 0) {
    try {
      const content = await tauriReadFile(filePath[0]);
      const text = new TextDecoder().decode(content);
      const agent = await importAgent(text);
      if (agent) {
        await selectAgent(agent.id);
      }
    } catch (err: any) {
      state.error = err?.message || String(err);
    }
  }
}

async function startRename(id: string) {
  renameId.value = id;
  renameName.value = '';
  showRenameDialog.value = true;
}

async function handleRenameAgent(id: string, newName: string) {
  await renameAgent(id, newName);
  showRenameDialog.value = false;
  renameId.value = null;
  renameName.value = '';
}

async function handleDuplicateAgent(id: string) {
  const newName = prompt('Enter name for the copy:');
  if (newName) {
    await duplicateAgent(id, newName);
  }
}

async function handleExportAgent() {
  if (currentAgent.value) {
    await exportAgent(currentAgent.value.id);
  }
}

async function handleSaveVersion() {
  const label = newVersionLabel.value.trim() || undefined;
  await savePlanVersion(label);
  newVersionLabel.value = '';

  if (currentAgent.value && currentPlan.value) {
    const planExport = {
      agentId: currentAgent.value.id,
      agentName: currentAgent.value.name,
      label: label || 'Untitled version',
      plan: currentPlan.value,
      exportedAt: new Date().toISOString(),
    };
    const json = JSON.stringify(planExport, null, 2);
    const defaultName = `${currentAgent.value.name || 'agent'}-plan.json`;
    const filePath = await tauriSaveDialog({
      defaultPath: defaultName,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (filePath) {
      try {
        await ipcWriteFile(filePath, json);
      } catch (err: any) {
        state.error = err?.message || String(err);
      }
    }
  }

  if (currentAgent.value) {
    await loadPlanVersions(currentAgent.value.id);
  }
}

async function handleRestoreVersion(versionId: string) {
  await restorePlanVersion(versionId);
  if (currentAgent.value) {
    await loadPlanVersions(currentAgent.value.id);
  }
}

async function setupKeyboardShortcuts() {
  document.addEventListener('keydown', (e) => {
    if (e.key === 'n' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      showCreateDialog.value = true;
    }
  });
}
</script>

<template>
  <div class="app-container" role="application">
    <header class="top-bar" role="banner">
      <div class="app-brand">
        <span class="brand-name">Opifex</span>
      </div>

      <div class="top-bar-center">
        <ModelSelector
          :models="models"
          :defaultModel="defaultModel ?? ''"
          :loading="loading"
          @change="handleSetDefaultModel"
          @pull="handlePullModel"
        />
      </div>

      <div class="top-bar-actions" aria-label="Top bar actions">
        <div class="action-group action-group--primary" aria-label="Primary actions">
          <button class="btn-primary" @click="showCreateDialog = true" title="New agent">
            New Agent
          </button>
          <div class="action-group action-group--segmented" role="tablist" aria-label="Workspace mode">
            <button
              class="btn-segment"
              :class="viewMode === 'canvas' ? 'is-active' : ''"
              @click="viewMode = 'canvas'"
              title="Canvas mode"
              role="tab"
              :aria-selected="viewMode === 'canvas'"
            >
              Canvas
            </button>
            <button
              class="btn-segment"
              :class="viewMode === 'oppy' ? 'is-active' : ''"
              @click="viewMode = 'oppy'"
              title="Oppy chat mode"
              role="tab"
              :aria-selected="viewMode === 'oppy'"
            >
              Oppy
            </button>
          </div>
        </div>

        <div class="action-group action-group--utility-menu" ref="utilityMenuRef">
          <button
            class="btn-utility btn-utility--menu"
            @click.stop="toggleUtilityMenu"
            :aria-expanded="showUtilityMenu"
            aria-haspopup="true"
            title="More actions"
          >
            ⋯
          </button>
          <div v-if="showUtilityMenu" class="utility-menu" role="menu">
            <button class="utility-menu-item" role="menuitem" @click="handleUtilityAction(() => { showProviderDialog = true })">
              API Keys
            </button>
            <button class="utility-menu-item" role="menuitem" @click="handleUtilityAction(() => { showImportDialog = true })">
              Import Agent
            </button>
            <button v-if="currentAgent" class="utility-menu-item" role="menuitem" @click="handleUtilityAction(handleGoHome)">
              Return to Library
            </button>
            <div class="utility-menu-divider"></div>
            <button class="utility-menu-item" role="menuitem" @click="handleUtilityAction(toggleDarkMode)">
              {{ isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode' }}
            </button>
          </div>
        </div>
      </div>
    </header>

    <div class="error-banner" v-if="error && !streaming">{{ error }}</div>

    <main class="panes" role="main">
      <div v-if="viewMode === 'canvas'" class="canvas-layout">
        <aside v-show="showPalette" class="canvas-palette" role="complementary" aria-label="Block palette">
          <BlockPalette :blocks="defaultBlocks" @block-drag-start="onBlockDragStart" />
        </aside>

        <section class="canvas-main" role="region" aria-label="Canvas workspace - system design">
          <header class="canvas-toolbar">
            <span class="canvas-title">Agent Canvas • Design your agent's logic and tools</span>
            <div class="canvas-toolbar-actions">
              <button
                class="btn-secondary"
                :class="{ 'is-on': showPalette }"
                :aria-pressed="showPalette"
                @click="showPalette = !showPalette"
                title="Show or hide the block palette"
              >
                Palette
              </button>
              <button
                class="btn-secondary"
                :class="{ 'is-on': showInspector }"
                :aria-pressed="showInspector"
                @click="showInspector = !showInspector"
                title="Show or hide the inspector"
              >
                Inspector
              </button>
              <button
                class="btn-secondary"
                @click="viewMode = 'oppy'"
                title="Switch to Oppy chat mode"
              >
                Switch to Oppy
              </button>
            </div>
          </header>
          <AgentCanvas />
        </section>

        <aside
          v-show="showInspector"
          class="canvas-inspector"
          role="complementary"
          aria-label="Inspector"
        >
          <div class="inspector-tabs" role="tablist" aria-label="Inspector panels">
            <button
              v-for="tab in inspectorTabs"
              :key="tab.id"
              class="inspector-tab"
              :class="{ 'is-active': inspectorTab === tab.id }"
              role="tab"
              :aria-selected="inspectorTab === tab.id"
              @click="inspectorTab = tab.id"
            >
              {{ tab.label }}
            </button>
          </div>

          <div class="inspector-body">
            <NodeInspector v-if="inspectorTab === 'node'" :nodeId="selectedCanvasNodeId" />
            <AgentToolDrawer
              v-else-if="inspectorTab === 'tools'"
              :nodeId="selectedCanvasNodeId"
            />
            <PinPanel
              v-else-if="inspectorTab === 'pins'"
              :agent-id="currentAgent?.id"
              :node-id="selectedCanvasNodeId"
              :port-id="undefined"
              :value="undefined"
            />
            <SkillEditor
              v-else-if="inspectorTab === 'skills'"
              :agent-id="currentAgent?.id"
            />
            <KnowledgeSourceEditor
              v-else-if="inspectorTab === 'knowledge'"
              :agent-id="currentAgent?.id"
            />
            <TriggerEditor
              v-else-if="inspectorTab === 'triggers'"
              :agent-id="currentAgent?.id"
            />
          </div>
        </aside>
      </div>

      <div v-else class="oppy-layout">
        <aside class="chat-pane-wrapper" role="complementary" aria-label="Chat workspace">
          <ChatPane
            v-if="currentAgent"
            :messages="messages"
            :streaming="streaming"
            :streaming-msg-id="streamingMsgId ?? ''"
            :current-agent-name="currentAgent?.name ?? ''"
            :interviewing="interviewing"
            :build-error="buildError"
            @send="handleSendMessage"
            @build="confirmBuild"
            @retry="retryBuild"
            @new-conversation="handleNewConversation"
          />

          <AgentLibrary
            v-if="!currentAgent"
            :agents="state.agents"
            :template-sources="templateSources"
            :installed-templates="installedTemplates"
            :marketplace-templates="marketplaceTemplates"
            :marketplace-loading="marketplaceLoading"
            :selected-source-id="selectedSourceId"
            @select="handleSelectAgent"
            @create="() => { showCreateDialog = true }"
            @rename="startRename"
            @duplicate="handleDuplicateAgent"
            @export="handleExportAgent"
            @delete="handleDeleteAgent"
            @add-template-source="handleAddTemplateSource"
            @load-template-sources="loadTemplateSources"
            @load-marketplace-templates="handleLoadMarketplaceTemplates"
            @install-marketplace-template="handleInstallMarketplaceTemplate"
            @load-installed-templates="loadInstalledTemplates"
          />
        </aside>

        <div class="divider"></div>

        <section class="plan-pane-wrapper" role="region" aria-label="Plan review">
          <PlanPane
            :plan="currentPlan"
            :agent-name="currentAgent?.name || 'Agent'"
            :versions="planVersions"
            :compare-version-id="compareVersionId"
            :compare-version="compareVersion"
            @update="handlePlanUpdate"
            @save-version="handleSaveVersion"
            @restore-version="handleRestoreVersion"
            @compare-version="setCompareVersion"
            @export-agent="handleExportAgent"
          />
        </section>
      </div>
    </main>

    <div class="modal-overlay" v-if="showCreateDialog" @click.self="showCreateDialog = false">
      <div class="modal" @click.stop>
        <h3>Create Agent</h3>
        <div class="form-group">
          <label>Name</label>
          <input
            v-model="newAgentName"
            type="text"
            placeholder="e.g. Research Assistant"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label>Description</label>
          <input
            v-model="newAgentDescription"
            type="text"
            placeholder="What does this agent do?"
            class="form-input"
          />
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showCreateDialog = false">Cancel</button>
          <button class="btn-primary" @click="handleCreateAgent">Create Agent</button>
        </div>
      </div>
    </div>

    <div class="modal-overlay" v-if="showImportDialog" @click.self="showImportDialog = false">
      <div class="modal" @click.stop>
        <h3>Import Agent</h3>
        <p class="form-hint">Upload a JSON agent specification file.</p>
        <div class="form-actions">
          <button class="btn-cancel" @click="showImportDialog = false">Cancel</button>
          <button class="btn-primary" @click="handleImportAgent">Import</button>
        </div>
      </div>
    </div>

    <div class="modal-overlay" v-if="showRenameDialog" @click.self="showRenameDialog = false">
      <div class="modal" @click.stop>
        <h3>Rename Agent</h3>
        <div class="form-group">
          <label>New Name</label>
          <input
            v-model="renameName"
            type="text"
            class="form-input"
          />
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showRenameDialog = false">Cancel</button>
          <button class="btn-primary" @click="handleRenameAgent(renameId!, renameName)">Rename</button>
        </div>
      </div>
    </div>

    <div class="modal-overlay" v-if="showProviderDialog" @click.self="showProviderDialog = false">
      <div class="modal" @click.stop>
        <h3>Configure API Keys</h3>
        <div class="form-group">
          <label>OpenAI API Key</label>
          <input
            :value="apiKeys.openai || ''"
            @input="(e) => handleApiKeyChange('openai', e)"
            type="password"
            placeholder="sk-..."
            class="form-input"
          />
          <button class="btn-save-key" @click="handleSetApiKey('openai', state.apiKeys.openai || '')">Save</button>
        </div>
        <div class="form-group">
          <label>Anthropic API Key</label>
          <input
            :value="apiKeys.anthropic || ''"
            @input="(e) => handleApiKeyChange('anthropic', e)"
            type="password"
            placeholder="sk-ant-..."
            class="form-input"
          />
          <button class="btn-save-key" @click="handleSetApiKey('anthropic', state.apiKeys.anthropic || '')">Save</button>
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showProviderDialog = false">Close</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
:global(body) {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}
:root {
  /* Surfaces */
  --surface-base: #0B1020;
  --surface-card: #111827;
  --surface-elevated: #151E2D;
  --surface-hover: #1F2937;
  --surface-muted: #0F172A;

  /* Text */
  --text-primary: #F3F7FF;
  --text-secondary: #B4C0D2;
  --text-tertiary: #7C8AA5;
  --text-muted: #94A3B8;

  /* Border */
  --border: #243247;
  --border-focus: #3B82F6;
  --border-hover: #334155;

  /* Interactive */
  --primary: #3B82F6;
  --primary-hover: #2563EB;
  --primary-fg: #FFFFFF;
  --primary-bg: rgba(59, 130, 246, 0.15);

  --success: #22C55E;
  --success-fg: #FFFFFF;
  --success-bg: rgba(34, 197, 94, 0.15);

  --danger: #EF4444;
  --danger-fg: #FFFFFF;
  --danger-bg: rgba(239, 68, 68, 0.15);

  --warning: #F59E0B;
  --warning-fg: #FFFFFF;
  --warning-bg: rgba(245, 158, 11, 0.15);

  /* Shadows — soft, offset shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
  --shadow-md: 0 6px 16px rgba(0, 0, 0, 0.5), 0 2px 6px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 18px 40px rgba(0, 0, 0, 0.6), 0 8px 18px rgba(0, 0, 0, 0.5);

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;

  /* Spacing scale (4px base) */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-10: 2.5rem;
  --space-12: 3rem;

  /* Typography scale */
  --type-xs: 0.75rem;
  --type-sm: 0.875rem;
  --type-base: 1rem;
  --type-lg: 1.125rem;
  --type-xl: 1.25rem;
  --type-2xl: 1.5rem;
  --type-3xl: 1.875rem;
  --type-display: 2.25rem;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  scrollbar-width: thin;
  scrollbar-color: var(--border) var(--surface-base);
}

body {
  margin: 0;
  font-size: var(--type-base);
  line-height: 1.5;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: var(--text-primary);
  background: var(--surface-base);
  -webkit-font-smoothing: antialiased;
}

::selection {
  background: var(--primary);
  color: #ffffff;
}

::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: var(--surface-base);
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 5px;
  border: 2px solid var(--surface-base);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}

:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px var(--border-focus);
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--surface-base);
  color: var(--text-primary);
  overflow: hidden;
}

.top-bar {
  height: 64px;
  padding: 0 var(--space-6);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: var(--space-4);
  flex-shrink: 0;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.brand-name {
  font-size: var(--type-xl);
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.top-bar-center {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-4);
  min-width: 0;
}

.top-bar-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  min-width: 0;
}

.action-group {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--surface-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-1);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

.action-group--primary {
  background: var(--surface-base);
}

.action-group--segmented {
  padding: 0;
  overflow: hidden;
  margin-left: var(--space-1);
}

.action-group--utility-menu {
  position: relative;
  padding: 0;
  overflow: visible;
  background: transparent;
  border-color: transparent;
  box-shadow: none;
}

.btn-utility,
.btn-segment,
.btn-primary {
  appearance: none;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--type-sm);
  line-height: 1.2;
  font-weight: 600;
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease, box-shadow 0.2s ease, transform 0.12s ease;
}

.btn-secondary,
.btn-utility {
  padding: 0.5rem 0.8rem;
  background: transparent;
  color: var(--text-primary);
  border-color: transparent;
}

.btn-secondary:hover,
.btn-utility:hover {
  background: var(--surface-hover);
  border-color: var(--border);
}

.btn-secondary {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  color: var(--text-primary);
}

.btn-utility--theme {
  min-width: 4.5rem;
}

.btn-utility--menu {
  padding: 0.5rem 0.7rem;
  font-size: var(--type-lg);
  line-height: 1;
  min-width: 2.25rem;
}

.btn-segment {
  padding: 0.5rem 0.85rem;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 0;
  border: none;
  min-width: 4.75rem;
}

.btn-segment + .btn-segment {
  border-left: 1px solid var(--border);
}

.btn-segment.is-active {
  background: var(--primary-bg);
  color: var(--primary);
  box-shadow: inset 0 0 0 1px rgba(11, 99, 217, 0.12);
}

.btn-primary {
  padding: 0.72rem 1.15rem;
  background: var(--primary);
  color: var(--primary-fg);
  border-color: var(--primary);
  box-shadow: 0 6px 16px rgba(11, 99, 217, 0.22);
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
  border-color: var(--primary-hover);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(1px);
}

.btn-primary:disabled {
  background: var(--border-hover);
  border-color: var(--border-hover);
  color: var(--text-muted);
  box-shadow: none;
  cursor: not-allowed;
}

.utility-menu {
  position: absolute;
  top: calc(100% + var(--space-2));
  right: 0;
  min-width: 180px;
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  z-index: 50;
  display: flex;
  flex-direction: column;
}

.utility-menu-item {
  appearance: none;
  border: none;
  background: transparent;
  text-align: left;
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  color: var(--text-primary);
  cursor: pointer;
  font-weight: 500;
}

.utility-menu-item:hover {
  background: var(--surface-hover);
}

.utility-menu-divider {
  height: 1px;
  background: var(--border);
  margin: var(--space-1) 0;
}

.btn-text {
  padding: var(--space-3);
  font-size: var(--type-sm);
  background: transparent;
  color: var(--text-secondary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-text:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.top-bar .btn-utility:focus-visible,
.top-bar .btn-segment:focus-visible,
.top-bar .btn-primary:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(11, 99, 217, 0.22);
}

.error-banner {
  padding: var(--space-3) var(--space-6);
  background: var(--danger-bg);
  color: var(--danger-fg);
  font-size: var(--type-sm);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.panes {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.chat-pane-wrapper {
  flex: 1;
  min-width: 340px;
  max-width: 560px;
  display: flex;
  flex-direction: column;
  background: var(--surface-card);
  border-right: 1px solid var(--border);
  overflow-y: auto;
}

.divider {
  width: 1px;
  background: var(--border);
  flex-shrink: 0;
}

.plan-pane-wrapper {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface-card);
  overflow: hidden;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(4px);
}

.modal {
  background: var(--surface-card);
  padding: var(--space-6);
  border-radius: var(--radius-lg);
  min-width: 440px;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border);
}

.modal h3 {
  margin: 0 0 var(--space-5) 0;
  font-size: var(--type-lg);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.form-hint {
  font-size: var(--type-sm);
  color: var(--text-secondary);
  margin-bottom: var(--space-4);
}

.form-group {
  margin-bottom: var(--space-4);
}

.form-group label {
  display: block;
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: var(--space-1);
  letter-spacing: 0.02em;
}

.form-input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  font-family: inherit;
  background: var(--surface-card);
  color: var(--text-primary);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  margin-top: var(--space-5);
}

.btn-cancel {
  padding: var(--space-2) var(--space-5);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-cancel:hover {
  background: var(--surface-hover);
}

.version-save-bar {
  padding: var(--space-4) var(--space-6);
  background: var(--surface-card);
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex-shrink: 0;
}

.version-save-bar label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-input {
  flex: 1;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  font-family: inherit;
  background: var(--surface-card);
  color: var(--text-primary);
}

.version-input:focus {
  outline: none;
  border-color: var(--border-focus);
}

.btn-save-version-bar {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--success);
  color: var(--success-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
}

.btn-save-version-bar:hover {
  background: var(--success);
}

.btn-save-key {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  margin-left: var(--space-2);
}

.btn-save-key:hover {
  background: var(--primary-hover);
}

/* Canvas mode layout */
.canvas-layout {
  position: relative;
  display: flex;
  flex: 1;
  overflow: hidden;
  background: var(--surface-base);
}

.canvas-palette {
  width: 320px;
  flex-shrink: 0;
  background: var(--surface-card);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.canvas-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-base);
}

.canvas-toolbar {
  height: 48px;
  padding: 0 var(--space-6);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.canvas-title {
  font-size: var(--type-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.canvas-toolbar-actions {
  display: flex;
  gap: var(--space-2);
}

.canvas-inspector {
  width: 340px;
  flex-shrink: 0;
  background: var(--surface-card);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.inspector-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  padding: var(--space-2);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.inspector-tab {
  padding: 4px 8px;
  font-family: inherit;
  font-size: var(--type-xs);
  color: var(--text-secondary);
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.inspector-tab:hover {
  color: var(--text-primary);
  background: var(--surface-hover);
}

.inspector-tab.is-active {
  color: var(--primary);
  background: rgba(59, 130, 246, 0.12);
  border-color: rgba(59, 130, 246, 0.35);
}

.inspector-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.canvas-toolbar-actions .btn-secondary.is-on {
  color: var(--primary);
  background: rgba(59, 130, 246, 0.12);
  border-color: rgba(59, 130, 246, 0.4);
}

.canvas-title {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.canvas-toolbar-actions {
  flex-shrink: 0;
}

/* In a narrow window the rails float over the canvas instead of squeezing it.
   The default Tauri window is 800x600, where two fixed rails would leave the
   canvas with almost no width. */
@media (max-width: 1240px) {
  .canvas-palette,
  .canvas-inspector {
    position: absolute;
    top: 0;
    bottom: 0;
    z-index: 30;
    box-shadow: 0 12px 32px rgba(2, 6, 23, 0.28);
  }

  .canvas-palette {
    left: 0;
  }

  .canvas-inspector {
    right: 0;
  }

  .canvas-main {
    width: 100%;
  }
}

@media (max-width: 900px) {
  .canvas-palette {
    width: 260px;
  }

  .canvas-inspector {
    width: 300px;
  }

  .canvas-title {
    display: none;
  }
}

.btn-active {
  color: var(--primary);
  font-weight: 600;
  text-decoration: underline;
}

:global([data-theme='light'] .bg-slate-950),
:global([data-theme='light'] .bg-slate-900),
:global([data-theme='light'] .bg-slate-800),
:global([data-theme='light'] .bg-slate-800\/50),
:global([data-theme='light'] .bg-slate-700),
:global([data-theme='light'] .bg-slate-700\/50) {
  background-color: var(--surface-card) !important;
}

:global([data-theme='light'] .text-slate-200),
:global([data-theme='light'] .text-slate-300),
:global([data-theme='light'] .text-slate-400),
:global([data-theme='light'] .text-slate-500),
:global([data-theme='light'] .text-slate-600) {
  color: var(--text-secondary) !important;
}

:global([data-theme='light'] .text-slate-100),
:global([data-theme='light'] .text-slate-50),
:global([data-theme='light'] .text-slate-200) {
  color: var(--text-primary) !important;
}

:global([data-theme='light'] .border-slate-800),
:global([data-theme='light'] .border-slate-700),
:global([data-theme='light'] .border-slate-700\/50),
:global([data-theme='light'] .border-slate-600) {
  border-color: var(--border) !important;
}

:global([data-theme='light'] .hover\:bg-slate-800:hover) {
  background-color: var(--surface-hover) !important;
}
</style>