<template>
  <div class="run-history p-4 bg-slate-900 border-t border-slate-800 text-slate-200">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold">Run History</h3>
      <button
        @click="refresh"
        class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-slate-200"
      >
        Refresh
      </button>
    </div>

    <p v-if="!agentId" class="text-slate-500 text-sm">Select an agent to see its runs.</p>
    <p v-else-if="loading && runs.length === 0" class="text-slate-500 text-sm">Loading runs…</p>
    <p v-else-if="runs.length === 0" class="text-slate-500 text-sm">No runs yet. Press Run Agent.</p>

    <ul v-else class="space-y-1.5">
      <li v-for="run in runs" :key="run.id">
        <button
          class="w-full text-left px-2 py-1.5 rounded border"
          :class="
            selectedId === run.id
              ? 'bg-slate-800 border-sky-600'
              : 'bg-slate-800/50 border-slate-700 hover:border-slate-600'
          "
          @click="select(run)"
        >
          <span class="flex items-center justify-between gap-2">
            <span :class="['text-[10px] font-mono px-1.5 py-0.5 rounded', statusClass(run.status)]">
              {{ run.status }}
            </span>
            <span class="text-[11px] text-slate-400">{{ formatRunStarted(run.startedAt) }}</span>
          </span>
          <span class="block mt-1 text-[11px] text-slate-400">
            {{ run.nodeCount }} node{{ run.nodeCount === 1 ? '' : 's' }}
            <template v-if="formatRunDuration(run)"> · {{ formatRunDuration(run) }}</template>
            <template v-if="run.mode !== 'live'"> · {{ run.mode }}</template>
          </span>
          <span
            v-if="describeStoredError(run.error)"
            class="block mt-1 text-[11px] text-red-400 break-words"
          >
            {{ describeStoredError(run.error) }}
          </span>
        </button>
      </li>
    </ul>

    <div v-if="detail" class="mt-3">
      <p class="text-xs font-semibold mb-1">Nodes</p>
      <ul class="space-y-1">
        <li
          v-for="node in detail.nodes"
          :key="node.nodeId"
          class="px-2 py-1.5 rounded bg-slate-800 border border-slate-700"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-[11px] font-mono text-slate-300 truncate">{{ nodeLabel(node.nodeId) }}</span>
            <span :class="['text-[10px] font-mono px-1.5 py-0.5 rounded', statusClass(node.status)]">
              {{ node.status }}
            </span>
          </div>
          <p
            v-if="node.tokensIn !== null || node.tokensOut !== null"
            class="mt-0.5 text-[10px] text-slate-500"
          >
            tokens {{ node.tokensIn ?? 0 }} in / {{ node.tokensOut ?? 0 }} out
          </p>
          <p v-if="describeStoredError(node.error)" class="mt-1 text-[11px] text-red-400 break-words">
            {{ describeStoredError(node.error) }}
          </p>
          <pre
            v-if="previewNodeOutput(node.output)"
            class="mt-1 text-[11px] text-slate-300 whitespace-pre-wrap break-words font-mono"
          >{{ previewNodeOutput(node.output) }}</pre>
        </li>
      </ul>
    </div>

    <p v-if="error" class="mt-3 p-2 rounded bg-red-500/10 border border-red-500/20 text-red-400 text-xs">
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { getRun, listRuns } from '@/services/ipc';
import { useGraphStore } from '@/stores/graphStore';
import {
  describeStoredError,
  formatRunDuration,
  formatRunStarted,
  nodeStatesFromRun,
  previewNodeOutput,
  type RunDetail,
  type RunSummary,
} from '@/services/runHistory';

interface Props {
  agentId: string | undefined;
}
const props = defineProps<Props>();

const graphStore = useGraphStore();

const runs = ref<RunSummary[]>([]);
const detail = ref<RunDetail | null>(null);
const selectedId = ref<string | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

async function refresh() {
  if (!props.agentId) {
    runs.value = [];
    return;
  }

  loading.value = true;
  error.value = null;
  try {
    const history = await listRuns(props.agentId);
    runs.value = history;

    if (selectedId.value && !history.some((run) => run.id === selectedId.value)) {
      selectedId.value = null;
      detail.value = null;
      graphStore.clearRunVisual();
    }
  } catch (err: any) {
    error.value = err?.message || String(err);
  } finally {
    loading.value = false;
  }
}

async function select(run: RunSummary) {
  selectedId.value = run.id;
  error.value = null;

  try {
    const loaded = await getRun(run.id);
    detail.value = loaded;
    if (loaded) paint(loaded);
  } catch (err: any) {
    error.value = err?.message || String(err);
  }
}

// Replaying a stored run repaints the canvas from what was recorded for each
// node, rather than re-deriving it from the graph.
function paint(loaded: RunDetail) {
  graphStore.setRunVisual(
    nodeStatesFromRun(loaded.nodes),
    {},
    `${loaded.summary.status === 'succeeded' ? 'Recorded run' : 'Recorded failed run'} · ${formatRunStarted(
      loaded.summary.startedAt,
    )}`,
  );
}

function nodeLabel(nodeId: string): string {
  const node = graphStore.graph?.nodes.find((candidate) => candidate.id === nodeId);
  if (!node) return nodeId;
  const block = graphStore.blocks.find((candidate) => candidate.id === node.blockId);
  return block ? `${block.name} · ${nodeId.slice(0, 6)}` : nodeId;
}

function statusClass(status: string): string {
  switch (status) {
    case 'succeeded':
      return 'bg-green-500/20 text-green-400';
    case 'failed':
      return 'bg-red-500/20 text-red-400';
    case 'running':
      return 'bg-blue-500/20 text-blue-400';
    case 'cancelled':
      return 'bg-yellow-500/20 text-yellow-400';
    default:
      return 'bg-slate-700 text-slate-300';
  }
}

onMounted(refresh);

watch(
  () => props.agentId,
  () => {
    selectedId.value = null;
    detail.value = null;
    graphStore.clearRunVisual();
    void refresh();
  },
);

// Finishing a run repaints the canvas, which changes the summary; reload then
// so the run that just happened appears without a manual refresh.
watch(
  () => graphStore.runSummary,
  () => {
    if (props.agentId) void refresh();
  },
);
</script>

<style scoped>
</style>
