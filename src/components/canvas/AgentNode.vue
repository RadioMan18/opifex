<template>
  <div
    class="w-64 p-3 rounded-lg border shadow-md flex flex-col gap-2 group"
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

    <div class="flex items-start gap-3 mb-2">
      <div class="flex flex-col gap-1.5 flex-1">
        <template
          v-for="input in data.block?.inputs || []"
          :key="input.id"
        >
          <div class="flex items-center gap-1.5">
            <div class="w-2 h-2 rounded-full bg-blue-400" :title="input.name"></div>
            <span class="text-[10px] text-slate-400 truncate max-w-[120px]" :title="input.name">
              {{ input.name }}
            </span>
          </div>
        </template>
      </div>
    </div>

    <div
      v-if="Object.keys(data.parameters || {}).length > 0"
      class="mb-2"
    >
      <div class="text-[10px] text-slate-500 mb-1">Parameters</div>
      <div class="flex flex-wrap gap-1">
        <span
          v-for="(value, key) in Object.entries(data.parameters || {})"
          :key="key"
          class="px-1.5 py-0.5 rounded bg-slate-800/50 border border-slate-700/50 text-[10px] text-slate-300"
        >
          {{ key }}={{ String(value).slice(0, 12) }}
        </span>
      </div>
    </div>

    <div class="flex items-end gap-3 mt-2">
      <div class="flex flex-col gap-1.5 flex-1">
        <template
          v-for="output in data.block?.outputs || []"
          :key="output.id"
        >
          <div class="flex items-center gap-1.5 justify-end">
            <span class="text-[10px] text-slate-400 truncate max-w-[120px]" :title="output.name">
              {{ output.name }}
            </span>
            <div class="w-2 h-2 rounded-full bg-green-400"></div>
          </div>
        </template>
      </div>
    </div>

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
</script>

<style scoped>
</style>
