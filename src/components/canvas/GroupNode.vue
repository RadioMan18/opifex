<template>
  <div class="graph-group">
    <span class="graph-group-label">{{ data.label }} · {{ data.memberCount }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  data: { label: string; memberCount: number };
}>();
</script>

<style scoped>
/* Group boxes sit behind the cards (zIndex -1 on the node) and must never
   steal a click intended for the canvas or a card inside them. */
.graph-group {
  width: 100%;
  height: 100%;
  border: 1.5px dashed var(--border-hover, #cbd5e1);
  border-radius: 12px;
  background: rgba(148, 163, 184, 0.07);
  position: relative;
  pointer-events: none;
}

.graph-group-label {
  position: absolute;
  top: -11px;
  left: 14px;
  padding: 1px 8px;
  font-size: 10px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-secondary);
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: 999px;
}
</style>
