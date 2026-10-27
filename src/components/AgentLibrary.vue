<script setup lang="ts">
import { computed, ref } from 'vue';
import { PropType } from 'vue';
import type { AgentListItem, TemplateEntry, TemplateSource, InstalledTemplate } from '@/types/agent';

const props = defineProps({
  agents: { type: Array as PropType<AgentListItem[]>, required: true },
  templateSources: { type: Array as PropType<TemplateSource[]>, required: false, default: () => [] },
  installedTemplates: { type: Array as PropType<InstalledTemplate[]>, required: false, default: () => [] },
  marketplaceTemplates: { type: Array as PropType<TemplateEntry[]>, required: false, default: () => [] },
  marketplaceLoading: { type: Boolean as PropType<boolean>, required: false, default: false },
  selectedSourceId: { type: [String, null] as PropType<string | null>, required: false, default: null },
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
      <button class="btn-new" @click="props.onCreate">New Agent</button>
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
          <button class="btn-clone" title="Clone starter" @click.stop="props.onDuplicate(agent.id)">
            Clone
          </button>
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
            <button class="btn-clone" title="Duplicate" @click.stop="props.onDuplicate(agent.id)">
              Clone
            </button>
            <button class="btn-export" title="Export" @click.stop="props.onExport(agent.id)">
              Export
            </button>
            <button class="btn-delete" title="Delete" @click.stop="props.onDelete(agent.id)">
              Delete
            </button>
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
          {{ props.marketplaceLoading ? 'Loading…' : '↻' }}
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
    <div v-if="showAddSource" class="dialog-overlay" @click.self="showAddSource = false">
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
.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 var(--space-6) var(--space-4) var(--space-6);
}

.library-header h3 {
  margin: 0;
  font-size: var(--type-xl);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.library-section {
  padding: 0 var(--space-6);
}

.library-section h4 {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: var(--space-6) 0 var(--space-3) 0;
}

.library-section h4:first-of-type {
  margin-top: 0;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.agent-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  background: var(--surface-card);
  transition: all 0.15s ease;
}

.agent-item:hover {
  border-color: var(--border-hover);
  background: var(--surface-hover);
  transform: translateY(-1px);
}

.agent-info {
  flex: 1;
}

.agent-name {
  font-size: var(--type-base);
  font-weight: 600;
  color: var(--text-primary);
}

.agent-desc {
  font-size: var(--type-sm);
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

.agent-badge {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--primary);
  background: var(--primary-bg);
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-sm);
  margin-left: var(--space-2);
}

.agent-actions {
  display: flex;
  gap: var(--space-2);
}

.btn-clone, .btn-export, .btn-delete {
  padding: var(--space-1) var(--space-2);
  font-size: var(--type-xs);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s, border-color 0.2s;
  white-space: nowrap;
}

.btn-clone {
  background: var(--surface-card);
  color: var(--text-secondary);
}

.btn-clone:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-export {
  background: var(--surface-card);
  color: var(--text-secondary);
}

.btn-export:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-delete {
  background: var(--danger-bg);
  color: var(--danger-fg);
  border-color: var(--danger);
}

.btn-delete:hover {
  background: var(--danger);
  color: #ffffff;
  border-color: var(--danger);
}

.btn-new {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s, border-color 0.2s;
}

.btn-new:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-6);
  color: var(--text-muted);
  text-align: center;
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
  margin-bottom: var(--space-3);
}

.empty-state p {
  font-size: var(--type-base);
  margin: 0;
}

/* Marketplace section */
.marketplace-section {
  border-top: 1px solid var(--border);
  padding-top: var(--space-4) !important;
}

.source-selector {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
  flex-wrap: wrap;
}

.source-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
}

.source-select {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  color: var(--text-primary);
  font-size: var(--type-sm);
  min-width: 200px;
}

.source-select option {
  background: var(--surface-card);
  color: var(--text-primary);
}

.btn-add-source {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
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
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.btn-refresh {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-muted);
  color: var(--text-primary);
  cursor: pointer;
  font-size: var(--type-sm);
}

.btn-refresh:hover:not(:disabled) {
  background: var(--surface-hover);
}

.btn-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.template-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  max-height: 300px;
  overflow-y: auto;
  padding-right: var(--space-1);
}

.template-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-secondary);
  transition: all 0.15s ease;
}

.template-item:hover {
  border-color: var(--border-hover);
  background: var(--surface-hover);
}

.template-info {
  flex: 1;
}

.template-name {
  font-size: var(--type-base);
  font-weight: 600;
  color: var(--text-primary);
}

.template-desc {
  font-size: var(--type-sm);
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

.template-meta {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.template-version,
.template-author,
.template-model {
  font-size: var(--type-xs);
  font-weight: 500;
  color: var(--text-muted);
  background: var(--surface-muted);
  padding: 0.125rem var(--space-2);
  border-radius: var(--radius-sm);
}

.btn-install {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  border: 1px solid var(--primary);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  color: var(--primary);
  cursor: pointer;
  font-weight: 500;
  white-space: nowrap;
}

.btn-install:hover:not(:disabled) {
  background: var(--primary);
  color: var(--primary-fg);
}

.btn-install:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: var(--border);
  color: var(--text-muted);
}

.installed-section {
  margin-top: var(--space-4);
}

.installed-section h5 {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-2);
}

.installed-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.installed-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
}

.installed-info {
  display: flex;
  flex-direction: column;
}

.installed-name {
  font-size: var(--type-base);
  font-weight: 600;
  color: var(--text-primary);
}

.installed-version {
  font-size: var(--type-sm);
  color: var(--text-secondary);
}

/* Dialog overlay */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(22, 24, 27, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.dialog {
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  width: 400px;
  max-width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
}

.dialog h4 {
  margin: 0 0 var(--space-4) 0;
  font-size: var(--type-lg);
  font-weight: 600;
  color: var(--text-primary);
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
}

.form-group input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  color: var(--text-primary);
  font-size: var(--type-sm);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.btn-cancel {
  padding: var(--space-2) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-muted);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: var(--type-sm);
}

.btn-cancel:hover {
  background: var(--border-hover);
}

.btn-save {
  padding: var(--space-2) var(--space-4);
  border: 1px solid var(--primary);
  border-radius: var(--radius-md);
  background: var(--primary);
  color: var(--primary-fg);
  cursor: pointer;
  font-size: var(--type-sm);
  font-weight: 500;
}

.btn-save:hover {
  opacity: 0.9;
}
</style>