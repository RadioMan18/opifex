<template>
  <div class="p-4 bg-slate-950 text-slate-200 h-full">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-xl font-bold text-white">S1 Canvas Spike</h1>
        <p class="text-sm text-slate-400">@vue-flow/core — 200-node drag latency test</p>
      </div>
      <div class="flex gap-2">
        <button @click="regenerate" class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 rounded text-white text-sm font-medium">
          Regenerate 200 Nodes
        </button>
        <button @click="runDragLatencyTest" class="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded text-white text-sm font-medium">
          Measure Drag Latency
        </button>
      </div>
    </div>

    <div class="flex gap-4">
      <!-- Metrics -->
      <div class="w-64 space-y-2">
        <div class="bg-slate-900 p-3 rounded border border-slate-800">
          <div class="text-xs text-slate-400 uppercase tracking-wider">Node Count</div>
          <div class="text-2xl font-mono text-green-400">{{ nodeCount }}</div>
        </div>
        <div class="bg-slate-900 p-3 rounded border border-slate-800">
          <div class="text-xs text-slate-400 uppercase tracking-wider">Avg Drag Frame</div>
          <div class="text-2xl font-mono text-yellow-400">{{ avgDragMs }} ms</div>
        </div>
        <div class="bg-slate-900 p-3 rounded border border-slate-800">
          <div class="text-xs text-slate-400 uppercase tracking-wider">60fps Target</div>
          <div class="text-2xl font-mono text-green-400">{{ meetsTarget ? 'PASS' : 'FAIL' }}</div>
        </div>
      </div>

      <!-- Canvas -->
      <div class="flex-1 h-[600px] bg-slate-900 rounded border border-slate-800 relative">
        <VueFlow
          v-model:nodes="nodes"
          v-model:edges="edges"
          fit-view-on-init
          :pan-on-scroll="true"
          :pan-on-drag="true"
          :zoom-on-pan="true"
          @connect="onConnect"
          @node-drag-start="onNodeDragStart"
          @node-drag-end="onNodeDragEnd"
        >
          <Background variant="dots" gap={16} size={2} offset={0} />
          <Controls />
          <MiniMap />
        </VueFlow>
        <div v-if="isDragging" class="absolute top-4 left-4 bg-slate-800 border border-slate-600 text-slate-300 px-3 py-1 rounded text-xs font-mono">
          Dragging: {{ draggedNodeId }}
        </div>
      </div>
    </div>

    <!-- Log -->
    <div class="mt-4 bg-black/50 p-3 rounded border border-slate-800 text-xs font-mono text-slate-400 overflow-h-auto max-h-32">
      <div v-for="(log, i) in logs" :key="i" class="mb-1">{{ log }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { VueFlow, useVueFlow, Background, Controls, MiniMap } from '@vue-flow/core';

const nodeCount = ref(200);
const avgDragMs = ref(0);
const meetsTarget = ref(false);
const isDragging = ref(false);
const draggedNodeId = ref('');
const logs = ref<string[]>([]);

const { addNodes, addEdges } = useVueFlow();
const nodes = ref<any[]>([]);
const edges = ref<any[]>([]);

const dragTimes = ref<number[]>([]);
let dragStartTime: number | null = null;

function log(msg: string) {
  logs.value.unshift(`[${new Date().toISOString().slice(11, 19)}] ${msg}`);
  if (logs.value.length > 50) logs.value.pop();
}

function regenerate() {
  dragTimes.value = [];
  avgDragMs.value = 0;
  meetsTarget.value = false;
  nodes.value = [];
  edges.value = [];

  const kinds = ['input', 'llm', 'agent', 'tool', 'control', 'output', 'utility'];
  const colors = ['bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-orange-500', 'bg-red-500', 'bg-yellow-500', 'bg-pink-500'];

  for (let i = 0; i < nodeCount.value; i++) {
    const kind = kinds[i % kinds.length];
    const color = colors[i % colors.length];
    nodes.value.push({
      id: `node-${i}`,
      type: kind,
      position: { x: (i % 20) * 220, y: Math.floor(i / 20) * 140 },
      data: { label: `${kind.toUpperCase()} #${i}`, kind },
      style: { background: color, color: '#ffffff' },
    });
  }

  // Create some edges
  for (let i = 0; i < nodeCount.value - 1; i += 3) {
    edges.value.push({
      id: `edge-${i}`,
      source: `node-${i}`,
      target: `node-${i + 1}`,
      sourceHandle: 'output',
      targetHandle: 'input',
    });
  }

  log(`Generated ${nodeCount.value} nodes and ${edges.value.length} edges`);
}

function onNodeDragStart(event: any, node: any) {
  isDragging.value = true;
  draggedNodeId.value = node.id;
  dragStartTime = performance.now();
}

function onNodeDragEnd(event: any, node: any) {
  isDragging.value = false;
  draggedNodeId.value = '';
  if (dragStartTime !== null) {
    const elapsed = performance.now() - dragStartTime;
    dragTimes.value.push(elapsed);
    if (dragTimes.value.length > 20) dragTimes.value.shift();
    avgDragMs.value = dragTimes.value.reduce((a, b) => a + b, 0) / dragTimes.value.length;
    meetsTarget.value = avgDragMs.value < 16;
    dragStartTime = null;
    log(`Drag end: ${elapsed.toFixed(2)}ms, avg: ${avgDragMs.value.toFixed(2)}ms, 60fps: ${meetsTarget.value}`);
  }
}

function onConnect(params: any) {
  edges.value.push({ id: `edge-${Date.now()}`, ...params });
  log(`Connected: ${params.source} -> ${params.target}`);
}

function runDragLatencyTest() {
  log('Starting drag latency test — drag a node and release to measure frame time');
}

regenerate();
</script>

<style scoped>
/* Spike component styles */
</style>
