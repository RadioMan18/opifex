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

      <!-- Canvas controls: one cluster, bottom-left, clear of the nodes -->
      <div class="canvas-controls" @keydown.esc="showCanvasMenu = false">
        <div class="canvas-control-group" role="group" aria-label="Canvas zoom">
          <button
            class="canvas-control-btn"
            title="Zoom out"
            aria-label="Zoom out"
            @click="zoomOut"
          >
            −
          </button>
          <button
            class="canvas-control-btn canvas-zoom-value"
            title="Reset zoom to 100%"
            aria-label="Reset zoom to 100%"
            @click="resetZoom"
          >
            {{ zoomPercent }}%
          </button>
          <button
            class="canvas-control-btn"
            title="Zoom in"
            aria-label="Zoom in"
            @click="zoomIn"
          >
            +
          </button>
        </div>

        <div class="canvas-control-group" role="group" aria-label="Canvas view">
          <button
            class="canvas-control-btn canvas-control-btn--wide"
            title="Fit view (Ctrl/Cmd + 0)"
            aria-label="Fit view"
            @click="fitView"
          >
            Fit
          </button>
        </div>

        <div class="canvas-control-group canvas-control-group--menu">
          <button
            class="canvas-control-btn canvas-control-btn--wide"
            :aria-expanded="showCanvasMenu"
            aria-haspopup="menu"
            title="Canvas tools"
            aria-label="Canvas tools"
            @click="showCanvasMenu = !showCanvasMenu"
          >
            ⋯
          </button>

          <div v-if="showCanvasMenu" class="canvas-menu" role="menu">
            <button class="canvas-menu-item" role="menuitem" @click="runAndClose(autoFormat)">
              Auto format
            </button>
            <button class="canvas-menu-item" role="menuitem" @click="runAndClose(expandAll)">
              Expand all
            </button>
            <button class="canvas-menu-item" role="menuitem" @click="runAndClose(collapseAll)">
              Collapse all
            </button>
            <div class="canvas-menu-divider"></div>
            <button
              class="canvas-menu-item"
              role="menuitemcheckbox"
              :aria-checked="snapToGrid"
              @click="toggleSnapToGrid"
            >
              <span class="canvas-menu-check" aria-hidden="true">{{ snapToGrid ? '✓' : '' }}</span>
              Snap to grid
            </button>
          </div>
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

.canvas-controls {
  position: absolute;
  left: var(--space-4, 12px);
  bottom: var(--space-4, 12px);
  z-index: 20;
  display: flex;
  align-items: flex-end;
  gap: 6px;
}

.canvas-control-group {
  display: flex;
  align-items: center;
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md, 8px);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08), 0 8px 20px rgba(15, 23, 42, 0.06);
}

.canvas-control-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 30px;
  height: 30px;
  padding: 0 8px;
  font-family: inherit;
  font-size: var(--type-xs);
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-left: 1px solid var(--border);
  cursor: pointer;
  transition: background 0.14s ease, color 0.14s ease;
}

.canvas-control-btn:first-child {
  border-left: none;
}

.canvas-control-btn:hover {
  color: var(--text-primary);
  background: var(--surface-hover);
}

.canvas-control-btn:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: -2px;
}

.canvas-control-btn--wide {
  min-width: 44px;
}

.canvas-zoom-value {
  min-width: 52px;
  font-variant-numeric: tabular-nums;
}

.canvas-control-group--menu {
  position: relative;
}

.canvas-menu {
  position: absolute;
  left: 0;
  bottom: calc(100% + 6px);
  min-width: 168px;
  display: flex;
  flex-direction: column;
  padding: 4px;
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md, 8px);
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.16);
}

.canvas-menu-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-family: inherit;
  font-size: var(--type-xs);
  color: var(--text-secondary);
  text-align: left;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
}

.canvas-menu-item:hover {
  color: var(--text-primary);
  background: var(--surface-hover);
}

.canvas-menu-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--border);
}

.canvas-menu-check {
  display: inline-block;
  width: 10px;
  font-size: 10px;
  color: var(--primary);
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
import { ref, computed, onMounted, markRaw } from 'vue';
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
  zoomTo: zoomToFn,
  viewport,
  screenToFlowCoordinate,
} = useVueFlow();

const showCanvasMenu = ref(false);
const zoomPercent = computed(() => Math.round((viewport.value?.zoom ?? 1) * 100));

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

  // Fit view: Cmd/Ctrl + 0
  if ((event.metaKey || event.ctrlKey) && !event.shiftKey && event.key === '0') {
    event.preventDefault();
    fitView();
  }
}

// --- Controls & Utilities ---

// Vue Flow keeps these maps in reactive state, so a plain object gets proxied
// and Vue then warns about reactive components on every node render. Excluding
// the maps from reactivity keeps the components raw and drops a proxy hop from
// every prop read.
const nodeTypes: any = markRaw({ agentNode: AgentNode, connectorNode: ConnectorNode });
const edgeTypes: any = markRaw({ stepEdge: StepEdge });


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

function resetZoom() {
  zoomToFn(1, { duration: 200 });
}

function runAndClose(action: () => void) {
  action();
  showCanvasMenu.value = false;
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