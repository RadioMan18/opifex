<template>
  <path
    :id="id"
    :d="path"
    stroke="#6B7280"
    stroke-width="2"
    fill="none"
    stroke-linecap="round"
    class="transition-all duration-200"
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
}>();

const path = computed(() => {
  const { sourceX, sourceY, targetX, targetY } = props;
  const midX = (sourceX + targetX) / 2;
  return `M ${sourceX} ${sourceY} C ${midX} ${sourceY}, ${midX} ${targetY}, ${targetX} ${targetY}`;
});
</script>
