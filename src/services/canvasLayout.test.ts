import { describe, expect, it } from 'vitest';
import { layeredLayout } from './canvasLayout';
import type { AgentGraph } from '@/types/agent';

function makeGraph(nodes: string[], edges: Array<[string, string]>): AgentGraph {
  return {
    id: 'g',
    name: 'g',
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

describe('layeredLayout', () => {
  it('puts a chain in consecutive columns', () => {
    const positions = layeredLayout(makeGraph(['a', 'b', 'c'], [['a', 'b'], ['b', 'c']]), {
      columnGap: 300,
      rowGap: 200,
      originX: 0,
      originY: 0,
    });

    expect(positions.a).toEqual({ x: 0, y: 0 });
    expect(positions.b).toEqual({ x: 300, y: 0 });
    expect(positions.c).toEqual({ x: 600, y: 0 });
  });

  it('stacks nodes that share a column', () => {
    const positions = layeredLayout(makeGraph(['a', 'b', 'c'], [['a', 'b'], ['a', 'c']]), {
      columnGap: 300,
      rowGap: 200,
      originX: 0,
      originY: 0,
    });

    expect(positions.a).toEqual({ x: 0, y: 0 });
    expect(positions.b).toEqual({ x: 300, y: 0 });
    expect(positions.c).toEqual({ x: 300, y: 200 });
  });

  it('places a node after its furthest input, not its first', () => {
    const positions = layeredLayout(
      makeGraph(['a', 'b', 'c', 'd'], [
        ['a', 'b'],
        ['b', 'c'],
        ['a', 'd'],
        ['c', 'd'],
      ]),
      { columnGap: 100, rowGap: 100, originX: 0, originY: 0 }
    );

    expect(positions.d).toEqual({ x: 300, y: 0 });
  });

  it('parks nodes trapped in a cycle in their own column', () => {
    const positions = layeredLayout(makeGraph(['a', 'b'], [['a', 'b'], ['b', 'a']]), {
      columnGap: 300,
      rowGap: 200,
      originX: 0,
      originY: 0,
    });

    // Nothing could be layered, so both land in the first column together.
    expect(positions.a?.x).toBe(0);
    expect(positions.b?.x).toBe(0);
    expect(positions.a?.y).not.toBe(positions.b?.y);
  });
});
