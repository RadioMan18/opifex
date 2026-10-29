<template>
  <div class="pin-panel p-4 bg-slate-900 border-t border-slate-800 text-slate-200">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold">Pinned Data</h3>
      <button
        @click="refreshPins"
        class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-slate-200"
      >
        Refresh
      </button>
    </div>

    <div v-if="pinnedData.length === 0" class="flex items-center justify-center py-6">
      <p class="text-slate-500 text-sm">No pinned data</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="pin in pinnedData"
        :key="`${pin.node_id}:${pin.port_id}`"
        class="p-2.5 rounded bg-slate-800 border border-slate-700"
      >
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs font-mono text-slate-400 truncate max-w-[200px]">
            {{ pin.node_id }}:{{ pin.port_id }}
          </span>
          <button
            @click="unpin(pin.node_id, pin.port_id)"
            class="text-[10px] text-red-400 hover:text-red-300 hover:underline"
          >
            Unpin
          </button>
        </div>
        <div class="text-[10px] text-slate-300 break-all">
          {{ formatValue(pin.value) }}
        </div>
        <div class="text-[10px] text-slate-500 mt-1">
          {{ pin.created_at }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { pinAgentData, unpinAgentData, getPinnedData } from '@/services/ipc';

interface Props {
  agentId: string | undefined;
  nodeId: string | undefined;
  portId: string | undefined;
  value: any | undefined;
}
const props = defineProps<Props>();

const pinnedData = ref<any[]>([]);

async function refreshPins() {
  if (!props.agentId) return;
  try {
    const data = await getPinnedData({ agent_id: props.agentId });
    pinnedData.value = data;
  } catch (e: any) {
    console.error('Failed to load pinned data:', e);
  }
}

async function pin() {
  if (!props.agentId || !props.nodeId || !props.portId) return;

  try {
    await pinAgentData({
      agent_id: props.agentId,
      node_id: props.nodeId,
      port_id: props.portId,
      value: props.value,
    });
    await refreshPins();
  } catch (e: any) {
    console.error('Failed to pin data:', e);
  }
}

async function unpin(nodeId: string, portId: string) {
  if (!props.agentId) return;

  try {
    await unpinAgentData({
      agent_id: props.agentId,
      node_id,
      port_id,
    });
    await refreshPins();
  } catch (e: any) {
    console.error('Failed to unpin data:', e);
  }
}

function formatValue(value: any): string {
  if (value === null) return 'null';
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return JSON.stringify(value);
}

onMounted(() => {
  refreshPins();
});
</script>

<style scoped>
</style>
