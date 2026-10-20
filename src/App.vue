<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { save as tauriSaveDialog, open as tauriOpenDialog } from '@tauri-apps/plugin-dialog';
import { useAgentStore } from '@/stores/agentStore';
import ChatPane from '@/components/ChatPane.vue';
import PlanPane from '@/components/PlanPane.vue';
import AgentLibrary from '@/components/AgentLibrary.vue';
import ModelSelector from '@/components/ModelSelector.vue';
import { writeFile as ipcWriteFile } from '@/services/ipc';
import type { AgentPlan } from '@/types/agent';

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

onMounted(() => {
  init();
  applyDarkMode();
  setupKeyboardShortcuts();
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
  document.documentElement.setAttribute('data-theme', darkMode.value ? 'dark' : 'light');
  localStorage.setItem('opifex-theme', darkMode.value ? 'dark' : 'light');
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
      // Read the file content using the Tauri read command via IPC
      const { readFile } = await import('@tauri-apps/plugin-fs');
    const content = await readFile(filePath[0]);
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
        <span class="brand-icon">⬢</span>
        <span class="brand-name">Opifex</span>
        <span class="brand-tagline">Agent Builder</span>
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
          <label class="provider-label">Provider</label>
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
            🔑
          </button>
        </div>

        <button class="btn-import" @click="showImportDialog = true" title="Import agent">⬆</button>
        <button class="btn-new" @click="showCreateDialog = true" title="New agent">+ Agent</button>
        <button
          v-if="currentAgent"
          class="btn-save-agent-header"
          @click="handleExportAgent(currentAgent.id)"
          title="Save agent with Windows Save dialog"
        >
          💾 Save Agent
        </button>
        <button
          v-if="currentAgent"
          class="btn-home"
          @click="goHome"
          title="Return to library"
        >
          ↥ Home
        </button>
        <button class="btn-dark-toggle" @click="toggleDarkMode" :title="isDark ? 'Light mode' : 'Dark mode'">
          {{ isDark ? '☀️' : '🌙' }}
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
          @select="handleSelectAgent"
          @create="() => { showCreateDialog = true }"
          @rename="startRename"
          @duplicate="handleDuplicateAgent"
          @export="handleExportAgent"
          @delete="handleDeleteAgent"
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
            @input="(e) => { state.apiKeys.openai = (e.target as HTMLInputElement).value }"
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
            @input="(e) => { state.apiKeys.anthropic = (e.target as HTMLInputElement).value }"
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
:root {
  --bg-primary: #f8fafc;
  --bg-secondary: #ffffff;
  --bg-tertiary: #f1f5f9;
  --bg-muted: #e2e8f0;
  --text-primary: #1a1a1a;
  --text-secondary: #6b7280;
  --text-muted: #64748b;
  --border: #6b7280;
  --border-hover: #64748b;
  --input-bg: #ffffff;
  --input-border: #6b7280;
  --accent: #0066cc;
  --accent-hover: #0052a3;
  --success: #10b981;
  --success-hover: #059669;
  --danger: #dc2626;
  --danger-hover: #b91c1c;
  --shadow: rgba(0, 0, 0, 0.1);
}

[data-theme="dark"] .app-container {
  --bg-primary: #0f172a;
  --bg-secondary: #1e293b;
  --bg-tertiary: #334155;
  --bg-muted: #475569;
  --text-primary: #f1f5f9;
  --text-secondary: #cbd5e1;
  --text-muted: #94a3b8;
  --border: #cbd5e1;
  --border-hover: #a5b8c8;
  --input-bg: #334155;
  --input-border: #cbd5e1;
  --accent: #3b82f6;
  --accent-hover: #2563eb;
  --success: #22c55e;
  --success-hover: #16a34a;
  --danger: #ef4444;
  --danger-hover: #dc2626;
  --shadow: rgba(0, 0, 0, 0.3);
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-primary);
}

.top-bar {
  padding: 0.875rem 1.5rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.brand-icon {
  font-size: 1.5rem;
}

.brand-name {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.brand-tagline {
  font-size: 0.8rem;
  color: var(--text-secondary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.top-bar-actions {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}

.error-banner {
  padding: 0.625rem 1.5rem;
  background: var(--bg-tertiary);
  color: var(--danger);
  font-size: 0.875rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.panes {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.chat-pane-wrapper {
  flex: 1;
  min-width: 320px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
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
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(2px);
}

.modal {
  background: var(--bg-secondary);
  padding: 1.75rem;
  border-radius: 12px;
  min-width: 420px;
  box-shadow: 0 20px 25px -5px var(--shadow), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.modal h3 {
  margin: 0 0 1.25rem 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 0.375rem;
}

.form-input {
  width: 100%;
  padding: 0.625rem 0.875rem;
  border: 1px solid var(--input-border);
  border-radius: 8px;
  font-size: 0.95rem;
  font-family: inherit;
  background: var(--input-bg);
}

.form-input:focus {
  outline: none;
  border-color: var(--accent);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  margin-top: 1rem;
}

.btn-cancel {
  padding: 0.5625rem 1.125rem;
  font-size: 0.9rem;
  background: var(--bg-muted);
  color: var(--text-secondary);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-cancel:hover {
  background: var(--border-hover);
}

.btn-create {
  padding: 0.5625rem 1.125rem;
  font-size: 0.9rem;
  background: var(--accent);
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-create:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-create:disabled {
  background: var(--border-hover);
  cursor: not-allowed;
}

.version-save-bar {
  padding: 0.75rem 1.5rem;
  background: var(--bg-primary);
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
}

.version-save-bar label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  font-size: 0.9rem;
  font-family: inherit;
  background: var(--input-bg);
}

.version-input:focus {
  outline: none;
  border-color: var(--accent);
}

.btn-save-version-bar {
  padding: 0.5rem 1rem;
  font-size: 0.9rem;
  background: var(--success);
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-save-version-bar:hover {
  background: var(--success-hover);
}

.provider-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.provider-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.provider-select {
  padding: 0.4rem 0.75rem;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  font-size: 0.9rem;
  background: var(--input-bg);
  color: var(--text-primary);
  min-width: 110px;
}

.provider-select:focus {
  outline: none;
  border-color: var(--accent);
}

.btn-api-key {
  padding: 0.5rem;
  background: #e2e8f0;
  color: #1a1a1a;
  border: 1px solid #6b7280;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
}

.btn-api-key:hover {
  background: #cbd5e1;
}

.btn-import, .btn-new {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: #e2e8f0;
  color: #1a1a1a;
  border: 1px solid #6b7280;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-import:hover, .btn-new:hover {
  background: #cbd5e1;
}

.btn-new {
  background: var(--accent);
  color: #ffffff;
}
.btn-new:hover {
  background: var(--accent-hover);
}

.btn-save-agent-header {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: var(--success);
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}
.btn-save-agent-header:hover {
  background: var(--success-hover);
}

.btn-home {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: #e2e8f0;
  color: #1a1a1a;
  border: 1px solid #6b7280;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}
.btn-home:hover {
  background: #cbd5e1;
}

.btn-dark-toggle {
  padding: 0.5rem;
  background: #e2e8f0;
  color: #1a1a1a;
  border: 1px solid #6b7280;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
}

.btn-dark-toggle:hover {
  background: #cbd5e1;
}
</style>
