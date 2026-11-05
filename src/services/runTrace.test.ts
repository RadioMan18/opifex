import { describe, expect, it } from 'vitest';
import { deriveRunTrace, executionOrder } from './runTrace';
import type { AgentGraph } from '@/types/agent';

function makeGraph(nodes: string[], edges: Array<[string, string]>): AgentGraph {
  return {
    id: 'graph-1',
    name: 'graph',
    schemaVersion: 1,
    nodes: nodes.map((id) => ({
      id,
      blockId: 'block',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      isCollapsed: false,
    })),
    edges: edges.map(([source, target], index) => ({ id: `e${index}`, source, target })),
    createdAt: '2026-01-01T00:00:00.000Z',
    updatedAt: '2026-01-01T00:00:00.000Z',
  };
}

describe('executionOrder', () => {
  it('orders a chain from its root', () => {
    expect(executionOrder(makeGraph(['a', 'b', 'c'], [['a', 'b'], ['b', 'c']]))).toEqual([
      'a',
      'b',
      'c',
    ]);
  });

  it('omits nodes trapped in a cycle', () => {
    expect(executionOrder(makeGraph(['a', 'b'], [['a', 'b'], ['b', 'a']]))).toEqual([]);
  });
});

describe('deriveRunTrace', () => {
  it('marks every executed node and edge on success', () => {
    const trace = deriveRunTrace(makeGraph(['a', 'b', 'c'], [['a', 'b'], ['b', 'c']]), {
      status: 'succeeded',
    });

    expect(trace.nodeStates).toEqual({ a: 'succeeded', b: 'succeeded', c: 'succeeded' });
    expect(trace.edgeStates).toEqual({ e0: 'succeeded', e1: 'succeeded' });
    expect(trace.summary).toBe('3 nodes executed');
  });

  it('splits a failed run around the node named in the rejection', () => {
    const trace = deriveRunTrace(makeGraph(['a', 'b', 'c'], [['a', 'b'], ['b', 'c']]), {
      status: 'failed',
      error: 'Execution stopped at node b: model unavailable',
    });

    expect(trace.nodeStates).toEqual({ a: 'succeeded', b: 'failed', c: 'skipped' });
    expect(trace.edgeStates).toEqual({ e0: 'failed', e1: 'skipped' });
    expect(trace.summary).toBe('Run failed at b: model unavailable');
  });

  it('does not accuse a node when the error names none', () => {
    const trace = deriveRunTrace(makeGraph(['a', 'b'], [['a', 'b']]), {
      status: 'failed',
      error: 'Execution failed: agent not found',
    });

    expect(trace.nodeStates).toEqual({ a: 'skipped', b: 'skipped' });
    expect(trace.summary).toBe('Run failed: Execution failed: agent not found');
  });

  it('ignores a node id that is not in this graph', () => {
    const trace = deriveRunTrace(makeGraph(['a'], []), {
      status: 'failed',
      error: 'Execution stopped at node zzz: boom',
    });

    expect(trace.nodeStates).toEqual({ a: 'skipped' });
  });

  it('reports a disabled block as skipped even on success', () => {
    const trace = deriveRunTrace(
      makeGraph(['a', 'b', 'c'], [['a', 'b'], ['b', 'c']]),
      { status: 'succeeded' },
      { disabled: ['b'] }
    );

    expect(trace.nodeStates).toEqual({ a: 'succeeded', b: 'skipped', c: 'succeeded' });
    expect(trace.edgeStates).toEqual({ e0: 'skipped', e1: 'skipped' });
  });

  it('marks a cancelled run as skipped', () => {
    const trace = deriveRunTrace(makeGraph(['a', 'b'], [['a', 'b']]), { status: 'cancelled' });

    expect(trace.nodeStates).toEqual({ a: 'skipped', b: 'skipped' });
    expect(trace.edgeStates).toEqual({ e0: 'skipped' });
    expect(trace.summary).toBe('Run cancelled');
  });
});
