<script setup lang="ts">
import { computed } from 'vue';
import { PropType } from 'vue';
import type { AgentListItem } from '@/types/agent';

const props = defineProps({
  agents: { type: Array as PropType<AgentListItem[]>, required: true },
  onSelect: { type: Function as PropType<(id: string) => void>, required: true },
  onCreate: { type: Function as PropType<() => void>, required: true },
  onRename: { type: Function as PropType<(id: string, newName: string) => void>, required: true },
  onDuplicate: { type: Function as PropType<(id: string, newName?: string) => void>, required: true },
  onExport: { type: Function as PropType<(id: string) => void>, required: true },
  onDelete: { type: Function as PropType<(id: string) => void>, required: true },
});

const starterAgents = computed(() => props.agents.filter(a => a.isStarter));
const userAgents = computed(() => props.agents.filter(a => !a.isStarter));
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

    <div v-else class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
      </div>
      <p>No agents yet. Create one to get started.</p>
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
</style>