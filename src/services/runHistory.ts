import type { RunNodeState } from '@/stores/graphStore';

export type RunRecordStatus = 'running' | 'succeeded' | 'failed' | 'cancelled';

export interface RunSummary {
  id: string;
  agentId: string;
  status: RunRecordStatus | string;
  mode: string;
  startedAt: string;
  endedAt: string | null;
  nodeCount: number;
  error: unknown;
}

export interface RunNodeRecord {
  nodeId: string;
  status: string;
  output: unknown;
  error: unknown;
  tokensIn: number | null;
  tokensOut: number | null;
  startedAt: string | null;
  endedAt: string | null;
}

export interface RunDetail {
  summary: RunSummary;
  nodes: RunNodeRecord[];
}

/** Statuses the canvas understands; anything else reads as no paint. */
export function toRunNodeState(status: string): RunNodeState | null {
  switch (status) {
    case 'succeeded':
      return 'succeeded';
    case 'failed':
      return 'failed';
    case 'running':
      return 'running';
    case 'skipped':
    case 'cancelled':
      return 'skipped';
    default:
      return null;
  }
}

/** Canvas paint data for a stored run, straight from what was recorded. */
export function nodeStatesFromRun(nodes: RunNodeRecord[]): Record<string, RunNodeState> {
  const states: Record<string, RunNodeState> = {};
  for (const node of nodes) {
    const state = toRunNodeState(node.status);
    if (state) states[node.nodeId] = state;
  }
  return states;
}

/** `2026-11-12T01:43:10Z` → `12 Nov 2026, 01:43`, in the viewer's locale. */
export function formatRunStarted(startedAt: string): string {
  const date = new Date(startedAt);
  if (Number.isNaN(date.getTime())) return startedAt;
  return date.toLocaleString(undefined, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/** Compact duration for a finished run, e.g. `3.4s`; null while it is running. */
export function formatRunDuration(summary: RunSummary): string | null {
  if (!summary.endedAt) return null;
  const start = new Date(summary.startedAt).getTime();
  const end = new Date(summary.endedAt).getTime();
  if (Number.isNaN(start) || Number.isNaN(end) || end < start) return null;

  const seconds = (end - start) / 1000;
  if (seconds < 10) return `${seconds.toFixed(1)}s`;
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${Math.round(seconds % 60)}s`;
}

/** Human text for a stored error, which may be a string or a JSON value. */
export function describeStoredError(error: unknown): string | null {
  if (error === null || error === undefined) return null;
  if (typeof error === 'string') return error.trim() || null;
  if (typeof error === 'object') {
    const message = (error as Record<string, unknown>).message;
    if (typeof message === 'string' && message.trim()) return message;
    try {
      return JSON.stringify(error);
    } catch {
      return null;
    }
  }
  return String(error);
}

/** The text a node produced, for the history list. */
export function previewNodeOutput(output: unknown, limit = 240): string | null {
  if (output === null || output === undefined) return null;
  const text =
    typeof output === 'string'
      ? output
      : JSON.stringify(output, null, 2) ?? String(output);
  const collapsed = text.trim();
  if (!collapsed) return null;
  return collapsed.length > limit ? `${collapsed.slice(0, limit)}…` : collapsed;
}
