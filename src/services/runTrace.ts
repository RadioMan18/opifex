import type { AgentGraph } from '@/types/agent';
import type { RunNodeState } from '@/stores/graphStore';

export type RunOutcomeStatus = 'succeeded' | 'failed' | 'cancelled';

export interface RunOutcome {
  status: RunOutcomeStatus;
  /** Rejection message from `run_agent_graph`, if the run threw. */
  error?: string | null;
}

export interface RunTrace {
  nodeStates: Record<string, RunNodeState>;
  edgeStates: Record<string, RunNodeState>;
  summary: string | null;
}

/**
 * Derive what to paint on the canvas for a finished run.
 *
 * `run_agent_graph` returns only the run header: the engine tracks a `NodeRun`
 * per node but never exposes or persists it through this command. A successful
 * run therefore means every node in execution order ran, and a failed run
 * rejects with "Execution stopped at node <id>: <reason>", which names the node
 * that broke. Both are read here rather than invented: when the failure message
 * carries no id, nothing is marked failed.
 */
export interface RunTraceOptions {
  /** Blocks the canvas excludes from the graph; they never run, so they read as skipped. */
  disabled?: string[];
}

export function deriveRunTrace(
  graph: AgentGraph,
  outcome: RunOutcome,
  options: RunTraceOptions = {}
): RunTrace {
  const disabled = new Set(options.disabled ?? []);
  const order = executionOrder(graph);
  const nodeStates: Record<string, RunNodeState> = {};
  const edgeStates: Record<string, RunNodeState> = {};

  if (outcome.status === 'succeeded') {
    for (const nodeId of order) {
      nodeStates[nodeId] = disabled.has(nodeId) ? 'skipped' : 'succeeded';
    }
    for (const edge of graph.edges) {
      edgeStates[edge.id] =
        nodeStates[edge.source] === 'succeeded' && nodeStates[edge.target] === 'succeeded'
          ? 'succeeded'
          : 'skipped';
    }
    const count = order.length;
    return {
      nodeStates,
      edgeStates,
      summary: `${count} node${count === 1 ? '' : 's'} executed`,
    };
  }

  if (outcome.status === 'cancelled') {
    for (const nodeId of order) nodeStates[nodeId] = 'skipped';
    for (const edge of graph.edges) edgeStates[edge.id] = 'skipped';
    return { nodeStates, edgeStates, summary: 'Run cancelled' };
  }

  const failedNodeId = failingNodeId(outcome.error, graph);
  if (!failedNodeId) {
    // No node in the message: the run failed before or outside the graph, so
    // claiming any node broke would be a guess.
    for (const nodeId of order) nodeStates[nodeId] = 'skipped';
    for (const edge of graph.edges) edgeStates[edge.id] = 'skipped';
    return { nodeStates, edgeStates, summary: `Run failed: ${cleanError(outcome.error)}` };
  }

  const failureIndex = order.indexOf(failedNodeId);
  order.forEach((nodeId, index) => {
    if (disabled.has(nodeId)) nodeStates[nodeId] = 'skipped';
    else if (index < failureIndex) nodeStates[nodeId] = 'succeeded';
    else if (index === failureIndex) nodeStates[nodeId] = 'failed';
    else nodeStates[nodeId] = 'skipped';
  });
  for (const edge of graph.edges) {
    if (edgeStates[edge.id]) continue;
    const sourceState = nodeStates[edge.source];
    const targetState = nodeStates[edge.target];
    edgeStates[edge.id] =
      targetState === 'failed'
        ? 'failed'
        : sourceState === 'succeeded' && targetState === 'succeeded'
          ? 'succeeded'
          : 'skipped';
  }

  return {
    nodeStates,
    edgeStates,
    summary: `Run failed at ${failedNodeId}: ${cleanError(outcome.error)}`,
  };
}

/**
 * Kahn's algorithm. Nodes left over from a cycle are omitted: the backend
 * refuses cyclic graphs, and reporting a cycle member as executed would be
 * worse than leaving it unpainted.
 */
export function executionOrder(graph: AgentGraph): string[] {
  const incoming = new Map<string, number>();
  const outgoing = new Map<string, string[]>();

  for (const node of graph.nodes) {
    incoming.set(node.id, 0);
    outgoing.set(node.id, []);
  }

  for (const edge of graph.edges) {
    if (!incoming.has(edge.source) || !incoming.has(edge.target)) continue;
    incoming.set(edge.target, (incoming.get(edge.target) ?? 0) + 1);
    outgoing.get(edge.source)?.push(edge.target);
  }

  const queue = graph.nodes.filter((node) => incoming.get(node.id) === 0).map((node) => node.id);
  const order: string[] = [];

  while (queue.length) {
    const nodeId = queue.shift()!;
    order.push(nodeId);
    for (const next of outgoing.get(nodeId) ?? []) {
      const remaining = (incoming.get(next) ?? 0) - 1;
      incoming.set(next, remaining);
      if (remaining === 0) queue.push(next);
    }
  }

  return order;
}

function failingNodeId(error: string | null | undefined, graph: AgentGraph): string | null {
  if (!error) return null;
  const match = /stopped at node ([^\s:]+)/i.exec(error);
  if (!match) return null;
  return graph.nodes.some((node) => node.id === match[1]) ? match[1] : null;
}

function cleanError(error: string | null | undefined): string {
  if (!error) return 'run failed';
  const stripped = error.replace(/^Execution stopped at node [^\s:]+:\s*/i, '').trim();
  return stripped || 'run failed';
}
