<template>
  <path
    :id="id"
    :d="path"
    class="step-edge"
    :class="{ 'step-edge--selected': isSelected }"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  id: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  sourcePosition: string;
  targetPosition: string;
  data?: { selected?: boolean };
}>();

// Vue Flow's own edge selection is not authoritative here: the canvas rebuilds
// its edge objects from the store, so selection is carried in `data`.
const isSelected = computed(() => Boolean(props.data?.selected));

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

.step-edge:hover {
  stroke: var(--primary);
  stroke-width: 2.5;
}

.step-edge--selected {
  stroke: var(--primary);
  stroke-width: 2.5;
}
</style>
