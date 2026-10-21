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

    <div v-else class="empty-state">
      <p>No agents yet. Create one to get started.</p>
    </div>
  </div>
</template>

<style scoped>
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
</style>
