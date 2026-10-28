<template>
  <div class="agent-canvas h-full flex flex-col bg-slate-950 relative">
    <div class="flex items-center justify-between px-3 py-2 bg-slate-900 border-b border-slate-800 shrink-0">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider">Canvas</span>
        <span
          v-if="isInSubGraph"
          class="px-1.5 py-0.5 rounded bg-indigo-600/20 text-indigo-300 text-xs border border-indigo-500/30"
        >
          Sub-graph
        </span>
      </div>

      <div class="flex items-center gap-2">
        <button
          v-if="isInSubGraph"
          @click="drillOut"
          class="px-3 py-1.5 text-xs font-medium text-slate-300 hover:text-white bg-slate-800/50 hover:bg-slate-800 rounded border border-slate-700 transition-colors"
          title="Drill out of sub-graph"
        >
          ← Back to parent
        </button>
        <button
          @click="validateGraph"
          class="px-3 py-1.5 text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded border border-indigo-500/30 transition-colors"
          title="Run logic check"
        >
          Validate
        </button>
        <span class="w-px h-3 bg-slate-700 mx-1"></span>
        <span :class="['text-xs font-mono', errorCount ? 'text-red-400' : 'text-green-400']">
          {{ errorCount }} errors
        </span>
      </div>
    </div>

    <div
      v-if="breadcrumbs.length > 0"
      class="px-3 py-1.5 bg-slate-900 border-b border-slate-800 shrink-0 overflow-hidden"
    >
      <div class="flex items-center gap-1 overflow-x-auto no-scrollbar">
        <span
          class="text-[10px] text-slate-500 cursor-pointer hover:text-slate-200"
          @click="clearBreadcrumbs"
        >
          Root
        </span>
        <span
          v-for="(crumb, idx) in breadcrumbs"
          :key="crumb.nodeId"
          class="text-[10px] text-slate-400"
        >
          <span class="text-slate-600">/</span>
          <span
            v-if="idx < breadcrumbs.length - 1"
            class="cursor-pointer hover:text-slate-200"
            @click="drillToLevel(idx)"
          >
            {{ crumb.name }}
          </span>
          <span v-else class="text-slate-200 font-medium">{{ crumb.name }}</span>
        </span>
      </div>
    </div>

    <div
      ref="canvasContainer"
      class="flex-1 overflow-hidden relative"
      @dragover.prevent="onDragOver"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
<VueFlow
  :nodes="flowNodes"
  :edges="flowEdges"
  fit-view-on-init
  pan-on-scroll
  pan-on-drag
  zoom-on-pan
  @connect="onConnect"
  @node-drag="onNodeDrag"
  @node-click="onNodeClick"
  @background-click="onBackgroundClick"
  :node-types="nodeTypes"
  :edge-types="edgeTypes"
>
        <div
          v-if="dragOver"
          class="absolute inset-0 bg-indigo-500/10 border-2 border-dashed border-indigo-500 flex items-center justify-center pointer-events-none z-10"
        >
          <span class="text-indigo-300 font-medium text-sm">Drop block here</span>
        </div>
      </VueFlow>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  VueFlow,
  type NodeDragEvent,
  type NodeMouseEvent,
  type Connection,
} from '@vue-flow/core';
import type { BlockDefinition } from '@/types/agent';
import { useGraphStore } from '@/stores/graphStore';
import { CanvasAdapter } from '@/services/canvasAdapter';
import AgentNode from '@/components/canvas/AgentNode.vue';
import StepEdge from '@/components/canvas/StepEdge.vue';

const store = useGraphStore();

const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);
const breadcrumbs = computed(() => store.breadcrumbs);
const isInSubGraph = computed(() => store.isInSubGraph);
const errorCount = computed(() => store.errorCount);

const flowNodes = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.nodes.map((node) => {
    const block = blocks.value.find((b) => b.id === node.blockId);
    const errors = store.getNodeErrors(node.id);
    const hasError = errors.some((e) => e.severity === 'error');

    return {
      id: node.id,
      type: 'agentNode',
      position: node.position,
      data: {
        label: block?.name ?? node.blockId,
        blockId: node.blockId,
        block,
        parameters: node.parameters,
        errors,
        hasError,
        isCollapsed: node.isCollapsed,
      },
      style: {
        background: blockColor(block?.kind ?? 'utility'),
        color: '#ffffff',
        border: hasError ? '2px solid #EF4444' : '1px solid rgba(255,255,255,0.2)',
      },
    };
  });
});

const flowEdges = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    sourceHandle: edge.sourceHandle,
    targetHandle: edge.targetHandle,
  }));
});

const canvasContainer = ref<HTMLDivElement | null>(null);
const dragOver = ref(false);

function onNodeDrag(event: NodeDragEvent) {
  const node = event.node;
  store.updateNodePosition(node.id, { x: node.position.x, y: node.position.y });
}

function onNodeClick(event: NodeMouseEvent) {
  store.selectNode(event.node.id);
}

function onBackgroundClick() {
  store.selectNode(null);
}

function onConnect(connection: Connection) {
  const allNodes = flowNodes.value;
  const sourceNode = allNodes.find((n: any) => n.id === connection.source);
  const targetNode = allNodes.find((n: any) => n.id === connection.target);

  if (!sourceNode || !targetNode) return;

  const sourceBlock = (sourceNode.data as any).block;
  const targetBlock = (targetNode.data as any).block;

  if (!sourceBlock || !targetBlock) return;

  const sourcePort = sourceBlock.outputs.find(
    (p: any) => p.id === connection.sourceHandle
  );
  const targetPort = targetBlock.inputs.find(
    (p: any) => p.id === connection.targetHandle
  );

  if (!sourcePort || !targetPort) return;

  const result = CanvasAdapter.canConnect(sourcePort, targetPort);
  if (!result.allowed) {
    console.warn(`Connection rejected: ${result.reason}`);
    return;
  }

  store.addEdge({
    source: connection.source,
    sourceHandle: connection.sourceHandle!,
    target: connection.target,
    targetHandle: connection.targetHandle!,
  });
}

function onDragEnter(event: DragEvent) {
  event.preventDefault();
  dragOver.value = true;
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  (event.dataTransfer as DataTransfer).dropEffect = 'copy';
  dragOver.value = true;
}

function onDragLeave() {
  dragOver.value = false;
}

async function onDrop(event: DragEvent) {
  event.preventDefault();
  dragOver.value = false;

  const data = event.dataTransfer?.getData('application/opifex-block');
  if (!data || !canvasContainer.value) return;

  const block: BlockDefinition = JSON.parse(data);
  const rect = canvasContainer.value.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  store.addNode({
    blockId: block.id,
    blockVersion: block.version,
    position: { x, y },
    parameters: {},
    subGraph: undefined,
    isCollapsed: false,
  });
}

function blockColor(kind: string): string {
  const colors: Record<string, string> = {
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

const nodeTypes = { agentNode: AgentNode as any };
const edgeTypes = { step: StepEdge as any };

function validateGraph() {
  store.validateGraph();
}

function drillOut() {
  store.drillOut();
}

function drillToLevel(level: number) {
  while (breadcrumbs.value.length > level + 1) {
    store.drillOut();
  }
}

function clearBreadcrumbs() {
  store.breadcrumbs = [];
}
</script>

<style scoped>
.agent-canvas {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
