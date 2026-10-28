<template>
  <div
    class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col"
  >
    <h3
      class="text-sm font-semibold mb-2 flex items-center justify-between"
    >
      <span>Attached Tools (Hybrid Wiring)</span>
      <span class="text-xs text-slate-400">{{ attachedTools.length }} active</span>
    </h3>

    <div v-if="availableTools.length === 0" class="flex-1 flex items-center justify-center">
      <p class="text-slate-500 text-sm">No tool nodes available on canvas</p>
    </div>

    <div
      v-else
      class="flex-1 overflow-y-auto space-y-2"
    >
      <div
        v-for="tool in availableTools"
        :key="tool.id"
        class="flex items-center justify-between p-2.5 rounded bg-slate-800 border border-slate-700 hover:border-slate-600 transition-colors"
      >
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-blue-500"></span>
          <span class="text-sm font-medium truncate max-w-[200px]">{{ toolBlockName(tool.id) }}</span>
        </div>
        <button
          @click="toggleToolAttachment(tool.id)"
          :class="
            isAttached(tool.id)
              ? 'bg-red-900/50 text-red-300 hover:bg-red-900/80'
              : 'bg-indigo-900/50 text-indigo-300 hover:bg-indigo-900/80'
          "
          class="px-2 py-1 text-xs rounded font-mono transition-colors"
          :aria-pressed="isAttached(tool.id)"
          :title="isAttached(tool.id) ? 'Detach tool' : 'Attach tool'"
        >
          {{ isAttached(tool.id) ? 'Detach' : 'Attach' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useGraphStore } from '@/stores/graphStore';

const store = useGraphStore();

const props = defineProps<{
  nodeId: string;
}>();

const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);

const availableTools = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.nodes.filter(
    (n) => blocks.value.find((b) => b.id === n.blockId)?.kind === 'tool'
  );
});

const attachedTools = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.edges
    .filter((e) => e.target === props.nodeId)
    .map((e) => g.nodes.find((n) => n.id === e.source)!)
    .filter(Boolean);
});

function toolBlockName(nodeId: string): string {
  const g = graph.value;
  if (!g) return 'Unknown';
  const node = g.nodes.find((n) => n.id === nodeId);
  if (!node) return 'Unknown';
  const block = blocks.value.find((b) => b.id === node.blockId);
  return block?.name ?? node.blockId;
}

function isAttached(toolNodeId: string): boolean {
  return attachedTools.value.some((t) => t.id === toolNodeId);
}

function toggleToolAttachment(toolNodeId: string) {
  const g = graph.value;
  if (!g) return;

  if (isAttached(toolNodeId)) {
    const edge = g.edges.find(
      (e) => e.source === toolNodeId && e.target === props.nodeId
    );
    if (edge) {
      store.removeEdge(edge.id);
    }
  } else {
    const toolNode = g.nodes.find((n) => n.id === toolNodeId);
    if (!toolNode) return;

    const targetNode = g.nodes.find((n) => n.id === props.nodeId);
    if (!targetNode) return;

    const toolBlock = blocks.value.find((b) => b.id === toolNode.blockId);
    const targetBlock = blocks.value.find((b) => b.id === targetNode.blockId);

    if (!toolBlock || !targetBlock) return;

    const toolOutput = toolBlock.outputs.find((p) => p.direction === 'output');
    const targetInput = targetBlock.inputs.find((p) => p.direction === 'input');

    if (!toolOutput || !targetInput) return;

    store.addEdge({
      source: toolNodeId,
      sourceHandle: toolOutput.id,
      target: props.nodeId,
      targetHandle: targetInput.id,
    });
  }
}
</script>

<style scoped>
</style>
