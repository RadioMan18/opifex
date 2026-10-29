<template>
  <div class="agent-canvas h-full flex flex-col bg-slate-950 relative">
    <!-- Four-tab agent builder header -->
    <div class="flex items-center justify-between px-3 py-2 bg-slate-900 border-b border-slate-800 shrink-0 z-10">
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

    <!-- Breadcrumb navigation for sub-graph drill-in -->
    <div
      v-if="breadcrumbs.length > 0"
      class="px-3 py-1.5 bg-slate-900 border-b border-slate-800 shrink-0 overflow-hidden z-10"
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

    <!-- Main canvas area -->
    <div
      ref="canvasContainer"
      class="flex-1 overflow-hidden relative flex"
      @dragover.prevent="onDragOver"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @drop="onDrop"
      @keydown="onKeyDown"
      tabindex="0"
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
        <!-- Drop zone overlay -->
        <div
          v-if="dragOver"
          class="absolute inset-0 bg-indigo-500/10 border-2 border-dashed border-indigo-500 flex items-center justify-center pointer-events-none z-10"
        >
          <span class="text-indigo-300 font-medium text-sm">Drop block here</span>
        </div>
      </VueFlow>

      <!-- Canvas controls (bottom right, Gumloop style) -->
      <div class="absolute bottom-4 right-4 flex flex-col gap-2 z-20">
        <div class="flex gap-1">
          <button
            @click="fitView"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Fit View (Ctrl/Cmd + 0)"
          >
            Fit View
          </button>
          <button
            @click="autoFormat"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Auto Format"
          >
            Auto Format
          </button>
          <button
            @click="expandAll"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Expand All Nodes"
          >
            Expand All
          </button>
          <button
            @click="collapseAll"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Collapse All Nodes"
          >
            Collapse All
          </button>
        </div>
        <div class="flex gap-1 mt-1">
          <button
            @click="toggleSnapToGrid"
            :class="[
              'px-2 py-1 text-xs rounded border text-slate-300 transition-colors',
              snapToGrid ? 'bg-indigo-600 border-indigo-500 text-white' : 'bg-slate-800 border-slate-700 hover:border-slate-500',
            ]"
            title="Snap to Grid"
          >
            Snap Grid
          </button>
          <button
            @click="zoomIn"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Zoom In"
          >
            +
          </button>
          <button
            @click="zoomOut"
            class="px-2 py-1 text-xs bg-slate-800 border border-slate-700 hover:border-slate-500 rounded text-slate-300 hover:text-white transition-colors"
            title="Zoom Out"
          >
            −
          </button>
        </div>
      </div>

      <!-- Run button (top right of canvas) -->
      <div class="absolute top-4 right-4 z-20">
        <RunControl :agent-id="selectedAgentId" />
      </div>
    </div>

      <!-- Right sidebar: Pin panel -->
      <div class="w-96 bg-slate-900 border-l border-slate-800 flex flex-col">
        <PinPanel
          :agent-id="selectedAgentId"
          :node-id="selectedNodeId"
          :port-id="selectedPortId"
          :value="selectedPortValue"
        />
        <SkillEditor :agent-id="selectedAgentId" />
        <KnowledgeSourceEditor :agent-id="selectedAgentId" />
        <TriggerEditor :agent-id="selectedAgentId" />
      </div>
    </div>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  VueFlow,
  useVueFlow,
  type NodeDragEvent,
  type NodeMouseEvent,
  type Connection,
} from '@vue-flow/core';
import type { BlockDefinition } from '@/types/agent';
import { useGraphStore } from '@/stores/graphStore';
import { CanvasAdapter } from '@/services/canvasAdapter';
import AgentNode from '@/components/canvas/AgentNode.vue';
import ConnectorNode from '@/components/canvas/ConnectorNode.vue';
import StepEdge from '@/components/canvas/StepEdge.vue';
import RunControl from '@/components/RunControl.vue';
import PinPanel from '@/components/PinPanel.vue';
import SkillEditor from '@/components/inspector/SkillEditor.vue';
import KnowledgeSourceEditor from '@/components/inspector/KnowledgeSourceEditor.vue';
import TriggerEditor from '@/components/inspector/TriggerEditor.vue';

const store = useGraphStore();

const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);
const breadcrumbs = computed(() => store.breadcrumbs);
const isInSubGraph = computed(() => store.isInSubGraph);
const errorCount = computed(() => store.errorCount);
const selectedAgentId = ref<string | undefined>(undefined);
const selectedNodeId = ref<string | undefined>(undefined);
const selectedPortId = ref<string | undefined>(undefined);
const selectedPortValue = ref<any>(undefined);

