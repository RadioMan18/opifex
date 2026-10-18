<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useAgentStore } from '@/stores/agentStore';
import ChatPane from '@/components/ChatPane.vue';
import PlanPane from '@/components/PlanPane.vue';
import AgentLibrary from '@/components/AgentLibrary.vue';
import ModelSelector from '@/components/ModelSelector.vue';
import type { AgentPlan } from '@/types/agent';

const store = useAgentStore();
const {
  state,
  init,
  selectAgent,
  createNewAgent,
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
const streamingMsgId = ref<string | null>(null);
const showImportDialog = ref(false);
const importJson = ref('');
const showRenameDialog = ref(false);
const renameId = ref<string | null>(null);
const renameName = ref('');
const showProviderDialog = ref(false);
const newVersionLabel = ref('');

onMounted(() => {
  init();
});

const currentAgent = computed(() => state.currentAgent);
const currentPlan = computed(() => state.currentPlan);
const messages = computed(() => state.messages);
const models = computed(() => state.models);
const defaultModel = computed(() => state.defaultModel);
const loading = computed(() => state.loading);
const streaming = computed(() => state.streaming);
const error = computed(() => state.error);
const provider = computed(() => state.provider);
const apiKeys = computed(() => state.apiKeys);
const planVersions = computed(() => state.planVersions);
const interviewing = computed(() => state.interviewing);
const buildError = computed(() => state.buildError);

async function handleSelectAgent(id: string) {
  await selectAgent(id);
}

async function handleSendMessage(content: string) {
  await sendMessage(content);
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

async function handleExportAgent(id: string) {
  try {
    const json = await exportAgent(id);
    const fileName = prompt('Save agent as:', `${currentAgent.value?.name || 'agent'}.json`);
    if (fileName) {
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  } catch (err: any) {
    state.error = err?.message || String(err);
  }
}

async function handleImportAgent() {
  if (!importJson.value.trim()) return;
  const agent = await importAgent(importJson.value);
  importJson.value = '';
  showImportDialog.value = false;
  if (agent) {
    // Select the newly imported agent
    await selectAgent(agent.id);
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
          class="btn-home"
          @click="goHome"
          title="Return to library"
        >
          ↥ Home
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
        @update="handlePlanUpdate"
        @save-version="handleSaveVersion"
        @restore-version="handleRestoreVersion"
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
          <label>Agent JSON</label>
          <textarea
            v-model="importJson"
            placeholder="Paste agent JSON here..."
            class="form-input"
            rows="6"
          ></textarea>
        </div>
        <div class="form-actions">
          <button class="btn-cancel" @click="showImportDialog = false">Cancel</button>
          <button class="btn-create" @click="handleImportAgent" :disabled="!importJson.trim()">
            Import
          </button>
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
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: #f8fafc;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
}

.top-bar {
  padding: 0.875rem 1.5rem;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
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
  color: #1a1a1a;
  letter-spacing: -0.02em;
}

.brand-tagline {
  font-size: 0.8rem;
  color: #6b7280;
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
  background: #fef2f2;
  color: #991b1b;
  font-size: 0.875rem;
  border-bottom: 1px solid #fee2e2;
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
  background: #ffffff;
  border-right: 1px solid #e2e8f0;
  overflow: hidden;
}

.divider {
  width: 1px;
  background: #e2e8f0;
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
  background: #ffffff;
  padding: 1.75rem;
  border-radius: 12px;
  min-width: 420px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.modal h3 {
  margin: 0 0 1.25rem 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: #1a1a1a;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: #374151;
  margin-bottom: 0.375rem;
}

.form-input {
  width: 100%;
  padding: 0.625rem 0.875rem;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 0.95rem;
  font-family: inherit;
}

.form-input:focus {
  outline: none;
  border-color: #0066cc;
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
  background: #f3f4f6;
  color: #374151;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-cancel:hover {
  background: #e5e7eb;
}

.btn-create {
  padding: 0.5625rem 1.125rem;
  font-size: 0.9rem;
  background: #0066cc;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-create:hover:not(:disabled) {
  background: #0052a3;
}

.btn-create:disabled {
  background: #9ca3af;
  cursor: not-allowed;
}

.version-save-bar {
  padding: 0.75rem 1.5rem;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
}

.version-save-bar label {
  font-size: 0.8rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 0.9rem;
  font-family: inherit;
}

.version-input:focus {
  outline: none;
  border-color: #0066cc;
}

.btn-save-version-bar {
  padding: 0.5rem 1rem;
  font-size: 0.9rem;
  background: #10b981;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-save-version-bar:hover {
  background: #059669;
}

.provider-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.provider-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.provider-select {
  padding: 0.4rem 0.75rem;
  border: 1px solid #d1d1d1;
  border-radius: 6px;
  font-size: 0.9rem;
  background: #ffffff;
  min-width: 110px;
}

.provider-select:focus {
  outline: none;
  border-color: #0066cc;
}

.btn-api-key {
  padding: 0.5rem;
  background: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d1d1;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
}

.btn-api-key:hover {
  background: #e5e7eb;
}

.btn-import, .btn-new {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-import {
  background: #f3f4f6;
  color: #374151;
}
.btn-import:hover {
  background: #e5e7eb;
}

.btn-new {
  background: #0066cc;
  color: #ffffff;
}
.btn-new:hover {
  background: #0052a3;
}

.btn-home {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: #f3f4f6;
  color: #374151;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-home:hover {
  background: #e5e7eb;
}
</style>
