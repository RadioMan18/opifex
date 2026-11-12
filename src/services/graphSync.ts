import { useAgentStore } from '@/stores/agentStore';
import { useGraphStore } from '@/stores/graphStore';
import type { AgentGraph } from '@/types/agent';

// The canvas and the stored agent spec are one document: a run loads the row
// from SQLite, so every canvas edit is written back to that row. Writes are
// debounced, because dragging a node would otherwise be one write per frame,
// and flushed before a run so a run never executes a graph older than the one
// on screen.
const DEBOUNCE_MS = 500;

let timer: ReturnType<typeof setTimeout> | null = null;
let inFlight: Promise<boolean> | null = null;
let lastSaved: string | null = null;

/** A snapshot safe to hand to IPC: reactive objects are not structured-clonable. */
function snapshot(): AgentGraph | null {
  const graphStore = useGraphStore();
  return graphStore.graph
    ? (JSON.parse(JSON.stringify(graphStore.graph)) as AgentGraph)
    : null;
}

// The timestamps are rewritten by every save, so they are excluded from the
// comparison; otherwise a write would always look like a change.
function fingerprint(graph: AgentGraph): string {
  return JSON.stringify({ ...graph, createdAt: '', updatedAt: '' });
}

export function queueGraphSave(): void {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    timer = null;
    void flushGraphSave();
  }, DEBOUNCE_MS);
}

/** Write the canvas if it differs from what is stored. Returns false on failure. */
export async function flushGraphSave(): Promise<boolean> {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  if (inFlight) await inFlight;

  const graph = snapshot();
  const agentStore = useAgentStore();
  if (!graph || !agentStore.state.currentAgent) return true;
  if (fingerprint(graph) === lastSaved) return true;

  inFlight = agentStore.saveAgentGraph({
    ...graph,
    updatedAt: new Date().toISOString(),
  });
  const saved = await inFlight;
  inFlight = null;

  if (saved) lastSaved = fingerprint(graph);
  return saved;
}

/** Forget what is stored, so the next flush writes even an identical graph. */
export function resetGraphSync(): void {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  lastSaved = null;
}

/** Record the graph as already stored, so loading an agent does not write it back. */
export function markGraphSynced(): void {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  const graph = snapshot();
  lastSaved = graph ? fingerprint(graph) : null;
}
