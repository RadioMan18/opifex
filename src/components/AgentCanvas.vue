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
        id="opifex-canvas"
        :nodes="flowNodes"
        :edges="flowEdges"
        pan-on-scroll
        pan-on-drag
        zoom-on-pan
        @connect="onConnect"
        @node-drag="onNodeDrag"
        @node-drag-start="onNodeDragStart"
        @node-drag-stop="onNodeDragStop"
        @node-click="onNodeClick"
        @edge-click="onEdgeClick"
        @background-click="onBackgroundClick"
        :node-types="nodeTypes"
        :edge-types="edgeTypes"
        :is-valid-connection="isValidConnection"
      >
        <!-- Drop zone overlay -->
        <div
          v-if="dragOver"
          class="drop-overlay"
        >
          <span class="drop-overlay-text">Drop block here</span>
        </div>
      </VueFlow>

      <div v-if="showSearch" class="canvas-search">
        <input
          ref="searchInput"
          v-model="searchTerm"
          class="canvas-search-input"
          type="search"
          placeholder="Find a block…"
          aria-label="Find a block on the canvas"
          @input="matchIndex = 0"
          @keydown.enter.prevent="nextMatch(1)"
          @keydown.esc.prevent="closeSearch"
        />
        <span class="canvas-search-count">{{ matchLabel }}</span>
        <button class="canvas-search-close" type="button" aria-label="Close search" @click="closeSearch">
          ✕
        </button>
      </div>

      <!-- A marker cannot take the colour of the path that references it, so
           every edge state has its own arrow def and paths point at one by id. -->
      <svg class="edge-marker-defs" aria-hidden="true" focusable="false">
        <defs>
          <marker
            id="opifex-arrow"
            viewBox="0 0 8 8"
            refX="7"
            refY="4"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 8 4 L 0 8 z" fill="var(--edge-stroke, #94a3b8)" />
          </marker>
          <marker
            id="opifex-arrow-accent"
            viewBox="0 0 8 8"
            refX="7"
            refY="4"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 8 4 L 0 8 z" fill="var(--primary, #3b82f6)" />
          </marker>
          <marker
            id="opifex-arrow-success"
            viewBox="0 0 8 8"
            refX="7"
            refY="4"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 8 4 L 0 8 z" fill="#10b981" />
          </marker>
          <marker
            id="opifex-arrow-failed"
            viewBox="0 0 8 8"
            refX="7"
            refY="4"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 8 4 L 0 8 z" fill="#ef4444" />
          </marker>
        </defs>
      </svg>

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

        <div class="canvas-control-group" role="group" aria-label="Edit history">
          <button
            class="canvas-control-btn"
            title="Undo (Ctrl/Cmd + Z)"
            aria-label="Undo"
            :disabled="!canUndo"
            @click="undo"
          >
            ↶
          </button>
          <button
            class="canvas-control-btn"
            title="Redo (Ctrl/Cmd + Shift + Z)"
            aria-label="Redo"
            :disabled="!canRedo"
            @click="redo"
          >
            ↷
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
          <button
            class="canvas-control-btn canvas-control-btn--wide"
            title="Find a block (Ctrl/Cmd + F)"
            aria-label="Find a block"
            @click="openSearch"
          >
            Find
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
            <button class="canvas-menu-item" role="menuitem" @click="runAndClose(autoLayout)">
              Auto layout
            </button>
            <button
              v-if="selectedNodeId"
              class="canvas-menu-item"
              role="menuitem"
              @click="runAndClose(toggleSelectedDisabled)"
            >
              {{ selectedNodeDisabled ? 'Enable selected block' : 'Disable selected block' }}
            </button>
            <div class="canvas-menu-divider"></div>
            <button
              class="canvas-menu-item"
              role="menuitem"
              :disabled="multiSelection.length < 2"
              @click="runAndClose(groupSelection)"
            >
              Group selection (Shift+click, then G)
            </button>
            <button
              class="canvas-menu-item"
              role="menuitem"
              :disabled="!selectedGroupId"
              @click="runAndClose(ungroupSelection)"
            >
              Ungroup
            </button>
            <div class="canvas-menu-divider"></div>
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

.canvas-control-btn:disabled {
  color: var(--text-muted);
  opacity: 0.45;
  cursor: default;
  background: transparent;
}

/* Marker defs must be in the document for url(#id) to resolve, but take no
   space in the canvas layout. */
.edge-marker-defs {
  position: absolute;
  width: 0;
  height: 0;
  overflow: hidden;
  pointer-events: none;
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

.canvas-menu-item:disabled {
  opacity: 0.45;
  cursor: default;
  background: transparent;
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

.canvas-search {
  position: absolute;
  top: var(--space-3, 10px);
  right: var(--space-4, 12px);
  z-index: 21;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md, 8px);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08), 0 8px 20px rgba(15, 23, 42, 0.06);
}

.canvas-search-input {
  width: 168px;
  padding: 3px 6px;
  font-family: inherit;
  font-size: var(--type-xs);
  color: var(--text-primary);
  background: transparent;
  border: none;
  outline: none;
}

.canvas-search-count {
  font-size: 10px;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.canvas-search-close {
  min-width: 20px;
  height: 20px;
  padding: 0;
  font-size: 10px;
  color: var(--text-muted);
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.canvas-search-close:hover {
  color: var(--text-primary);
  background: var(--surface-hover);
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
import { ref, computed, onMounted, onBeforeUnmount, markRaw, nextTick, watch } from 'vue';
import {
  VueFlow,
  useVueFlow,
  type NodeDragEvent,
  type NodeMouseEvent,
  type Connection,
  type EdgeMouseEvent,
} from '@vue-flow/core';
import type { BlockDefinition } from '@/types/agent';
import { useGraphStore } from '@/stores/graphStore';
import { CanvasAdapter } from '@/services/canvasAdapter';
import AgentNode from '@/components/canvas/AgentNode.vue';
import ConnectorNode from '@/components/canvas/ConnectorNode.vue';
import StepEdge from '@/components/canvas/StepEdge.vue';
import GroupNode from '@/components/canvas/GroupNode.vue';
import { layeredLayout } from '@/services/canvasLayout';
import type { GraphGroup } from '@/stores/graphStore';

const store = useGraphStore();

// Type assertions and type safety improvements
const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);
const breadcrumbs = computed(() => store.breadcrumbs);

// Card sizes are measured from the DOM: Vue Flow keeps them in its own store and
// group boxes need them to hug their contents.
const nodeSizes = ref<Record<string, { width: number; height: number }>>({});

function groupBounds(group: GraphGroup) {
  const members = (graph.value?.nodes ?? []).filter((node) => group.nodeIds.includes(node.id));
  if (!members.length) return null;

  const PAD = 26;
  const HEADER = 24;
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const member of members) {
    const size = nodeSizes.value[member.id] ?? { width: 250, height: 150 };
    minX = Math.min(minX, member.position.x);
    minY = Math.min(minY, member.position.y);
    maxX = Math.max(maxX, member.position.x + size.width);
    maxY = Math.max(maxY, member.position.y + size.height);
  }

  return {
    x: minX - PAD,
    y: minY - PAD - HEADER,
    width: maxX - minX + PAD * 2,
    height: maxY - minY + PAD * 2 + HEADER,
  };
}

const flowNodes = computed(() => {
  const g = graph.value;
  if (!g) return [];

  const nodes: any[] = g.nodes.map((node) => {
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
        runState: store.runNodeStates[node.id],
        isDisabled: store.isNodeDisabled(node.id),
        isMatch: node.id === currentMatchId.value,
      },
    };
  });

  // Groups render as nodes so Vue Flow's own transform keeps the boxes lined up
  // through pan and zoom; zIndex -1 puts them behind the cards they contain.
  for (const group of store.groups) {
    const box = groupBounds(group);
    if (!box) continue;
    nodes.push({
      id: `group:${group.id}`,
      type: 'groupNode',
      position: { x: box.x, y: box.y },
      draggable: false,
      selectable: false,
      connectable: false,
      zIndex: -1,
      style: { width: `${box.width}px`, height: `${box.height}px` },
      data: { label: group.name, memberCount: group.nodeIds.length },
    });
  }

  return nodes;
});

const selectedEdgeId = ref<string | null>(null);

const multiSelection = ref<string[]>([]);

const selectedNodeId = computed(() => store.selectedNodeId);
const selectedNodeDisabled = computed(() =>
  selectedNodeId.value ? store.isNodeDisabled(selectedNodeId.value) : false
);
const selectedGroupId = computed(() =>
  selectedNodeId.value ? (store.groupForNode(selectedNodeId.value)?.id ?? null) : null
);

// Find-in-graph state. Matches are highlighted on the cards, and the canvas
// does not pan: selecting the match is enough to reach it from the inspector.
const showSearch = ref(false);
const searchTerm = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const matchIndex = ref(0);

function nodeLabel(nodeId: string) {
  const node = graph.value?.nodes.find((n) => n.id === nodeId);
  return blocks.value.find((b) => b.id === node?.blockId)?.name ?? node?.blockId ?? nodeId;
}

const matchIds = computed(() => {
  const term = searchTerm.value.trim().toLowerCase();
  if (!term) return [] as string[];
  return (graph.value?.nodes ?? [])
    .filter((node) => nodeLabel(node.id).toLowerCase().includes(term))
    .map((node) => node.id);
});

const currentMatchId = computed(() => matchIds.value[matchIndex.value] ?? null);
const matchLabel = computed(() =>
  matchIds.value.length ? `${matchIndex.value + 1}/${matchIds.value.length}` : '0/0'
);

const flowEdges = computed(() => {
  const g = graph.value;
  if (!g) return [];
  return g.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    type: 'stepEdge',
    data: {
      selected: edge.id === selectedEdgeId.value,
      runState: store.runEdgeStates[edge.id],
    },
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
} = useVueFlow('opifex-canvas');

const canUndo = computed(() => store.canUndo);
const canRedo = computed(() => store.canRedo);

const showCanvasMenu = ref(false);
const zoomPercent = computed(() => Math.round((viewport.value?.zoom ?? 1) * 100));

onMounted(() => {
  canvasContainer.value?.focus();
  window.addEventListener('keydown', onWindowKeyDown);
  // Capture phase: Vue Flow clears its connection state on a bubbling
  // document mouseup, which would run after this.
  document.addEventListener('mousedown', onDocumentMouseDown, true);
  document.addEventListener('mousemove', onDocumentMouseMove, true);
  document.addEventListener('mouseup', onDocumentMouseUp, true);
  nextTick(measureNodes);
  watch(
    () => [graph.value?.nodes.length, store.groups.length, viewport.value?.zoom],
    () => nextTick(measureNodes)
  );
  window.addEventListener('open-tool-manager', (e: Event) => {
    const customEvent = e as CustomEvent<{ nodeId: string; block: any }>;
    const { nodeId, block } = customEvent.detail;
    console.log('Open tool manager for node:', nodeId, block);
    // The tool manager panel will be shown by the parent app
  });
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown, true);
  document.removeEventListener('mousemove', onDocumentMouseMove, true);
  document.removeEventListener('mouseup', onDocumentMouseUp, true);
  window.removeEventListener('keydown', onWindowKeyDown);
});

// --- Functions (Logic Review: Minimal changes for type safety/modernity) ---

function onNodeDrag(event: NodeDragEvent) {
  const node = event.node;
  store.updateNodePosition(node.id, { x: node.position.x, y: node.position.y });
}

function onNodeClick(event: NodeMouseEvent) {
  selectedEdgeId.value = null;
  const nodeId = event.node.id;

  // Shift extends the selection, which is what grouping operates on.
  if (event.event?.shiftKey) {
    multiSelection.value = multiSelection.value.includes(nodeId)
      ? multiSelection.value.filter((id) => id !== nodeId)
      : [...multiSelection.value, nodeId];
  } else {
    multiSelection.value = [nodeId];
  }

  store.selectNode(nodeId);
}

function onBackgroundClick() {
  selectedEdgeId.value = null;
  multiSelection.value = [];
  store.selectNode(null);
}

function onEdgeClick(event: EdgeMouseEvent) {
  selectedEdgeId.value = event.edge.id;
  store.selectNode(null);
}

function removeSelectedEdge() {
  if (!selectedEdgeId.value) return;
  store.removeEdge(selectedEdgeId.value);
  selectedEdgeId.value = null;
}

// A drag moves the node on every frame but is one edit, so history opens at the
// start and closes at the end instead of recording every frame.
function onNodeDragStart() {
  store.beginPositionChange();
}

function onNodeDragStop() {
  store.endPositionChange();
  nextTick(measureNodes);
}

function undo() {
  store.undo();
  selectedEdgeId.value = null;
}

function redo() {
  store.redo();
  selectedEdgeId.value = null;
}

// Undo is useless if it only fires while the canvas holds focus, so the
// shortcut lives on the window and stays out of the way of text fields.
function onWindowKeyDown(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null;
  const tag = target?.tagName ?? '';
  if (target?.isContentEditable || tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
    return;
  }
  const key = event.key.toLowerCase();

  if (!(event.metaKey || event.ctrlKey)) {
    // Single-key actions: B switches a block out of the run without deleting the
    // work on it, G groups the shift-clicked selection. Ctrl/Cmd+G is the
    // browser's find-next, so grouping deliberately avoids a modifier.
    if (key === 'b' && selectedNodeId.value) {
      event.preventDefault();
      toggleSelectedDisabled();
    } else if (key === 'g') {
      event.preventDefault();
      if (event.shiftKey) ungroupSelection();
      else groupSelection();
    }
    return;
  }

  if (key === 'f') {
    event.preventDefault();
    openSearch();
    return;
  }
  if (key === 'z') {
    event.preventDefault();
    if (event.shiftKey) redo();
    else undo();
  } else if (key === 'y') {
    event.preventDefault();
    redo();
  }
}

// Vue Flow's `connect` event never reaches this component (the instance emits
// are dropped in this setup), so a connection is completed from the two handles
// the pointer visits: the one under mousedown and the one under mouseup. A
// plain click never draws an edge, and `onConnect` still runs if the library
// does emit — the store ignores duplicate edges.
type HandleRef = { nodeId: string; type: 'source' | 'target' };

let dragHandle: HandleRef | null = null;
let dragMoved = false;

function handleUnderPointer(event: MouseEvent): HTMLElement | null {
  const under = document.elementFromPoint(event.clientX, event.clientY) as HTMLElement | null;
  return (under?.closest('.vue-flow__handle') as HTMLElement | null) ?? null;
}

function handleRef(handle: HTMLElement): HandleRef | null {
  const nodeId = handle.getAttribute('data-nodeid');
  if (!nodeId) return null;
  return { nodeId, type: handle.classList.contains('source') ? 'source' : 'target' };
}

function onDocumentMouseDown(event: MouseEvent) {
  dragMoved = false;
  const handle = handleUnderPointer(event);
  dragHandle = handle ? handleRef(handle) : null;
}

function onDocumentMouseMove() {
  if (dragHandle) dragMoved = true;
}

function onDocumentMouseUp(event: MouseEvent) {
  const start = dragHandle;
  dragHandle = null;
  if (!start || !dragMoved) return;

  const handle = handleUnderPointer(event);
  if (!handle) return;

  const end = handleRef(handle);
  if (!end || end.type === start.type || end.nodeId === start.nodeId) return;

  onConnect(
    start.type === 'source'
      ? { source: start.nodeId, target: end.nodeId, sourceHandle: null, targetHandle: null }
      : { source: end.nodeId, target: start.nodeId, sourceHandle: null, targetHandle: null }
  );
}

function onConnect(connection: Connection) {
  const sourceNode = graph.value?.nodes.find((n) => n.id === connection.source);
  const targetNode = graph.value?.nodes.find((n) => n.id === connection.target);

  if (!sourceNode || !targetNode || sourceNode.id === targetNode.id) return;

  const sourceBlock = blocks.value.find((b) => b.id === sourceNode.blockId);
  const targetBlock = blocks.value.find((b) => b.id === targetNode.blockId);

  if (!sourceBlock || !targetBlock) return;

  if (!isValidConnection(connection)) {
    const result = CanvasAdapter.canConnectNodes({ block: sourceBlock }, { block: targetBlock });
    console.warn(`[canvas] connection rejected: ${result.reason}`);
    return;
  }

  store.addEdge({
    source: connection.source,
    target: connection.target,
  });
}

// Run while a connection is dragged, so an invalid target is refused at the
// pointer instead of silently creating an edge the graph cannot execute.
function isValidConnection(connection: Connection): boolean {
  const g = graph.value;
  if (!g || !connection.source || !connection.target) return false;
  if (connection.source === connection.target) return false;
  // Vue Flow re-validates every stored edge through this same prop, so an edge
  // that is already in the store must not be rejected as its own duplicate.
  // Only a freshly dragged connection (which carries no id) is deduped here.
  const storedEdgeId = (connection as Connection & { id?: string }).id;
  if (
    !storedEdgeId &&
    g.edges.some((e) => e.source === connection.source && e.target === connection.target)
  ) {
    return false;
  }

  const sourceNode = g.nodes.find((n) => n.id === connection.source);
  const targetNode = g.nodes.find((n) => n.id === connection.target);
  if (!sourceNode || !targetNode) return false;

  const sourceBlock = blocks.value.find((b) => b.id === sourceNode.blockId);
  const targetBlock = blocks.value.find((b) => b.id === targetNode.blockId);
  if (!sourceBlock || !targetBlock) return false;

  return CanvasAdapter.canConnectNodes({ block: sourceBlock }, { block: targetBlock }).allowed;
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
  nextTick(measureNodes);

  console.debug('[canvas] dropped', block.id, position, 'nodes:', graph.value?.nodes.length);
}

function onKeyDown(event: KeyboardEvent) {
  // Remove the selected edge: an edge you cannot undo is a trap once edges are
  // drawable.
  if ((event.key === 'Delete' || event.key === 'Backspace') && selectedEdgeId.value) {
    event.preventDefault();
    removeSelectedEdge();
    return;
  }

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
const nodeTypes: any = markRaw({
  agentNode: AgentNode,
  connectorNode: ConnectorNode,
  groupNode: GroupNode,
});
const edgeTypes: any = markRaw({ stepEdge: StepEdge });


// Zoom is divided out because this reads screen pixels, not flow units.
function measureNodes() {
  const root = canvasContainer.value;
  if (!root) return;

  const zoom = viewport.value?.zoom ?? 1;
  const measured: Record<string, { width: number; height: number }> = {};

  root.querySelectorAll<HTMLElement>('.vue-flow__node').forEach((el) => {
    const id = el.getAttribute('data-id');
    if (!id || id.startsWith('group:')) return;
    const rect = el.getBoundingClientRect();
    measured[id] = { width: rect.width / zoom, height: rect.height / zoom };
  });

  const changed =
    Object.keys(measured).length !== Object.keys(nodeSizes.value).length ||
    Object.entries(measured).some(([id, size]) => {
      const previous = nodeSizes.value[id];
      return (
        !previous ||
        Math.abs(previous.width - size.width) > 1 ||
        Math.abs(previous.height - size.height) > 1
      );
    });

  if (changed) nodeSizes.value = measured;
}

function autoLayout() {
  const g = graph.value;
  if (!g || !g.nodes.length) return;

  const positions = layeredLayout(g);
  store.beginPositionChange();
  for (const [nodeId, position] of Object.entries(positions)) {
    store.updateNodePosition(nodeId, position);
  }
  store.endPositionChange();

  nextTick(() => {
    measureNodes();
    fitView();
  });
}

function openSearch() {
  showSearch.value = true;
  nextTick(() => searchInput.value?.focus());
}

function closeSearch() {
  showSearch.value = false;
  searchTerm.value = '';
  matchIndex.value = 0;
}

function nextMatch(step = 1) {
  const total = matchIds.value.length;
  if (!total) return;
  matchIndex.value = (matchIndex.value + step + total) % total;
  const target = matchIds.value[matchIndex.value];
  selectedEdgeId.value = null;
  store.selectNode(target);
}

function toggleSelectedDisabled() {
  if (selectedNodeId.value) store.toggleNodeDisabled(selectedNodeId.value);
}

function groupSelection() {
  const members = multiSelection.value.length
    ? multiSelection.value
    : selectedNodeId.value
      ? [selectedNodeId.value]
      : [];
  if (!store.createGroup(members)) return;
  multiSelection.value = [];
  nextTick(measureNodes);
}

function ungroupSelection() {
  if (!selectedGroupId.value) return;
  store.removeGroup(selectedGroupId.value);
  nextTick(measureNodes);
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