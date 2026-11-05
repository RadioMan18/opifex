import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  BlockDefinition,
  NodeInstance,
  EdgeInstance,
  AgentGraph,
  ValidationResult,
  ValidationError,
} from '@/types/agent';

export type RunNodeState = 'running' | 'succeeded' | 'failed' | 'skipped';

/** A named set of canvas nodes shown as one box. */
export interface GraphGroup {
  id: string;
  name: string;
  nodeIds: string[];
}

export const useGraphStore = defineStore('graph', () => {
  // Current graph being edited
  const graph = ref<AgentGraph | null>(null);

  // Available block definitions (from backend/library)
  const blocks = ref<BlockDefinition[]>([]);

  // Block currently being dragged out of the palette. WebView2 does not always
  // hand the drop event its custom drag payload, so the canvas keeps this as a
  // fallback source of the block being dragged.
  const draggingBlock = ref<BlockDefinition | null>(null);

  // Currently selected node on canvas
  const selectedNodeId = ref<string | null>(null);

  // Validation result for current graph
  const validationResult = ref<ValidationResult | null>(null);

  // Breadcrumbs for sub-graph drill-in navigation
  const breadcrumbs = ref<Array<{ nodeId: string; name: string }>>([]);

  // Edit history. Snapshots are serialised graphs rather than live objects: the
  // canvas renders the very node objects the store holds, so an undo that
  // shared them would mutate the view it is meant to roll back.
  const history = ref<string[]>([]);
  const future = ref<string[]>([]);
  const MAX_HISTORY = 100;

  // Run visualisation. Painted onto the canvas after a run and cleared whenever
  // the graph changes, so a stale trace never reads as the current one.
  const runNodeStates = ref<Record<string, RunNodeState>>({});
  const runEdgeStates = ref<Record<string, RunNodeState>>({});
  const runSummary = ref<string | null>(null);

  // Canvas-only state. Disabled blocks and groups are not part of the graph
  // document: the Rust NodeInstance has no such fields, so a graph that carried
  // them would disagree with what a run actually executes.
  const disabledNodeIds = ref<string[]>([]);
  const groups = ref<GraphGroup[]>([]);

  // Set graph (load from backend or create new)
  function setGraph(newGraph: AgentGraph) {
    graph.value = newGraph;
    selectedNodeId.value = null;
    breadcrumbs.value = [];
    clearHistory();
    clearRunVisual();
  }

  // Create a new empty graph
  function createGraph(name: string) {
    const now = new Date().toISOString();
    graph.value = {
      id: crypto.randomUUID(),
      name,
      description: undefined,
      schemaVersion: 1,
      nodes: [],
      edges: [],
      createdAt: now,
      updatedAt: now,
    };
    selectedNodeId.value = null;
    breadcrumbs.value = [];
    clearHistory();
    clearRunVisual();
  }

  // Track the block being dragged from the palette
  function setDraggingBlock(block: BlockDefinition | null) {
    draggingBlock.value = block;
  }

  // Add a node to the graph
  function addNode(node: Omit<NodeInstance, 'id'> & { position: { x: number; y: number } }) {
    if (!graph.value) return;
    pushHistory();
    clearRunVisual();
    const newNode: NodeInstance = {
      id: crypto.randomUUID(),
      ...node,
    };
    graph.value.nodes.push(newNode);
    graph.value.updatedAt = new Date().toISOString();
    return newNode.id;
  }

  // Remove a node and all connected edges
  function removeNode(nodeId: string) {
    if (!graph.value) return;
    pushHistory();
    clearRunVisual();
    graph.value.nodes = graph.value.nodes.filter((n) => n.id !== nodeId);
    graph.value.edges = graph.value.edges.filter(
      (e) => e.source !== nodeId && e.target !== nodeId
    );
    if (selectedNodeId.value === nodeId) {
      selectedNodeId.value = null;
    }
    disabledNodeIds.value = disabledNodeIds.value.filter((id) => id !== nodeId);
    groups.value = groups.value
      .map((group) => ({ ...group, nodeIds: group.nodeIds.filter((id) => id !== nodeId) }))
      .filter((group) => group.nodeIds.length > 1);
    graph.value.updatedAt = new Date().toISOString();
  }

  // Update node position
  function updateNodePosition(nodeId: string, position: { x: number; y: number }) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node) {
      node.position = position;
      graph.value!.updatedAt = new Date().toISOString();
    }
  }

  // Update node parameters
  function updateNodeParameters(nodeId: string, parameters: Record<string, unknown>) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node) {
      pushHistory();
      node.parameters = { ...node.parameters, ...parameters };
      graph.value!.updatedAt = new Date().toISOString();
    }
  }

  // Add an edge (node-to-node, no port handles)
  function addEdge(edge: Omit<EdgeInstance, 'id'>) {
    if (!graph.value) return;
    // Prevent duplicate edges
    const exists = graph.value.edges.some(
      (e) => e.source === edge.source && e.target === edge.target
    );
    if (exists) return;
    pushHistory();
    clearRunVisual();

    const newEdge: EdgeInstance = {
      id: crypto.randomUUID(),
      ...edge,
    };
    graph.value.edges.push(newEdge);
    graph.value.updatedAt = new Date().toISOString();
    return newEdge.id;
  }

  // Remove an edge
  function removeEdge(edgeId: string) {
    if (!graph.value) return;
    pushHistory();
    clearRunVisual();
    graph.value.edges = graph.value.edges.filter((e) => e.id !== edgeId);
    graph.value.updatedAt = new Date().toISOString();
  }

  // Select a node
  function selectNode(nodeId: string | null) {
    selectedNodeId.value = nodeId;
  }

  function serializeGraph(value: AgentGraph | null): string | null {
    if (!value) return null;
    // Drilling into a sub-graph links the parent onto the child, and those
    // references are circular: navigation state, not document state.
    return JSON.stringify(value, (key, nested) => (key.startsWith('_') ? undefined : nested));
  }

  function pushHistory() {
    const snapshot = serializeGraph(graph.value);
    if (!snapshot) return;
    history.value.push(snapshot);
    if (history.value.length > MAX_HISTORY) history.value.shift();
    future.value = [];
  }

  function clearHistory() {
    history.value = [];
    future.value = [];
  }

  function restoreSnapshot(snapshot: string) {
    const restored = JSON.parse(snapshot) as AgentGraph;
    const current = graph.value as
      | (AgentGraph & { _parentGraph?: AgentGraph; _parentNodeId?: string })
      | null;
    if (current?._parentGraph) {
      (restored as any)._parentGraph = current._parentGraph;
      (restored as any)._parentNodeId = current._parentNodeId;
    }
    graph.value = restored;
    if (selectedNodeId.value && !restored.nodes.some((node) => node.id === selectedNodeId.value)) {
      selectedNodeId.value = null;
    }
  }

  function undo() {
    if (!history.value.length) return;
    const current = serializeGraph(graph.value);
    if (current) future.value.push(current);
    restoreSnapshot(history.value.pop()!);
  }

  function redo() {
    if (!future.value.length) return;
    const current = serializeGraph(graph.value);
    if (current) history.value.push(current);
    restoreSnapshot(future.value.pop()!);
  }

  // A drag emits a position update per frame, so history opens once when the
  // drag starts and is dropped at the end if the node never actually moved.
  function beginPositionChange() {
    pushHistory();
  }

  function endPositionChange() {
    const last = history.value[history.value.length - 1];
    if (last && last === serializeGraph(graph.value)) history.value.pop();
  }

  function setRunVisual(
    nodeStates: Record<string, RunNodeState>,
    edgeStates: Record<string, RunNodeState>,
    summary: string | null
  ) {
    runNodeStates.value = nodeStates;
    runEdgeStates.value = edgeStates;
    runSummary.value = summary;
  }

  function clearRunVisual() {
    runNodeStates.value = {};
    runEdgeStates.value = {};
    runSummary.value = null;
  }

  function toggleNodeDisabled(nodeId: string) {
    disabledNodeIds.value = disabledNodeIds.value.includes(nodeId)
      ? disabledNodeIds.value.filter((id) => id !== nodeId)
      : [...disabledNodeIds.value, nodeId];
    // A disabled block changes what the next run would reach, so the previous
    // paint no longer describes this graph.
    clearRunVisual();
  }

  function isNodeDisabled(nodeId: string): boolean {
    return disabledNodeIds.value.includes(nodeId);
  }

  function createGroup(nodeIds: string[], name?: string): string | null {
    const members = Array.from(new Set(nodeIds)).filter((id) =>
      graph.value?.nodes.some((node) => node.id === id)
    );
    if (members.length < 2) return null;

    const group: GraphGroup = {
      id: crypto.randomUUID(),
      name: name ?? `Group ${groups.value.length + 1}`,
      nodeIds: members,
    };
    groups.value = [...groups.value, group];
    return group.id;
  }

  function removeGroup(groupId: string) {
    groups.value = groups.value.filter((group) => group.id !== groupId);
  }

  function groupForNode(nodeId: string): GraphGroup | null {
    return groups.value.find((group) => group.nodeIds.includes(nodeId)) ?? null;
  }

  // Set validation result
  function setValidationResult(result: ValidationResult | null) {
    validationResult.value = result;
  }

  // Get errors for a specific node
  function getNodeErrors(nodeId: string): ValidationError[] {
    if (!validationResult.value) return [];
    return validationResult.value.errors.filter((e) => e.nodePath === nodeId);
  }

  // Run validation via backend (placeholder - will be wired in Phase 3)
  async function validateGraph() {
    if (!graph.value) return;
    // This will call the Tauri command in Phase 3
    // For now, do a basic structural check client-side
    const errors: ValidationError[] = [];

    // Check for at least one trigger
    const triggerCount = graph.value.nodes.filter(
      (n) => blocks.value.find((b) => b.id === n.blockId)?.kind === 'trigger'
    ).length;
    if (triggerCount === 0) {
      errors.push({
        severity: 'warning',
        code: 'NO_TRIGGER',
        message: 'Graph lacks an Input block. Every graph requires exactly one trigger.',
        suggestion: 'Drag an Input block onto the canvas.',
      });
    }

    // Check for cycles (simple DFS)
    const adj = new Map<string, string[]>();
    for (const edge of graph.value.edges) {
      const targets = adj.get(edge.source) ?? [];
      targets.push(edge.target);
      adj.set(edge.source, targets);
    }

    const visited = new Set<string>();
    const recStack = new Set<string>();

    function dfs(nodeId: string, path: string[]): boolean {
      visited.add(nodeId);
      recStack.add(nodeId);
      path.push(nodeId);

      const targets = adj.get(nodeId) ?? [];
      for (const target of targets) {
        if (recStack.has(target)) {
          errors.push({
            severity: 'error',
            code: 'CYCLE_DETECTED',
            message: `Cycle detected in graph involving node '${target}'. Agent graphs must be acyclic.`,
            suggestion: 'Remove the edge that creates the cycle.',
            nodePath: target,
          });
        } else if (!visited.has(target)) {
          dfs(target, path);
        }
      }

      path.pop();
      recStack.delete(nodeId);
      return false;
    }

    for (const node of graph.value.nodes) {
      if (!visited.has(node.id)) {
        dfs(node.id, []);
      }
    }

    setValidationResult({
      valid: errors.filter((e) => e.severity === 'error').length === 0,
      errors,
      graphHash: '', // Will be computed by backend
    });
  }

  // Drill into a sub-graph node
  function drillIntoNode(nodeId: string) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node?.subGraph) {
      breadcrumbs.value.push({ nodeId, name: node.blockId });
      const parentGraph = graph.value;
      graph.value = node.subGraph;
      (graph.value as any)._parentGraph = parentGraph;
      (graph.value as any)._parentNodeId = nodeId;
    }
  }

  // Drill out of sub-graph back to parent
  function drillOut() {
    const current = graph.value as any;
    if (current?._parentGraph) {
      graph.value = current._parentGraph;
      breadcrumbs.value.pop();
      selectedNodeId.value = current._parentNodeId ?? null;
      delete current._parentGraph;
      delete current._parentNodeId;
    }
  }

  // Expand a collapsed node
  function expandNode(nodeId: string) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node && node.isCollapsed) {
      pushHistory();
      node.isCollapsed = false;
      graph.value!.updatedAt = new Date().toISOString();
    }
  }

  // Collapse an expanded node
  function collapseNode(nodeId: string) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node && !node.isCollapsed) {
      pushHistory();
      node.isCollapsed = true;
      graph.value!.updatedAt = new Date().toISOString();
    }
  }

  // Run the graph (execution - wired in Phase 3)
  async function runGraph() {
    if (!graph.value) return;
    // This will call the Tauri command in Phase 3
    console.log('Running graph:', graph.value.id);
  }

  // Check if we're inside a sub-graph
  const isInSubGraph = computed(() => breadcrumbs.value.length > 0);

  const canUndo = computed(() => history.value.length > 0);
  const canRedo = computed(() => future.value.length > 0);

  // Error count
  const errorCount = computed(() => validationResult.value?.errors.filter((e) => e.severity === 'error').length ?? 0);

  return {
    graph,
    blocks,
    draggingBlock,
    selectedNodeId,
    validationResult,
    breadcrumbs,
    isInSubGraph,
    errorCount,
    canUndo,
    canRedo,
    undo,
    redo,
    beginPositionChange,
    endPositionChange,
    runNodeStates,
    runEdgeStates,
    runSummary,
    setRunVisual,
    clearRunVisual,
    disabledNodeIds,
    groups,
    toggleNodeDisabled,
    isNodeDisabled,
    createGroup,
    removeGroup,
    groupForNode,
    setGraph,
    createGraph,
    setDraggingBlock,
    addNode,
    removeNode,
    updateNodePosition,
    updateNodeParameters,
    addEdge,
    removeEdge,
    selectNode,
    setValidationResult,
    getNodeErrors,
    validateGraph,
    drillIntoNode,
    drillOut,
    expandNode,
    collapseNode,
    runGraph,
  };
});
