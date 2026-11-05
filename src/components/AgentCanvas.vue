<template>
  <div class="agent-canvas">
    <div
      v-if="breadcrumbs.length > 0"
      class="canvas-breadcrumbs"
    >
      <div class="breadcrumbs-list">
        <span class="breadcrumb-item" @click="clearBreadcrumbs">Root</span>
        <span
          v-for="(crumb, idx) in breadcrumbs"
          :key="crumb.nodeId"
          class="breadcrumb-separator"
        >
          <span class="breadcrumb-slash">/</span>
          <span
            v-if="idx < breadcrumbs.length - 1"
            class="breadcrumb-item"
            @click="drillToLevel(idx)"
          >
            {{ crumb.name }}
          </span>
          <span v-else class="breadcrumb-current">{{ crumb.name }}</span>
        </span>
      </div>
    </div>

    <!-- Main canvas area -->
    <div
      ref="canvasContainer"
      class="canvas-main-area"
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
          class="drop-overlay"
        >
          <span class="drop-overlay-text">Drop block here</span>
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
    </div>

  </div>
</template>

<style scoped>
.agent-canvas {
  display: flex;
  flex-direction: column;
  height: 100%;
  flex: 1;
  min-height: 0;
  background: var(--surface-base);
  position: relative;
}

.canvas-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-4);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  z-index: 10;
}

.canvas-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.canvas-header-title {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.subgraph-badge {
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  background: rgba(59, 130, 246, 0.15);
  color: var(--primary);
  font-size: 10px;
  border: 1px solid rgba(59, 130, 246, 0.4);
}

.canvas-breadcrumbs {
  padding: var(--space-1) var(--space-4);
  background: var(--surface-card);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  overflow: hidden;
  z-index: 10;
}

.breadcrumbs-list {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  overflow-x: auto;
  scrollbar-width: none;
}

.breadcrumbs-list::-webkit-scrollbar {
  display: none;
}

.breadcrumb-item {
  font-size: 10px;
  color: var(--text-muted);
  cursor: pointer;
}

.breadcrumb-item:hover {
  color: var(--text-primary);
}

.breadcrumb-separator {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 10px;
}

.breadcrumb-slash {
  color: var(--border);
}

.breadcrumb-current {
  font-size: 10px;
  color: var(--text-primary);
  font-weight: 500;
}

.canvas-main-area {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  position: relative;
  display: flex;
}

.drop-overlay {
  position: absolute;
  inset: 0;
  background: rgba(59, 130, 246, 0.1);
  border: 2px dashed var(--primary);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 10;
}

.drop-overlay-text {
  color: var(--primary);
  font-weight: 500;
  font-size: var(--type-sm);
}
</style>

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

const store = useGraphStore();

// Type assertions and type safety improvements
const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);
const breadcrumbs = computed(() => store.breadcrumbs);

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

// Vue Flow instance methods (No change needed here)
const {
  fitView: fitViewFn,
  zoomIn: zoomInFn,
  zoomOut: zoomOutFn,
  screenToFlowCoordinate,
} = useVueFlow();

onMounted(() => {
  canvasContainer.value?.focus();
  window.addEventListener('open-tool-manager', (e: Event) => {
    const customEvent = e as CustomEvent<{ nodeId: string; block: any }>;
    const { nodeId, block } = customEvent.detail;
    console.log('Open tool manager for node:', nodeId, block);
    // The tool manager panel will be shown by the parent app
  });
});

// --- Functions (Logic Review: Minimal changes for type safety/modernity) ---

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

function resolveDraggedBlock(event: DragEvent): BlockDefinition | null {
  const raw = event.dataTransfer?.getData('application/opifex-block') ?? '';
  if (raw) {
    try {
      return JSON.parse(raw) as BlockDefinition;
    } catch {
      // fall through to the lookups below
    }
  }

  // WebView2 does not reliably return a custom drag format on drop, so fall
  // back to the plain-text id and finally to what the palette reported.
  const id = event.dataTransfer?.getData('text/plain') ?? '';
  return blocks.value.find((b) => b.id === id) ?? store.draggingBlock;
}

async function onDrop(event: DragEvent) {
  event.preventDefault();
  dragOver.value = false;

  const block = resolveDraggedBlock(event);
  if (!block || !canvasContainer.value) {
    console.warn('[canvas] drop ignored: no block payload', {
      types: event.dataTransfer ? Array.from(event.dataTransfer.types) : [],
    });
    return;
  }

  // Drop coordinates are screen-space; Vue Flow positions nodes in flow-space,
  // so the current pan/zoom must be applied or the node lands off-screen.
  const position = screenToFlowCoordinate({ x: event.clientX, y: event.clientY });

  store.addNode({
    blockId: block.id,
    blockVersion: block.version,
    position,
    parameters: {},
    subGraph: undefined,
    isCollapsed: false,
  });
  store.setDraggingBlock(null);

  console.debug('[canvas] dropped', block.id, position, 'nodes:', graph.value?.nodes.length);
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

// --- Controls & Utilities ---

const nodeTypes: any = { agentNode: AgentNode, connectorNode: ConnectorNode };
const edgeTypes: any = { stepEdge: StepEdge };


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