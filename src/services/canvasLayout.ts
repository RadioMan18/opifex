import type { AgentGraph } from '@/types/agent';

export interface LayoutOptions {
  columnGap?: number;
  rowGap?: number;
  originX?: number;
  originY?: number;
}

/**
 * Layered layout: each node sits one column right of its furthest incoming
 * path, so an agent graph reads left-to-right in execution order. Layering
 * cannot place nodes that are part of a cycle, so those are parked in a column
 * after the layered graph rather than being left where they were.
 */
export function layeredLayout(
  graph: AgentGraph,
  options: LayoutOptions = {}
): Record<string, { x: number; y: number }> {
  const { columnGap = 320, rowGap = 200, originX = 80, originY = 80 } = options;

  const depth = new Map<string, number>();
  const remaining = new Map<string, number>();
  const outgoing = new Map<string, string[]>();

  for (const node of graph.nodes) {
    remaining.set(node.id, 0);
    outgoing.set(node.id, []);
  }

  for (const edge of graph.edges) {
    if (!remaining.has(edge.source) || !remaining.has(edge.target)) continue;
    remaining.set(edge.target, (remaining.get(edge.target) ?? 0) + 1);
    outgoing.get(edge.source)?.push(edge.target);
  }

  const queue = graph.nodes.filter((node) => remaining.get(node.id) === 0).map((node) => node.id);
  for (const nodeId of queue) depth.set(nodeId, 0);

  while (queue.length) {
    const nodeId = queue.shift()!;
    const level = depth.get(nodeId) ?? 0;
    for (const next of outgoing.get(nodeId) ?? []) {
      depth.set(next, Math.max(depth.get(next) ?? 0, level + 1));
      const left = (remaining.get(next) ?? 0) - 1;
      remaining.set(next, left);
      if (left === 0) queue.push(next);
    }
  }

  const cycleColumn = Math.max(-1, ...Array.from(depth.values())) + 1;
  const positions: Record<string, { x: number; y: number }> = {};
  const rows = new Map<number, number>();

  for (const node of graph.nodes) {
    const column = depth.get(node.id) ?? cycleColumn;
    const row = rows.get(column) ?? 0;
    rows.set(column, row + 1);
    positions[node.id] = { x: originX + column * columnGap, y: originY + row * rowGap };
  }

  return positions;
}
