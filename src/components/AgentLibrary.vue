<script setup lang="ts">
import { computed, ref } from 'vue';
import { PropType } from 'vue';
import type {
  AgentListItem,
  InstalledTemplate,
  TemplateEntry,
  TemplateSource,
} from '@/types/agent';

const props = defineProps({
  agents: { type: Array as PropType<AgentListItem[]>, required: true },
  onSelect: { type: Function as PropType<(id: string) => void>, required: true },
  onCreate: { type: Function as PropType<() => void>, required: true },
  onRename: { type: Function as PropType<(id: string, newName: string) => void>, required: true },
  onDuplicate: { type: Function as PropType<(id: string, newName?: string) => void>, required: true },
  onExport: { type: Function as PropType<(id: string) => void>, required: true },
  onDelete: { type: Function as PropType<(id: string) => void>, required: true },
  onAddTemplateSource: { type: Function as PropType<(name: string, url: string) => void>, required: false },
  onLoadTemplateSources: { type: Function as PropType<() => void>, required: false },
  onLoadMarketplaceTemplates: { type: Function as PropType<(sourceId: string) => void>, required: false },
  onInstallMarketplaceTemplate: { type: Function as PropType<(templateId: string, version: string, sourceId: string) => void>, required: false },
  onLoadInstalledTemplates: { type: Function as PropType<() => void>, required: false },
  templateSources: { type: Array as PropType<TemplateSource[]>, required: false, default: () => [] },
  installedTemplates: { type: Array as PropType<InstalledTemplate[]>, required: false, default: () => [] },
  marketplaceTemplates: { type: Array as PropType<TemplateEntry[]>, required: false, default: () => [] },
  marketplaceLoading: { type: Boolean as PropType<boolean>, required: false, default: false },
  selectedSourceId: { type: [String, null] as PropType<string | null>, required: false, default: null },
});

const starterAgents = computed(() => props.agents.filter(a => a.isStarter));
const userAgents = computed(() => props.agents.filter(a => !a.isStarter));

const selectedSource = ref<string | null>(props.selectedSourceId);
const showAddSource = ref(false);
const newSourceName = ref('');
const newSourceUrl = ref('');

const marketplaceSection = computed(() => {
  if (!selectedSource.value) return [];
  return props.marketplaceTemplates;
});

const isInstalled = (templateId: string, version: string) => {
  return props.installedTemplates.some(t => t.templateId === templateId && t.version === version);
};
</script>

