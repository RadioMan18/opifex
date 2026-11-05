<template>
  <!-- Edges are node-to-node, so one anchor per side. They live on the shell so
       they straddle the card border instead of being clipped by its overflow. -->
  <div class="node-shell">
    <Handle
      v-if="visibleInputs.length"
      id="in"
      type="target"
      :position="Position.Left"
      class="node-handle node-handle--in"
    />
    <Handle
      v-if="visibleOutputs.length"
      id="out"
      type="source"
      :position="Position.Right"
      class="node-handle node-handle--out"
    />

    <div class="node-card" :class="{ 'node-card--error': data.hasError }">
      <span class="node-accent" :style="{ background: accentColor }" aria-hidden="true"></span>

      <header class="node-header">
        <span class="node-kind-dot" :style="{ background: accentColor }" aria-hidden="true"></span>
        <h4 class="node-title" :title="data.label">{{ data.label }}</h4>
        <span
          v-if="data.hasError"
          class="node-error-dot"
          title="Validation error"
          aria-label="Validation error"
        ></span>
      </header>

      <p class="node-subtitle">{{ kindLabel }}</p>

      <div v-if="visibleInputs.length || visibleOutputs.length" class="node-ports">
        <div class="node-port-column">
          <div v-for="input in visibleInputs" :key="input.id" class="node-port node-port--in">
            <span class="node-port-name" :title="input.name">{{ input.name }}</span>
          </div>
        </div>

        <div class="node-port-column node-port-column--out">
          <div v-for="output in visibleOutputs" :key="output.id" class="node-port node-port--out">
            <span class="node-port-name" :title="output.name">{{ output.name }}</span>
          </div>
        </div>
      </div>

      <p v-if="overflowCount > 0" class="node-ports-more">+{{ overflowCount }} more</p>

      <div v-if="parameterChips.length" class="node-params">
        <span
          v-for="chip in parameterChips"
          :key="chip.key"
          class="node-param"
          :title="`${chip.key}=${chip.value}`"
        >
          {{ chip.key }}={{ chip.value }}
        </span>
      </div>

      <ul v-if="data.errors?.length" class="node-errors">
        <li v-for="err in data.errors" :key="err.code">{{ err.message }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import type { BlockKind } from '@/types/agent';

const props = defineProps<{
  id: string;
  data: any;
}>();

const KIND_COLORS: Record<BlockKind, string> = {
  trigger: '#10B981',
  llm: '#3B82F6',
  agent: '#8B5CF6',
  tool: '#F59E0B',
  control: '#EF4444',
  output: '#10B981',
  utility: '#6B7280',
  subgraph: '#A855F7',
};

// Kind is an accent, not the surface: a saturated fill makes a graph of many
// nodes hard to scan, and colour is better spent on state (error, selection).
const accentColor = computed(
  () => KIND_COLORS[(props.data.block?.kind as BlockKind) ?? 'utility'] ?? '#6B7280'
);

const kindLabel = computed(() => props.data.block?.kind ?? 'utility');

const MAX_PORTS = 2;

const visibleInputs = computed(() => (props.data.block?.inputs ?? []).slice(0, MAX_PORTS));
const visibleOutputs = computed(() => (props.data.block?.outputs ?? []).slice(0, MAX_PORTS));

const overflowCount = computed(() => {
  const inputs = props.data.block?.inputs?.length ?? 0;
  const outputs = props.data.block?.outputs?.length ?? 0;
  return Math.max(0, inputs - MAX_PORTS) + Math.max(0, outputs - MAX_PORTS);
});

const parameterChips = computed(() =>
  Object.entries(props.data.parameters ?? {})
    .slice(0, 3)
    .map(([key, value]) => ({ key, value: String(value).slice(0, 12) }))
);
</script>

<style src="./nodeCard.css"></style>
