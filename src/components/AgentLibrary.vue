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
  color: #1a1a1a;
  letter-spacing: -0.02em;
}

.library-section {
  padding: 0 1.5rem;
}

.library-section h4 {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
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
  border: 1px solid #e5e5e5;
  border-radius: 10px;
  cursor: pointer;
  background: #ffffff;
  transition: all 0.15s ease;
}

.agent-item:hover {
  border-color: #0066cc;
  background: #f0f7ff;
  transform: translateY(-1px);
}

.agent-info {
  flex: 1;
}

.agent-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: #1a1a1a;
}

.agent-desc {
  font-size: 0.85rem;
  color: #6b7280;
  margin-top: 0.125rem;
}

.agent-badge {
  font-size: 0.7rem;
  font-weight: 600;
  color: #0066cc;
  background: #eef4ff;
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
  background: #f3f4f6;
  color: #374151;
}
.btn-clone:hover {
  background: #e5e7eb;
}

.btn-export {
  background: #f3f4f6;
  color: #374151;
}
.btn-export:hover {
  background: #e5e7eb;
}

.btn-delete {
  background: transparent;
  color: #dc2626;
}
.btn-delete:hover {
  background: #fef2f2;
}

.btn-new {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: #0066cc;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-new:hover {
  background: #0052a3;
}

.empty-state {
  padding: 2rem 1.5rem;
  color: #9ca3af;
  font-size: 0.9rem;
  text-align: center;
}
</style>