const flowNodes = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.nodes.map((node) => {
    const block = blocks.value.find((b) => b.id === node.blockId);
    const errors = store.getNodeErrors(node.id);
    const hasError = errors.some((e) => e.severity === 'error');
    const isConnector = block?.account_selection || block?.tool_management;

    return {
      id: node.id,
      type: isConnector ? 'connectorNode' : 'agentNode',
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
  }));
});

const canvasContainer = ref<HTMLDivElement | null>(null);
const dragOver = ref(false);
const snapToGrid = ref(false);

// Vue Flow instance methods
const { fitView: fitViewFn, zoomIn: zoomInFn, zoomOut: zoomOutFn } = useVueFlow();

onMounted(() => {
  canvasContainer.value?.focus();
  window.addEventListener('open-tool-manager', (e: Event) => {
    const customEvent = e as CustomEvent<{ nodeId: string; block: any }>;
    const { nodeId, block } = customEvent.detail;
    console.log('Open tool manager for node:', nodeId, block);
    // The tool manager panel will be shown by the parent app
  });
});

function onNodeDrag(event: NodeDragEvent) {
  const node = event.node;
  store.updateNodePosition(node.id, { x: node.position.x, y: node.position.y });
}

function onNodeClick(event: NodeMouseEvent) {
  store.selectNode(event.node.id);
  selectedNodeId.value = event.node.id;
  // Default to first output port if available
  const node = graph.value?.nodes.find((n) => n.id === event.node.id);
  const block = blocks.value.find((b) => b.id === node?.blockId);
  const firstOutput = block?.outputs?.[0];
  selectedPortId.value = firstOutput?.id ?? undefined;
  selectedPortValue.value = undefined;
}

function onBackgroundClick() {
  store.selectNode(null);
}

function onConnect(connection: Connection) {
  const sourceNode = graph.value?.nodes.find((n) => n.id === connection.source);
  const targetNode = graph.value?.nodes.find((n) => n.id === connection.target);

  if (!sourceNode || !targetNode) return;

  const sourceBlock = blocks.value.find((b) => b.id === sourceNode.blockId);
  const targetBlock = blocks.value.find((b) => b.id === targetNode.blockId);

  if (!sourceBlock || !targetBlock) return;

  const result = CanvasAdapter.canConnectNodes(
    { block: { outputs: sourceBlock.outputs } },
    { block: { inputs: targetBlock.inputs } }
  );
  if (!result.allowed) {
    console.warn(`Connection rejected: ${result.reason}`);
    return;
  }

  store.addEdge({
    source: connection.source,
    target: connection.target,
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

function onKeyDown(event: KeyboardEvent) {
  // Resume button: Cmd/Ctrl + Shift + Enter
  if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'Enter') {
    event.preventDefault();
    runGraph();
  }
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

const nodeTypes = { agentNode: AgentNode as any, connectorNode: ConnectorNode as any };
const edgeTypes = { step: StepEdge as any };

function autoFormat() {
  // Auto-arrange nodes in a clean layout
  const g = graph.value;
  if (!g) return;
  const nodes = g.nodes.slice().sort((a, b) => a.position.y - b.position.y || a.position.x - b.position.x);
  let x = 100;
  let y = 100;
  const rowHeight = 200;
  nodes.forEach((node, i) => {
    if (i > 0 && i % 3 === 0) {
      x = 100;
      y += rowHeight;
    }
    store.updateNodePosition(node.id, { x, y });
    x += 240;
  });
}

function expandAll() {
  const g = graph.value;
  if (!g) return;
  g.nodes.forEach((node) => {
    if (node.isCollapsed) {
      (store as any).expandNode(node.id);
    }
  });
}

function collapseAll() {
  const g = graph.value;
  if (!g) return;
  g.nodes.forEach((node) => {
    if (!node.isCollapsed) {
      (store as any).collapseNode(node.id);
    }
  });
}

function toggleSnapToGrid() {
  snapToGrid.value = !snapToGrid.value;
}

function zoomIn() {
  zoomInFn({ duration: 200 });
}

function zoomOut() {
  zoomOutFn({ duration: 200 });
}

function runGraph() {
  store.runGraph?.();
}

function fitView() {
  fitViewFn({ duration: 300 });
}

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
