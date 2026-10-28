<template>
  <div class="block-palette h-full flex flex-col bg-slate-900 border-r border-slate-800 text-slate-200">
    <div class="p-3 border-b border-slate-800 flex items-center gap-2">
      <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider">Blocks</span>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search blocks..."
        class="flex-1 bg-slate-950 border border-slate-700 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
        aria-label="Search block palette"
      />
    </div>

    <div class="flex-1 overflow-y-auto p-2 space-y-4">
      <div
        v-for="category in categorizedBlocks"
        :key="category.category"
        class="space-y-2"
      >
        <h3 class="px-3 py-1.5 text-xs font-bold text-slate-500 uppercase tracking-wider">
          {{ category.category }}
        </h3>

        <div
          v-for="block in category.blocks"
          :key="block.id"
          class="group cursor-pointer select-none"
          draggable="true"
          @dragstart="onDragStart(block)"
          @dragend="onDragEnd"
        >
          <div class="p-2.5 rounded bg-slate-800/50 border border-slate-700/50 hover:bg-slate-800 hover:border-indigo-500/50 transition-all duration-150">
            <div class="flex items-center gap-2 mb-1">
              <span class="w-2 h-2 rounded-full" :style="{ backgroundColor: blockColor(block.kind) }"></span>
              <span class="text-sm font-medium text-slate-200 truncate">{{ block.name }}</span>
            </div>
            <p class="text-[10px] text-slate-400 truncate">{{ block.description || '' }}</p>
            <div class="flex items-center gap-1 mt-1.5">
              <span class="text-[9px] text-slate-500 uppercase tracking-tighter">{{ block.kind }}</span>
              <span
                v-for="port in block.inputs.slice(0, 2)"
                :key="port.id"
                class="w-1.5 h-1.5 rounded-full bg-blue-400"
                title="Input"
              />
              <span
                v-for="port in block.outputs.slice(0, 2)"
                :key="port.id"
                class="w-1.5 h-1.5 rounded-full bg-green-400"
                title="Output"
              />
            </div>
          </div>
        </div>
      </div>

      <div v-if="filteredBlocks.length === 0" class="p-4 text-center text-slate-500 text-sm">
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

function onDragStart(block: BlockDefinition) {
  emit('block-drag-start', block);
}

function onDragEnd() {
  // No-op for now; drag visual reset handled by browser
}
</script>

<style scoped>
.block-palette {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* Custom scrollbar */
.block-palette ::-webkit-scrollbar {
  width: 8px;
}
.block-palette ::-webkit-scrollbar-track {
  background: transparent;
}
.block-palette ::-webkit-scrollbar-thumb {
  background: #374151;
  border-radius: 4px;
}
.block-palette ::-webkit-scrollbar-thumb:hover {
  background: #4B5563;
}
</style>
