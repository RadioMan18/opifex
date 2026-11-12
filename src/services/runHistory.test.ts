import { describe, it, expect } from 'vitest';
import {
  describeStoredError,
  formatRunDuration,
  formatRunStarted,
  nodeStatesFromRun,
  previewNodeOutput,
  toRunNodeState,
  type RunSummary,
} from '@/services/runHistory';

const baseSummary: RunSummary = {
  id: 'run-1',
  agentId: 'agent-1',
  status: 'failed',
  mode: 'live',
  startedAt: '2026-11-12T01:43:10Z',
  endedAt: '2026-11-12T01:43:13.4Z',
  nodeCount: 3,
  error: 'Execution stopped at node n2: missing message',
};

describe('runHistory', () => {
  it('maps stored statuses onto canvas states', () => {
    expect(toRunNodeState('succeeded')).toBe('succeeded');
    expect(toRunNodeState('failed')).toBe('failed');
    expect(toRunNodeState('running')).toBe('running');
    expect(toRunNodeState('cancelled')).toBe('skipped');
    expect(toRunNodeState('queued')).toBeNull();
  });

  it('builds canvas states from recorded node runs', () => {
    const states = nodeStatesFromRun([
      { nodeId: 'n1', status: 'succeeded' } as any,
      { nodeId: 'n2', status: 'failed' } as any,
      { nodeId: 'n3', status: 'queued' } as any,
    ]);

    expect(states).toEqual({ n1: 'succeeded', n2: 'failed' });
  });

  it('formats duration and started time', () => {
    expect(formatRunDuration(baseSummary)).toBe('3.4s');
    expect(formatRunDuration({ ...baseSummary, endedAt: null })).toBeNull();
    expect(formatRunStarted('not a date')).toBe('not a date');
    expect(formatRunStarted('2026-11-12T01:43:10Z')).toContain('2026');
  });

  it('reads errors that were stored as strings or JSON', () => {
    expect(describeStoredError('boom')).toBe('boom');
    expect(describeStoredError({ message: 'boom' })).toBe('boom');
    expect(describeStoredError({ code: 7 })).toBe('{"code":7}');
    expect(describeStoredError(null)).toBeNull();
  });

  it('previews node output and truncates long values', () => {
    expect(previewNodeOutput({ message: 'hi' })).toContain('"hi"');
    expect(previewNodeOutput('x'.repeat(300))?.endsWith('…')).toBe(true);
    expect(previewNodeOutput(undefined)).toBeNull();
    expect(previewNodeOutput('   ')).toBeNull();
  });
});
