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

function toggleDarkMode() {
  darkMode.value = !darkMode.value;
}

function applyDarkMode() {
  const isDark = darkMode.value;
  const root = document.documentElement;
  root.setAttribute('data-theme', isDark ? 'dark' : 'light');

  if (isDark) {
    root.style.setProperty('--surface-base', '#111827');
    root.style.setProperty('--surface-card', '#1F2937');
    root.style.setProperty('--surface-hover', '#374151');
    root.style.setProperty('--surface-muted', '#111827');
    root.style.setProperty('--text-primary', '#F9FAFB');
    root.style.setProperty('--text-secondary', '#9CA3AF');
    root.style.setProperty('--text-muted', '#D1D5DB');
    root.style.setProperty('--border', '#374151');
    root.style.setProperty('--border-focus', '#3B82F6');
    root.style.setProperty('--border-hover', '#4B5563');
    root.style.setProperty('--primary', '#3B82F6');
    root.style.setProperty('--primary-hover', '#2563EB');
    root.style.setProperty('--primary-fg', '#FFFFFF');
    root.style.setProperty('--primary-bg', 'rgba(59, 130, 246, 0.15)');
    root.style.setProperty('--success', '#22C55E');
    root.style.setProperty('--success-fg', '#FFFFFF');
    root.style.setProperty('--success-bg', 'rgba(34, 197, 94, 0.15)');
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
  <div class="app-container">
    <div class="top-bar">
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

      <div class="top-bar-actions">
        <div class="action-group">
          <button class="btn-secondary" @click="showProviderDialog = true" title="Configure API keys">
            API Keys
          </button>
          <button class="btn-secondary" @click="showImportDialog = true" title="Import agent">Import</button>
        </div>

        <div class="action-group">
          <button class="btn-primary" @click="showCreateDialog = true" title="New agent">
            New Agent
          </button>
        </div>

        <div class="action-group">
          <button class="btn-text" @click="toggleDarkMode" :title="isDark ? 'Light mode' : 'Dark mode'">
            {{ isDark ? 'Light' : 'Dark' }}
          </button>
        </div>
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
        @compare-version="setCompareVersion"
        @export-agent="handleExportAgent"
      />
    </div>

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
/* Opifex design tokens — Operate mode, desktop, Relevance AI inspired */
:root {
  /* Surfaces */
  --surface-base: #FAFAF9;
  --surface-card: #FFFFFF;
  --surface-hover: #F3F4F6;
  --surface-muted: #F9FAFB;

  /* Text */
  --text-primary: #111827;
  --text-secondary: #6B7280;
  --text-muted: #9CA3AF;

  /* Border */
  --border: #E5E7EB;
  --border-focus: #2563EB;
  --border-hover: #D1D5DB;

  /* Interactive */
  --primary: #2563EB;
  --primary-hover: #1D4ED8;
  --primary-fg: #FFFFFF;
  --primary-bg: rgba(37, 99, 235, 0.10);

  --success: #16A34A;
  --success-fg: #FFFFFF;
  --success-bg: rgba(22, 163, 74, 0.10);

  --danger: #DC2626;
  --danger-fg: #FFFFFF;
  --danger-bg: rgba(220, 38, 38, 0.10);

  --warning: #CA8A04;
  --warning-fg: #FFFFFF;
  --warning-bg: rgba(202, 138, 4, 0.10);

  /* Shadows — soft, offset shadows */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);

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
  display: flex;
  align-items: center;
  justify-content: space-between;
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
  gap: var(--space-4);
}

.top-bar-actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.action-group {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.btn-secondary {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
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
  transition: all 0.2s;
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

.btn-text {
  padding: var(--space-2);
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
</style>