<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { save as tauriSaveDialog, open as tauriOpenDialog } from '@tauri-apps/plugin-dialog';
import { readFile as tauriReadFile } from '@tauri-apps/plugin-fs';
import { useAgentStore } from '@/stores/agentStore';
import ChatPane from '@/components/ChatPane.vue';
import PlanPane from '@/components/PlanPane.vue';
import AgentLibrary from '@/components/AgentLibrary.vue';
import ModelSelector from '@/components/ModelSelector.vue';
import { writeFile as ipcWriteFile } from '@/services/ipc';
import type { AgentPlan, InstalledTemplate, TemplateSource } from '@/types/agent';

const store = useAgentStore();
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
  setProvider,
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

onMounted(async () => {
  init();
  applyDarkMode();
  setupKeyboardShortcuts();
  await loadTemplateSources();
  await loadInstalledTemplates();
});

function setupKeyboardShortcuts() {
  window.addEventListener('keydown', (e) => {
    // Don't trigger shortcuts when typing in inputs/textareas
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
      return;
    }

    const isCtrlOrMeta = e.ctrlKey || e.metaKey;

    if (isCtrlOrMeta && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      showCreateDialog.value = true;
    } else if (isCtrlOrMeta && e.key.toLowerCase() === 's') {
      e.preventDefault();
      if (currentAgent.value && currentPlan.value) {
        handleSaveVersion();
      }
    } else if (isCtrlOrMeta && e.key.toLowerCase() === 'o') {
      e.preventDefault();
      showImportDialog.value = true;
    } else if (isCtrlOrMeta && e.key.toLowerCase() === 'e') {
      e.preventDefault();
      if (currentAgent.value) {
        handleExportAgent(currentAgent.value.id);
      }
    } else if (isCtrlOrMeta && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      if (currentAgent.value) {
        handleDeleteAgent(currentAgent.value.id);
      }
    }
  });
}

function toggleDarkMode() {
  darkMode.value = !darkMode.value;
  applyDarkMode();
}

