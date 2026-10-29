<template>
  <div
    class="w-72 p-3 rounded-lg border shadow-md flex flex-col gap-2 group"
    :style="computedStyle"
  >
    <div class="flex items-center justify-between mb-1">
      <h4 class="text-xs font-bold text-slate-100 truncate">{{ data.label }}</h4>
      <span
        v-if="data.hasError"
        class="w-1.5 h-1.5 rounded-full bg-red-500"
        title="Validation error"
        aria-label="Error"
      ></span>
    </div>

    <!-- Connector type badge -->
    <div class="flex items-center gap-1 mb-2">
      <span class="w-2 h-2 rounded-full bg-blue-500"></span>
      <span class="text-[10px] text-blue-400 font-mono">{{ data.block?.category }}</span>
    </div>

    <!-- Account Selection -->
    <div class="p-2 rounded bg-slate-800/50 border border-slate-700/50 mb-2">
      <div class="text-[10px] text-slate-400 uppercase tracking-wider mb-1">
        Account Selection
      </div>
      <select
        v-model="accountSelection"
        class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
        @change="updateAccountSelection(($event.target as HTMLSelectElement).value)"
      >
        <option value="personal-default">Personal Default</option>
        <option value="specific-account">Specific Account</option>
        <option value="team-default">Team Default</option>
      </select>
    </div>

    <!-- Input ports -->
    <div class="flex items-start gap-3 mb-2">
      <div class="flex flex-col gap-1.5 flex-1">
        <template v-for="input in data.block?.inputs || []" :key="input.id">
          <div class="flex items-center gap-1.5">
            <div class="w-2 h-2 rounded-full bg-blue-400" :title="input.name"></div>
            <span class="text-[10px] text-slate-400 truncate max-w-[120px]" :title="input.name">
              {{ input.name }}
            </span>
          </div>
        </template>
      </div>
    </div>

    <!-- Output ports -->
    <div class="flex items-end gap-3 mt-2">
      <div class="flex flex-col gap-1.5 flex-1">
        <template v-for="output in data.block?.outputs || []" :key="output.id">
          <div class="flex items-center gap-1.5 justify-end">
            <span class="text-[10px] text-slate-400 truncate max-w-[120px]" :title="output.name">
              {{ output.name }}
            </span>
            <div class="w-2 h-2 rounded-full bg-green-400"></div>
          </div>
        </template>
      </div>
    </div>

    <!-- Tool Management status -->
    <div class="mt-2 p-2 rounded bg-slate-800/30 border border-slate-700/30">
      <div class="text-[10px] text-slate-400 uppercase tracking-wider mb-1">
        Tool Management
      </div>
      <div class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full" :style="{ backgroundColor: tmColor }"></span>
        <span class="text-[10px] text-slate-300">{{ tmLabel }}</span>
      </div>
      <button
        @click="openToolManager"
        class="ml-auto text-[10px] text-indigo-400 hover:text-indigo-300 hover:underline"
      >
        Configure
      </button>
    </div>

    <!-- Error messages -->
    <div v-if="data.errors.length > 0" class="mt-2">
      <template v-for="err in data.errors" :key="err.code">
        <div class="text-[10px] text-red-400 bg-red-500/10 px-1.5 py-1 rounded border border-red-500/20">
          {{ err.message }}
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  id: string;
  data: any;
}>();

const computedStyle = computed(() => {
  const hasError = props.data.hasError;
  return {
    background: blockColor(props.data.block?.kind ?? 'utility'),
    color: '#ffffff',
    border: hasError
      ? '2px solid #EF4444'
      : '1px solid rgba(255,255,255,0.2)',
  };
});

const accountSelection = computed(() => props.data.block?.account_selection ?? 'personal-default');

function updateAccountSelection(val: string) {
  // Emit update to store - handled by parent canvas
  const tm = props.data.block?.tool_management;
  if (tm) {
    // The store update is handled via the canvas component
    console.log('Update account selection:', val);
  }
}

function blockColor(kind: string): string {
  const colors: Record<string, string> = {
    trigger: '#10B981',
    llm: '#3B82F6',
    agent: '#8B5CF6',
    tool: '#F59E0B',
    control: '#EF4444',
    output: '#10B981',
    utility: '#6B7280',
    subgraph: '#A855F7',
  };
  return colors[kind] ?? '#6B7280';
}

const tmColor = computed(() => {
  const tm = props.data.block?.tool_management;
  if (!tm) return '#6B7280';
  switch (tm.preset) {
    case 'always-allow': return '#10B981';
    case 'ask-each-time': return '#F59E0B';
    case 'ask-for-writes-deletes': return '#EF4444';
    case 'custom': return '#8B5CF6';
    default: return '#6B7280';
  }
});

const tmLabel = computed(() => {
  const tm = props.data.block?.tool_management;
  if (!tm) return 'Not configured';
  switch (tm.preset) {
    case 'always-allow': return 'Always allow';
    case 'ask-each-time': return 'Ask each time';
    case 'ask-for-writes-deletes': return 'Ask for writes/deletes';
    case 'custom': return 'Custom';
    default: return 'Unknown';
  }
});

function openToolManager() {
  // Emit event to open tool manager dialog
  const evt = new CustomEvent('open-tool-manager', {
    detail: { nodeId: props.data.id, block: props.data.block },
  });
  window.dispatchEvent(evt);
}
</script>

<style scoped>
</style>
