import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useAgentStore } from '@/stores/agentStore';
import { useGraphStore } from '@/stores/graphStore';
import { flushGraphSave, resetGraphSync } from '@/services/graphSync';
import { updateAgent } from '@/services/ipc';
import type { AgentGraph } from '@/types/agent';

vi.mock('@/services/ipc');

const mockUpdateAgent = vi.mocked(updateAgent);

const agent = {
  id: 'agent-1',
  name: 'Test Agent',
  description: null,
  specJson: {},
  model: 'ollama',
  isStarter: false,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

function makeGraph(): AgentGraph {
  return {
    id: 'graph-1',
    name: 'Canvas graph',
    schemaVersion: 1,
    nodes: [
      {
        id: 'n1',
        blockId: 'builtin:trigger',
        blockVersion: 1,
        position: { x: 0, y: 0 },
        parameters: {},
        isCollapsed: false,
      },
    ],
    edges: [],
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  } as AgentGraph;
}

function setup() {
  const agentStore = useAgentStore();
  const graphStore = useGraphStore();
  agentStore.state.currentAgent = { ...agent } as any;
  graphStore.setGraph(makeGraph());
  return { agentStore, graphStore };
}

describe('graphSync', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockUpdateAgent.mockReset();
    resetGraphSync();
  });

  it('writes the canvas into the agent row', async () => {
    setup();
    mockUpdateAgent.mockResolvedValue({ ...agent, updatedAt: '2026-01-01T00:00:02Z' } as any);

    expect(await flushGraphSave()).toBe(true);
    expect(mockUpdateAgent).toHaveBeenCalledTimes(1);

    const [id, input] = mockUpdateAgent.mock.calls[0];
    expect(id).toBe('agent-1');
    expect((input.specJson as AgentGraph).schemaVersion).toBe(1);
    expect((input.specJson as AgentGraph).nodes[0].blockId).toBe('builtin:trigger');
  });

  it('skips the write when the graph has not changed', async () => {
    setup();
    mockUpdateAgent.mockResolvedValue({ ...agent, updatedAt: '2026-01-01T00:00:02Z' } as any);

    expect(await flushGraphSave()).toBe(true);
    expect(await flushGraphSave()).toBe(true);
    expect(mockUpdateAgent).toHaveBeenCalledTimes(1);
  });

  it('reports a failed write so a run can refuse to start', async () => {
    const { agentStore } = setup();
    mockUpdateAgent.mockRejectedValue(new Error('database is locked'));

    expect(await flushGraphSave()).toBe(false);
    expect(agentStore.state.graphSaveError).toContain('database is locked');
  });
});