<template>
  <div class="agent-library">
    <div class="library-header">
      <h3>Agent Library</h3>
      <button class="btn-new" @click="props.onCreate">+ New Agent</button>
    </div>

    <div class="library-section">
      <h4>Starter Agents</h4>
      <div class="agent-list">
        <div
          v-for="agent of starterAgents"
          :key="agent.id"
          class="agent-item"
          @click="props.onSelect(agent.id)"
        >
          <div class="agent-info">
            <div class="agent-name">{{ agent.name }}</div>
            <div class="agent-desc">{{ agent.description }}</div>
          </div>
          <div class="agent-badge">Starter</div>
          <button class="btn-clone" title="Clone starter" @click.stop="props.onDuplicate(agent.id)">⧉</button>
        </div>
      </div>
    </div>

    <div class="library-section" v-if="userAgents.length > 0">
      <h4>Your Agents</h4>
      <div class="agent-list">
        <div
          v-for="agent of userAgents"
          :key="agent.id"
          class="agent-item"
          @click="props.onSelect(agent.id)"
        >
          <div class="agent-info">
            <div class="agent-name">{{ agent.name }}</div>
            <div class="agent-desc">{{ agent.description }}</div>
          </div>
          <div class="agent-actions">
            <button class="btn-clone" title="Duplicate" @click.stop="props.onDuplicate(agent.id)">⧉</button>
            <button class="btn-export" title="Export" @click.stop="props.onExport(agent.id)">⬇</button>
            <button class="btn-delete" title="Delete" @click.stop="props.onDelete(agent.id)">×</button>
          </div>
        </div>
      </div>
    </div>

    <div class="library-section marketplace-section">
      <h4>📦 Template Marketplace</h4>

      <div class="source-selector">
        <label class="source-label">Catalog Source:</label>
        <select
          v-model="selectedSource"
          class="source-select"
          @change="() => { props.onLoadTemplateSources?.(); }"
        >
          <option v-for="source of props.templateSources" :key="source.id" :value="source.id">
            {{ source.name }} {{ source.enabled ? '' : '(disabled)' }}
          </option>
        </select>
        <button
          v-if="props.onAddTemplateSource"
          class="btn-add-source"
          @click="showAddSource = true"
          title="Add catalog source"
        >
          +
        </button>
      </div>

      <div v-if="selectedSource && props.onLoadMarketplaceTemplates" class="marketplace-controls">
        <button
          class="btn-refresh"
          @click="props.onLoadMarketplaceTemplates(selectedSource!)"
          :disabled="props.marketplaceLoading"
          title="Refresh templates"
        >
          {{ props.marketplaceLoading ? 'Loading...' : '↻' }}
        </button>
      </div>

      <div class="template-list">
        <div
          v-if="props.marketplaceLoading"
          class="empty-state"
        >
          <p>Loading templates…</p>
        </div>
        <div
          v-else-if="selectedSource && marketplaceSection.length === 0"
          class="empty-state"
        >
          <p>No templates available from this source.</p>
        </div>
        <div
          v-for="template of marketplaceSection"
          :key="template.id + template.version"
          class="template-item"
        >
          <div class="template-info">
            <div class="template-name">{{ template.name }}</div>
            <div class="template-desc">{{ template.description }}</div>
            <div class="template-meta">
              <span class="template-version">v{{ template.version }}</span>
              <span v-if="template.author" class="template-author">by {{ template.author }}</span>
              <span v-if="template.model" class="template-model">{{ template.model }}</span>
            </div>
          </div>
          <button
            class="btn-install"
            :disabled="isInstalled(template.id, template.version)"
            @click="() => {
              props.onInstallMarketplaceTemplate?.(template.id, template.version, selectedSource!);
            }"
          >
            {{ isInstalled(template.id, template.version) ? 'Installed' : 'Install' }}
          </button>
        </div>
      </div>

      <div v-if="props.installedTemplates.length > 0" class="installed-section">
        <h5>Installed Templates</h5>
        <div class="installed-list">
          <div
            v-for="tpl of props.installedTemplates"
            :key="tpl.id"
            class="installed-item"
          >
            <div class="installed-info">
              <div class="installed-name">{{ tpl.templateId }}</div>
              <div class="installed-version">v{{ tpl.version }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="!props.templateSources.length && !userAgents.length" class="empty-state">
      <p>No agents yet. Create one to get started.</p>
    </div>

    <!-- Add Source Dialog -->
    <div v-if="showAddSource" class="dialog-overlay" @click="showAddSource = false">
      <div class="dialog" @click.stop>
        <h4>Add Template Source</h4>
        <div class="form-group">
          <label>Name:</label>
          <input v-model="newSourceName" type="text" placeholder="e.g. Community Templates" />
        </div>
        <div class="form-group">
          <label>Catalog URL:</label>
          <input v-model="newSourceUrl" type="text" placeholder="https://example.com/catalog.json" />
        </div>
        <div class="dialog-actions">
          <button class="btn-cancel" @click="showAddSource = false">Cancel</button>
          <button
            class="btn-save"
            @click="() => {
              showAddSource = false;
              if (props.onAddTemplateSource && newSourceName.trim() && newSourceUrl.trim()) {
                props.onAddTemplateSource(newSourceName.trim(), newSourceUrl.trim());
              }
              newSourceName = '';
              newSourceUrl = '';
            }"
          >
            Add Source
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-library {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 1.5rem 1rem 1.5rem;
}

