import { describe, it, expect } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useGraphStore } from '@/stores/graphStore';
import type { BlockDefinition, BlockKind } from '@/types/agent';

function makeBlock(id: string, kind: BlockKind = 'utility'): BlockDefinition {
  return {
    id,
    version: 1,
    name: `Block ${id}`,
    description: 'Test block',
    category: 'Test',
    kind,
    origin: 'user',
    inputs: [
      {
        id: 'input',
        name: 'Input',
        type: 'any',
        direction: 'input',
        required: true,
      },
    ],
    outputs: [
      {
        id: 'output',
        name: 'Output',
        type: 'any',
        direction: 'output',
        required: false,
      },
    ],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

function createStore() {
  setActivePinia(createPinia());
  const store = useGraphStore();
  store.setGraph({
    id: 'graph-1',
    name: 'Test Graph',
    description: undefined,
    schemaVersion: 1,
    nodes: [],
    edges: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  });
  store.blocks = [makeBlock('block-1', 'trigger'), makeBlock('block-2', 'llm')];
  return store;
}

describe('GraphStore', () => {
  it('adds a node to the graph', () => {
    const store = createStore();
    const nodeId = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 100, y: 200 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    });

    expect(nodeId).toBeDefined();
    const id = nodeId!;
    expect(store.graph!.nodes.length).toBe(1);
    expect(store.graph!.nodes[0]!.id).toBe(id);
    expect(store.graph!.nodes[0]!.position).toEqual({ x: 100, y: 200 });
  });

  it('adds an edge between nodes', () => {
    const store = createStore();
    const node1Id = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;
    const node2Id = store.addNode({
      blockId: 'block-2',
      blockVersion: 1,
      position: { x: 200, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    const edgeId = store.addEdge({
      source: node1Id,
      target: node2Id,
    })!;

    expect(edgeId).toBeDefined();
    expect(store.graph!.edges.length).toBe(1);
    expect(store.graph!.edges[0]!.source).toBe(node1Id);
    expect(store.graph!.edges[0]!.target).toBe(node2Id);
  });

  it('prevents duplicate edges', () => {
    const store = createStore();
    const node1Id = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;
    const node2Id = store.addNode({
      blockId: 'block-2',
      blockVersion: 1,
      position: { x: 200, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.addEdge({
      source: node1Id,
      target: node2Id,
    });
    store.addEdge({
      source: node1Id,
      target: node2Id,
    });

    expect(store.graph!.edges.length).toBe(1);
  });

  it('removes a node and its connected edges', () => {
    const store = createStore();
    const node1Id = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;
    const node2Id = store.addNode({
      blockId: 'block-2',
      blockVersion: 1,
      position: { x: 200, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.addEdge({
      source: node1Id,
      target: node2Id,
    });

    store.removeNode(node1Id);

    expect(store.graph!.nodes.length).toBe(1);
    expect(store.graph!.edges.length).toBe(0);
    expect(store.selectedNodeId).toBeNull();
  });

  it('updates node position', () => {
    const store = createStore();
    const nodeId = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.updateNodePosition(nodeId, { x: 300, y: 400 });
    const node = store.graph!.nodes.find((n) => n.id === nodeId);
    expect(node?.position).toEqual({ x: 300, y: 400 });
  });

  it('updates node parameters', () => {
    const store = createStore();
    const nodeId = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.updateNodeParameters(nodeId, { model: 'llama3.2' });
    const node = store.graph!.nodes.find((n) => n.id === nodeId);
    expect(node?.parameters['model']).toBe('llama3.2');
  });

  it('selects and deselects nodes', () => {
    const store = createStore();
    const nodeId = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.selectNode(nodeId);
    expect(store.selectedNodeId).toBe(nodeId);

    store.selectNode(null);
    expect(store.selectedNodeId).toBeNull();
  });

  it('validates graph with no trigger', () => {
    const store = createStore();
    store.addNode({
      blockId: 'block-2', // llm block, not a trigger
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    });

    store.validateGraph();
    expect(store.validationResult!.valid).toBe(true);
    expect(store.validationResult!.errors.some((e) => e.code === 'NO_TRIGGER')).toBe(true);
    expect(store.validationResult!.errors.some((e) => e.severity === 'warning')).toBe(true);
  });

  it('validates graph with trigger as valid', () => {
    const store = createStore();
    store.addNode({
      blockId: 'block-1', // trigger block
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    });

    store.validateGraph();
    expect(store.validationResult!.valid).toBe(true);
  });

  it('detects cycles', () => {
    const store = createStore();
    const n1 = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;
    const n2 = store.addNode({
      blockId: 'block-2',
      blockVersion: 1,
      position: { x: 200, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    store.addEdge({ source: n1, target: n2 });
    store.addEdge({ source: n2, target: n1 });

    store.validateGraph();
    expect(store.validationResult!.valid).toBe(false);
    expect(store.validationResult!.errors.some((e) => e.code === 'CYCLE_DETECTED')).toBe(true);
  });

  it('drill into sub-graph and out', () => {
    const store = createStore();
    const parentNodeId = store.addNode({
      blockId: 'block-1',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: {
        id: 'sub-1',
        name: 'Sub Graph',
        description: undefined,
        schemaVersion: 1,
        nodes: [],
        edges: [],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
      isCollapsed: false,
    })!;

    expect(store.isInSubGraph).toBe(false);

    store.drillIntoNode(parentNodeId);
    expect(store.isInSubGraph).toBe(true);
    expect(store.breadcrumbs.length).toBe(1);

    store.drillOut();
    expect(store.isInSubGraph).toBe(false);
    expect(store.breadcrumbs.length).toBe(0);
  });
});
