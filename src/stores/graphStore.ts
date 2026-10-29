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

export const useGraphStore = defineStore('graph', () => {
  // Current graph being edited
  const graph = ref<AgentGraph | null>(null);

  // Available block definitions (from backend/library)
  const blocks = ref<BlockDefinition[]>([]);

  // Currently selected node on canvas
  const selectedNodeId = ref<string | null>(null);

  // Validation result for current graph
  const validationResult = ref<ValidationResult | null>(null);

  // Breadcrumbs for sub-graph drill-in navigation
  const breadcrumbs = ref<Array<{ nodeId: string; name: string }>>([]);

  // Set graph (load from backend or create new)
  function setGraph(newGraph: AgentGraph) {
    graph.value = newGraph;
    selectedNodeId.value = null;
    breadcrumbs.value = [];
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
  }

  // Add a node to the graph
  function addNode(node: Omit<NodeInstance, 'id'> & { position: { x: number; y: number } }) {
    if (!graph.value) return;
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
    graph.value.nodes = graph.value.nodes.filter((n) => n.id !== nodeId);
    graph.value.edges = graph.value.edges.filter(
      (e) => e.source !== nodeId && e.target !== nodeId
    );
    if (selectedNodeId.value === nodeId) {
      selectedNodeId.value = null;
    }
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
    graph.value.edges = graph.value.edges.filter((e) => e.id !== edgeId);
    graph.value.updatedAt = new Date().toISOString();
  }

  // Select a node
  function selectNode(nodeId: string | null) {
    selectedNodeId.value = nodeId;
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
      node.isCollapsed = false;
      graph.value!.updatedAt = new Date().toISOString();
    }
  }

  // Collapse an expanded node
  function collapseNode(nodeId: string) {
    const node = graph.value?.nodes.find((n) => n.id === nodeId);
    if (node && !node.isCollapsed) {
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

  // Error count
  const errorCount = computed(() => validationResult.value?.errors.filter((e) => e.severity === 'error').length ?? 0);

  return {
    graph,
    blocks,
    selectedNodeId,
    validationResult,
    breadcrumbs,
    isInSubGraph,
    errorCount,
    setGraph,
    createGraph,
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