.library-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.library-section {
  padding: 0 1.5rem;
}

.library-section h4 {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 1.5rem 0 0.75rem 0;
}

.library-section h4:first-of-type {
  margin-top: 0;
}

.marketplace-section {
  border-top: 1px solid var(--border);
  padding-top: 1rem;
}

.source-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
}

.source-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.source-select {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 0.85rem;
  min-width: 200px;
}

.source-select option {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-add-source {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-muted);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-add-source:hover {
  background: var(--border-hover);
}

.marketplace-controls {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.btn-refresh {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-muted);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.85rem;
}
.btn-refresh:hover {
  background: var(--border-hover);
}
.btn-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.template-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 300px;
  overflow-y: auto;
  padding-right: 0.25rem;
}

.template-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.875rem 1rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-secondary);
  transition: all 0.15s ease;
}

.template-item:hover {
  border-color: var(--accent);
  background: var(--bg-tertiary);
}

.template-info {
  flex: 1;
}

.template-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}

.template-desc {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-top: 0.125rem;
}

.template-meta {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.375rem;
}

.template-version,
.template-author,
.template-model {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--text-muted);
  background: var(--bg-muted);
  padding: 0.125rem 0.5rem;
  border-radius: 8px;
}

.btn-install {
  padding: 0.5rem 1rem;
  border: 1px solid var(--accent);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--accent);
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 500;
  white-space: nowrap;
}
.btn-install:hover:not(:disabled) {
  background: var(--accent);
  color: #ffffff;
}
.btn-install:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: var(--border);
  color: var(--text-muted);
}

.installed-section {
  margin-top: 1rem;
}

.installed-section h5 {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
}

.installed-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.installed-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.625rem 0.875rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-secondary);
}

.installed-info {
  display: flex;
  flex-direction: column;
}

.installed-name {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-primary);
}

.installed-version {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.agent-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.875rem 1rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  background: var(--bg-secondary);
  transition: all 0.15s ease;
}

.agent-item:hover {
  border-color: var(--accent);
  background: var(--bg-tertiary);
  transform: translateY(-1px);
}

.agent-info {
  flex: 1;
}

.agent-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}

.agent-desc {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-top: 0.125rem;
}

.agent-badge {
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--accent);
  background: rgba(59, 130, 246, 0.1);
  padding: 0.2rem 0.6rem;
  border-radius: 12px;
  margin-left: 0.5rem;
}

.agent-actions {
  display: flex;
  gap: 0.375rem;
}

.btn-clone, .btn-export, .btn-delete {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-clone {
  background: var(--bg-muted);
  color: var(--text-secondary);
}
.btn-clone:hover {
  background: var(--border-hover);
}

.btn-export {
  background: var(--bg-muted);
  color: var(--text-secondary);
}
.btn-export:hover {
  background: var(--border-hover);
}

.btn-delete {
  background: transparent;
  color: var(--danger);
}
.btn-delete:hover {
  background: rgba(239, 68, 68, 0.1);
}

.btn-new {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: var(--bg-muted);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}
.btn-new:hover {
  background: var(--bg-tertiary);
}

.empty-state {
  padding: 2rem 1.5rem;
  color: var(--text-muted);
  font-size: 0.9rem;
  text-align: center;
}

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.5rem;
  width: 400px;
  max-width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
}

.dialog h4 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 0.375rem;
}

.form-group input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  background: var(--input-bg);
  color: var(--text-primary);
  font-size: 0.85rem;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1rem;
}

.btn-cancel {
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-muted);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 0.85rem;
}
.btn-cancel:hover {
  background: var(--border-hover);
}

.btn-save {
  padding: 0.5rem 1rem;
  border: 1px solid var(--accent);
  border-radius: 6px;
  background: var(--accent);
  color: #ffffff;
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 500;
}
.btn-save:hover {
  opacity: 0.9;
}
</style>