<template>
  <div class="block-palette h-full flex flex-col bg-[var(--surface-card)] border-r border-[var(--border)] text-[var(--text-primary)]">
    <div class="palette-header">
      <span class="palette-title">Block Palette</span>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search blocks..."
        class="palette-search"
        aria-label="Search block palette"
      />
    </div>

    <div class="palette-content">
      <div
        v-for="category in categorizedBlocks"
        :key="category.category"
        class="category-group"
      >
        <h3 class="category-header">{{ category.category }}</h3>

        <div
          v-for="block in category.blocks"
          :key="block.id"
          class="block-item"
          draggable="true"
          @dragstart="(e) => { onDragStart(e, block); $emit('block-drag-start', block); }"
          @dragend.stop
          style="cursor: grab; touch-action: none; user-select: none;"
        >
          <div class="block-card">
            <div class="block-card-header">
              <span class="block-kind-dot" :style="{ backgroundColor: blockColor(block.kind) }"></span>
              <span class="block-name">{{ block.name }}</span>
            </div>
            <p class="block-desc">{{ block.description || '' }}</p>
            <div class="block-meta">
              <span class="block-kind-label">{{ block.kind }}</span>
              <div class="block-ports">
                <span
                  v-for="port in block.inputs.slice(0, 2)"
                  :key="port.id"
                  class="port-dot port-dot--in"
                  title="Input"
                />
                <span
                  v-for="port in block.outputs.slice(0, 2)"
                  :key="port.id"
                  class="port-dot port-dot--out"
                  title="Output"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="filteredBlocks.length === 0" class="empty-blocks">
        No blocks found
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { BlockDefinition, BlockKind } from '@/types/agent';

const props = defineProps<{
  blocks: BlockDefinition[];
}>();

const emit = defineEmits<{
  (e: 'block-drag-start', block: BlockDefinition): void;
}>();

const searchQuery = ref('');

const filteredBlocks = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();
  if (!query) return props.blocks;
  return props.blocks.filter(
    (b) =>
      b.name.toLowerCase().includes(query) ||
      b.category.toLowerCase().includes(query) ||
      b.description?.toLowerCase().includes(query)
  );
});

const categorizedBlocks = computed(() => {
  const groups = new Map<string, BlockDefinition[]>();
  for (const block of filteredBlocks.value) {
    const existing = groups.get(block.category) ?? [];
    existing.push(block);
    groups.set(block.category, existing);
  }
  return Array.from(groups.entries()).map(([category, blocks]) => ({
    category,
    blocks,
  }));
});

function blockColor(kind: BlockKind): string {
  const colors: Record<BlockKind, string> = {
    trigger: '#10B981',
    llm: '#3B82F6',
    agent: '#8B5CF6',
    tool: '#F59E0B',
    control: '#EF4444',
    output: '#10B981',
    utility: '#6B7280',
    subgraph: '#A855F7',
  };
  return colors[kind] ?? '#6B7280';
}

function onDragStart(event: DragEvent, block: BlockDefinition) {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData('application/opifex-block', JSON.stringify(block));
    event.dataTransfer.setData('text/plain', block.name);
  }
  emit('block-drag-start', block);
}
</script>

<style scoped>
.block-palette {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.palette-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  background: var(--surface-hover);
}

.palette-title {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.palette-search {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-sm);
  color: var(--text-primary);
  font-family: inherit;
}

.palette-search:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.palette-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.category-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.category-header {
  padding: var(--space-1) var(--space-2);
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.block-item {
  cursor: grab;
  user-select: none;
}

.block-item:active {
  cursor: grabbing;
}

.block-card {
  padding: var(--space-3);
  background: var(--surface-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  transition: all 0.15s ease;
}

.block-card:hover {
  background: var(--surface-hover);
  border-color: var(--primary);
  box-shadow: 0 0 0 1px rgba(59, 130, 246, 0.2);
}

.block-card-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.block-kind-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.block-name {
  font-size: var(--type-sm);
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.block-desc {
  font-size: var(--type-xs);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 0;
}

.block-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: var(--space-2);
}

.block-kind-label {
  font-size: 10px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.block-ports {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.port-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.port-dot--in {
  background: #3B82F6;
}

.port-dot--out {
  background: #22C55E;
}

.empty-blocks {
  padding: var(--space-6);
  text-align: center;
  color: var(--text-muted);
  font-size: var(--type-sm);
}

/* Custom scrollbar */
.palette-content::-webkit-scrollbar {
  width: 8px;
}
.palette-content::-webkit-scrollbar-track {
  background: transparent;
}
.palette-content::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 4px;
}
.palette-content::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}
</style>