function applyDarkMode() {
  const isDark = darkMode.value;
  const root = document.documentElement;
  root.setAttribute('data-theme', isDark ? 'dark' : 'light');

  if (isDark) {
    root.style.setProperty('--surface-base', '#16181B');
    root.style.setProperty('--surface-card', '#202328');
    root.style.setProperty('--surface-hover', '#2A2D34');
    root.style.setProperty('--surface-muted', '#1E2126');
    root.style.setProperty('--text-primary', '#F1F3F5');
    root.style.setProperty('--text-secondary', '#9BA1A9');
    root.style.setProperty('--text-muted', '#D0D0D0');
    root.style.setProperty('--border', '#7A7F88');
    root.style.setProperty('--border-focus', '#82BCF4');
    root.style.setProperty('--border-hover', '#B8B8B8');
    root.style.setProperty('--primary', '#82BCF4');
    root.style.setProperty('--primary-hover', '#98C8F7');
    root.style.setProperty('--primary-fg', '#09192A');
    root.style.setProperty('--primary-bg', 'rgba(130, 188, 244, 0.15)');
    root.style.setProperty('--success', '#63B87E');
    root.style.setProperty('--success-fg', '#E1F7EA');
    root.style.setProperty('--success-bg', 'rgba(99, 184, 126, 0.15)');
    root.style.setProperty('--danger', '#E06C6C');
    root.style.setProperty('--danger-fg', '#FBE4E4');
    root.style.setProperty('--danger-bg', 'rgba(224, 108, 108, 0.15)');
    root.style.setProperty('--warning', '#E2B350');
    root.style.setProperty('--warning-fg', '#FBE9D8');
    root.style.setProperty('--warning-bg', 'rgba(226, 179, 80, 0.15)');
    root.style.setProperty('--shadow-sm', '0 1px 2px rgba(0, 0, 0, 0.4)');
    root.style.setProperty('--shadow-md', '0 4px 12px rgba(0, 0, 0, 0.5)');
    root.style.setProperty('--shadow-lg', '0 12px 32px rgba(0, 0, 0, 0.6)');
  } else {
    root.style.removeProperty('--surface-base');
    root.style.removeProperty('--surface-card');
    root.style.removeProperty('--surface-hover');
    root.style.removeProperty('--surface-muted');
    root.style.removeProperty('--text-primary');
    root.style.removeProperty('--text-secondary');
    root.style.removeProperty('--text-muted');
    root.style.removeProperty('--border');
    root.style.removeProperty('--border-focus');
    root.style.removeProperty('--border-hover');
    root.style.removeProperty('--primary');
    root.style.removeProperty('--primary-hover');
    root.style.removeProperty('--primary-fg');
    root.style.removeProperty('--primary-bg');
    root.style.removeProperty('--success');
    root.style.removeProperty('--success-fg');
    root.style.removeProperty('--success-bg');
    root.style.removeProperty('--danger');
    root.style.removeProperty('--danger-fg');
    root.style.removeProperty('--danger-bg');
    root.style.removeProperty('--warning');
    root.style.removeProperty('--warning-fg');
    root.style.removeProperty('--warning-bg');
    root.style.removeProperty('--shadow-sm');
    root.style.removeProperty('--shadow-md');
    root.style.removeProperty('--shadow-lg');
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
const provider = computed(() => state.provider);
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

async function goHome() {
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

async function handleSetProvider(prov: string) {
  await setProvider(prov);
}

async function handleSetApiKey(prov: string, key: string) {
  await setApiKey(prov, key);
}

async function handleApiKeyChange(prov: string, e: Event) {
  const target = e.target as HTMLInputElement;
  state.apiKeys[prov as keyof typeof state.apiKeys] = target.value;
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

async function handlePlanUpdate(section: keyof AgentPlan, value: string | string[]) {
  await updatePlanField(section, value);
}

async function handleSaveVersion() {
  const label = newVersionLabel.value.trim() || undefined;
  await savePlanVersion(label);
  newVersionLabel.value = '';

  // Open native Windows Save dialog
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

  // Reload versions for current agent
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

async function handleCompareVersion(versionId: string | null) {
  compareVersionId.value = versionId;
}

async function handleExportAgent(id: string) {
  try {
    const json = await exportAgent(id);
    const defaultName = `${currentAgent.value?.name || 'agent'}.json`;
    const filePath = await tauriSaveDialog({
      defaultPath: defaultName,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (filePath) {
      await ipcWriteFile(filePath, json);
    }
  } catch (err: any) {
    state.error = err?.message || String(err);
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
</script>

<template>
  <div class="app-container">
    <div class="top-bar">
      <div class="app-brand">
        <span class="brand-name">Opifex</span>
      </div>

      <div class="top-bar-actions">
        <ModelSelector
          :models="models"
          :defaultModel="defaultModel ?? ''"
          :loading="loading"
          @change="handleSetDefaultModel"
          @pull="handlePullModel"
        />

        <div class="provider-selector">
          <select
            v-model="provider"
            class="provider-select"
            @change="(e) => handleSetProvider((e.target as HTMLSelectElement).value)"
          >
            <option value="ollama">Ollama (Local)</option>
            <option value="openai">OpenAI</option>
            <option value="anthropic">Anthropic</option>
          </select>
          <button
            class="btn-api-key"
            @click="showProviderDialog = true"
            title="Configure API keys"
          >
            API Keys
          </button>
        </div>

        <button class="btn-import" @click="showImportDialog = true" title="Import agent">Import</button>
        <button class="btn-new" @click="showCreateDialog = true" title="New agent">New Agent</button>
        <button
          v-if="currentAgent"
          class="btn-save-agent-header"
          @click="handleExportAgent(currentAgent.id)"
          title="Save agent"
        >
          Save Agent
        </button>
        <button
          v-if="currentAgent"
          class="btn-home"
          @click="goHome"
          title="Return to library"
        >
          Library
        </button>
        <button class="btn-dark-toggle" @click="toggleDarkMode" :title="isDark ? 'Light mode' : 'Dark mode'">
          {{ isDark ? 'Light' : 'Dark' }}
        </button>
      </div>
    </div>

    <div class="error-banner" v-if="error && !streaming">{{ error }}</div>

    <div class="panes">
      <div class="chat-pane-wrapper">
        <ChatPane
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
      </div>

      <div class="divider"></div>

      <PlanPane
        :plan="currentPlan"
        :agent-name="currentAgent?.name || 'Agent'"
        :versions="planVersions"
        :compare-version-id="compareVersionId"
        :compare-version="compareVersion"
        @update="handlePlanUpdate"
        @save-version="handleSaveVersion"
        @restore-version="handleRestoreVersion"
        @compare-version="handleCompareVersion"
        @export-agent="() => currentAgent && handleExportAgent(currentAgent.id)"
      />
    </div>

    <div v-if="showCreateDialog" class="modal-overlay" @click.self="showCreateDialog = false">
      <div class="modal">
        <h3>Create New Agent</h3>
        <div class="form-group">
          <label>Name</label>
          <input
            v-model="newAgentName"
            placeholder="e.g. Customer Support Agent"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label>Description</label>
          <textarea
            v-model="newAgentDescription"
            placeholder="What does this agent do?"
            class="form-input"
            rows="3"
          ></textarea>
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showCreateDialog = false">Cancel</button>
          <button class="btn-create" @click="handleCreateAgent" :disabled="!newAgentName.trim()">
            Create Agent
          </button>
        </div>
      </div>
    </div>

    <div v-if="showImportDialog" class="modal-overlay" @click.self="showImportDialog = false">
      <div class="modal">
        <h3>Import Agent</h3>
        <div class="form-group">
          <label>Select agent JSON file</label>
          <button class="btn-create" @click="handleImportAgent">Browse Files</button>
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showImportDialog = false">Close</button>
        </div>
      </div>
    </div>

    <div v-if="showRenameDialog && renameId" class="modal-overlay" @click.self="showRenameDialog = false">
      <div class="modal">
        <h3>Rename Agent</h3>
        <div class="form-group">
          <label>New Name</label>
          <input
            v-model="renameName"
            class="form-input"
          />
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showRenameDialog = false">Cancel</button>
          <button class="btn-create" @click="handleRenameAgent(renameId!, renameName)" :disabled="!renameName.trim()">
            Rename
          </button>
        </div>
      </div>
    </div>

    <div v-if="showProviderDialog" class="modal-overlay" @click.self="showProviderDialog = false">
      <div class="modal">
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

    <div v-if="currentAgent && currentPlan" class="version-save-bar">
      <label>Save plan version:</label>
      <input
        v-model="newVersionLabel"
        class="version-input"
        placeholder="Version label (optional)"
      />
      <button class="btn-save-version-bar" @click="handleSaveVersion">Save Version</button>
    </div>
  </div>
</template>

<style scoped>
/* Opifex design tokens — Operate mode, desktop */
:root {
  /* Surfaces */
  --surface-base: #FAF7F3;
  --surface-card: #FFFFFF;
  --surface-hover: #F5F2ED;
  --surface-muted: #F0ECE6;

  /* Text */
  --text-primary: #24262B;
  --text-secondary: #5A5F68;
  --text-muted: #6E737C;

  /* Border */
  --border: #8A8F98;
  --border-focus: #3A7BD5;
  --border-hover: #6E737C;

  /* Interactive */
  --primary: #4A90E2;
  --primary-hover: #357ABD;
  --primary-fg: #0F2840;
  --primary-bg: rgba(123, 179, 232, 0.10);

  --success: #88CF9E;
  --success-fg: #133821;
  --success-bg: rgba(136, 207, 158, 0.10);

  --danger: #F29393;
  --danger-fg: #4A1E1E;
  --danger-bg: rgba(242, 147, 147, 0.10);

  --warning: #F5CD79;
  --warning-fg: #4A3A0F;
  --warning-bg: rgba(245, 205, 121, 0.10);

  /* Shadows — offset + blur, no zero-offset halos */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.06);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.12);

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 14px;

  /* Spacing scale (4px base) */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;

  /* Typography scale */
  --type-xs: 0.75rem;
  --type-sm: 0.875rem;
  --type-base: 1rem;
  --type-lg: 1.125rem;
  --type-xl: 1.25rem;
  --type-display: 1.5rem;
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
  padding: var(--space-4) var(--space-6);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.top-bar-actions {
  display: flex;
  gap: var(--space-3);
  align-items: center;
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
  max-width: 520px;
  display: flex;
  flex-direction: column;
  background: var(--surface-card);
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.divider {
  width: 1px;
  background: var(--border);
  flex-shrink: 0;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(22, 24, 27, 0.4);
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
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.1);
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
  background: var(--surface-muted);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s, border-color 0.2s;
}

.btn-cancel:hover {
  background: var(--border-hover);
  border-color: var(--border-focus);
}

.btn-primary {
  padding: var(--space-2) var(--space-5);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s, transform 0.1s;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(1px);
}

.btn-primary:disabled {
  background: var(--border-hover);
  color: var(--text-muted);
  cursor: not-allowed;
}

.version-save-bar {
  padding: var(--space-4) var(--space-6);
  background: var(--surface-card);
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 0.75rem;
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
  background: #7BB3E8;
  border-color: var(--border-focus);
}

.provider-selector {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.provider-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.provider-select {
padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  background: var(--surface-card);
  color: var(--text-primary);
  min-width: 110px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.provider-select:focus {
  outline: none;
  border-color: var(--border-focus);
}

.btn-api-key {
  padding: var(--space-2);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--type-base);
}

.btn-api-key:hover {
  background: var(--surface-hover);
  border-color: var(--border-focus);
}

.btn-import, .btn-new {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
}

.btn-import:hover {
  background: var(--surface-hover);
  border-color: var(--border-focus);
}

.btn-new:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-save-agent-header {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.btn-save-agent-header:hover {
  background: var(--surface-hover);
  border-color: var(--border-focus);
}

.btn-home {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
}

.btn-home:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-dark-toggle {
  padding: var(--space-2);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--type-base);
}

.btn-dark-toggle:hover {
  background: var(--surface-hover);
  border-color: var(--border-focus);
}
</style>
