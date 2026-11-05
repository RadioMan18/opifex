<template>
  <path
    :id="id"
    :d="path"
    class="step-edge"
    :class="[runClass, { 'step-edge--selected': isSelected }]"
    :marker-end="markerEnd"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { RunNodeState } from '@/stores/graphStore';

const props = defineProps<{
  id: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  sourcePosition: string;
  targetPosition: string;
  data?: { selected?: boolean; runState?: RunNodeState };
}>();

// Vue Flow's own edge selection is not authoritative here: the canvas rebuilds
// its edge objects from the store, so selection is carried in `data`.
const isSelected = computed(() => Boolean(props.data?.selected));
const runState = computed(() => props.data?.runState);
const runClass = computed(() => (runState.value ? `step-edge--run-${runState.value}` : null));

// A marker cannot take its fill from the path's stroke, so each state has its
// own arrow and the path points at the one that matches.
const hovered = ref(false);
const markerEnd = computed(() => {
  switch (runState.value) {
    case 'succeeded':
      return 'url(#opifex-arrow-success)';
    case 'failed':
      return 'url(#opifex-arrow-failed)';
    case 'running':
      return 'url(#opifex-arrow-accent)';
  }
  return isSelected.value || hovered.value ? 'url(#opifex-arrow-accent)' : 'url(#opifex-arrow)';
});

const path = computed(() => {
  const { sourceX, sourceY, targetX, targetY } = props;
  const midX = (sourceX + targetX) / 2;
  return `M ${sourceX} ${sourceY} C ${midX} ${sourceY}, ${midX} ${targetY}, ${targetX} ${targetY}`;
});
</script>

<style scoped>
.step-edge {
  fill: none;
  stroke: var(--edge-stroke, #94a3b8);
  stroke-width: 2;
  stroke-linecap: round;
  transition: stroke 0.14s ease, stroke-width 0.14s ease;
}

.step-edge--run-succeeded {
  stroke: #10b981;
}

.step-edge--run-failed {
  stroke: #ef4444;
}

.step-edge--run-running {
  stroke: var(--primary);
  stroke-dasharray: 6 5;
  animation: step-edge-flow 0.7s linear infinite;
}

.step-edge--run-skipped {
  stroke: var(--edge-stroke, #94a3b8);
  opacity: 0.45;
}

@keyframes step-edge-flow {
  to {
    stroke-dashoffset: -11;
  }
}

.step-edge:hover {
  stroke: var(--primary);
  stroke-width: 2.5;
}

.step-edge--selected {
  stroke: var(--primary);
  stroke-width: 2.5;
}
</style>
