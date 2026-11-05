<template>
  <div class="run-control p-4 bg-slate-900 border-t border-slate-800 text-slate-200">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold">Run Control</h3>
      <span
        :class="[
          'text-xs font-mono px-2 py-1 rounded',
          runStatus ? runStatusColor(runStatus) : 'bg-slate-700 text-slate-300',
        ]"
      >
        {{ runStatusLabel(runStatus) }}
      </span>
    </div>

    <div class="flex gap-2">
      <button
        v-if="!isRunning"
        @click="runAgent"
        class="flex-1 px-3 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors"
        :disabled="!agentId"
        :title="agentId ? 'Run the agent graph' : 'No agent selected'"
      >
        Run Agent
      </button>
      <button
        v-if="isRunning"
        @click="cancelRun"
        class="px-3 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-medium transition-colors"
      >
        Cancel
      </button>
    </div>

    <div v-if="runError" class="mt-3 p-2.5 rounded bg-red-500/10 border border-red-500/20 text-red-400 text-xs">
      {{ runError }}
    </div>

    <div v-if="runResult" class="mt-3 p-2.5 rounded bg-green-500/10 border border-green-500/20 text-green-400 text-xs">
      Run completed successfully
    </div>

    <div
      v-if="graphStore.runSummary"
      class="mt-3 flex items-start justify-between gap-3 p-2.5 rounded bg-slate-800 border border-slate-700 text-slate-300 text-xs"
    >
      <span>{{ graphStore.runSummary }}</span>
      <button class="text-slate-400 hover:text-slate-200" @click="clearPaint">Clear</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { runAgentGraph } from '@/services/ipc';
import { useGraphStore } from '@/stores/graphStore';
import { deriveRunTrace, type RunOutcomeStatus } from '@/services/runTrace';

interface Props {
  agentId: string | undefined;
}
const props = defineProps<Props>();

const graphStore = useGraphStore();

const isRunning = ref(false);
const runStatus = ref<string | null>(null);
const runError = ref<string | null>(null);
const runResult = ref<any | null>(null);

async function runAgent() {
  if (!props.agentId) return;

  isRunning.value = true;
  runError.value = null;
  runResult.value = null;

  try {
    const run = await runAgentGraph({
      agent_id: props.agentId,
      mode: 'mock', // Start with mock mode for safety
    });
    runResult.value = run;
    runStatus.value = run.status;
    paint(run.status === 'cancelled' ? 'cancelled' : 'succeeded');
  } catch (e: any) {
    runError.value = e.message || 'Unknown error';
    runStatus.value = 'failed';
    paint('failed', runError.value);
  } finally {
    isRunning.value = false;
  }
}

// The backend returns only the run header, so the canvas is painted from the
// graph plus what the run told us (see deriveRunTrace).
function paint(status: RunOutcomeStatus, error: string | null = null) {
  const graph = graphStore.graph;
  if (!graph) return;
  const trace = deriveRunTrace(graph, { status, error }, { disabled: graphStore.disabledNodeIds });
  graphStore.setRunVisual(trace.nodeStates, trace.edgeStates, trace.summary);
}

function clearPaint() {
  graphStore.clearRunVisual();
  runResult.value = null;
  runError.value = null;
  runStatus.value = null;
}

async function cancelRun() {
  isRunning.value = false;
}

function runStatusColor(status: string | null): string {
  switch (status) {
    case 'succeeded': return 'bg-green-500/20 text-green-400';
    case 'failed': return 'bg-red-500/20 text-red-400';
    case 'running': return 'bg-blue-500/20 text-blue-400';
    case 'cancelled': return 'bg-yellow-500/20 text-yellow-400';
    default: return 'bg-slate-700 text-slate-300';
  }
}

function runStatusLabel(status: string | null): string {
  switch (status) {
    case 'succeeded': return 'Succeeded';
    case 'failed': return 'Failed';
    case 'running': return 'Running';
    case 'cancelled': return 'Cancelled';
    default: return 'Idle';
  }
}
</script>

<style scoped>
</style>
