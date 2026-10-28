<template>
  <div class="p-4 bg-slate-950 text-slate-200 h-full">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-xl font-bold text-white">S1 Canvas Spike</h1>
        <p class="text-sm text-slate-400">@vue-flow/core — 200-node drag latency test</p>
      </div>
      <div class="flex gap-2">
        <button
          @click="regenerate"
          class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 rounded text-white text-sm font-medium"
        >
          Regenerate 200 Nodes
        </button>
      </div>
    </div>

    <div class="flex gap-4">
      <div class="w-64 space-y-2">
        <div class="bg-slate-900 p-3 rounded border border-slate-800">
          <div class="text-xs text-slate-400 uppercase tracking-wider">Node Count</div>
          <div class="text-2xl font-mono text-green-400">{{ nodeCount }}</div>
        </div>
      </div>

      <div class="flex-1 h-[600px] bg-slate-900 rounded border border-slate-800 relative">
        <VueFlow
          v-model:nodes="flowNodes"
          v-model:edges="flowEdges"
          fit-view-on-init
        >
        </VueFlow>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { VueFlow } from '@vue-flow/core';

const nodeCount = ref(200);
const kinds = ['input', 'llm', 'agent', 'tool', 'control', 'output', 'utility'];
const colors = ['bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-orange-500', 'bg-red-500', 'bg-yellow-500', 'bg-pink-500'];

const flowNodes = ref<any[]>([]);
const flowEdges = ref<any[]>([]);

function regenerate() {
  flowNodes.value = [];
  flowEdges.value = [];

  for (let i = 0; i < nodeCount.value; i++) {
    const kind = kinds[i % kinds.length];
    const color = colors[i % colors.length];
    flowNodes.value.push({
      id: `node-${i}`,
      type: kind,
      position: { x: (i % 20) * 220, y: Math.floor(i / 20) * 140 },
      data: { label: `${kind.toUpperCase()} #${i}`, kind },
      style: { background: color, color: '#ffffff' },
    });
  }

  for (let i = 0; i < nodeCount.value - 1; i += 3) {
    flowEdges.value.push({
      id: `edge-${i}`,
      source: `node-${i}`,
      target: `node-${i + 1}`,
      sourceHandle: 'output',
      targetHandle: 'input',
    });
  }
}

regenerate();
</script>

<style scoped>
</style>
