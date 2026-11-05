<template>
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

    <div class="node-card node-card--wide" :class="{ 'node-card--error': data.hasError }">
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

      <p class="node-subtitle">{{ data.block?.category ?? 'Connector' }}</p>

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

      <label class="node-field">
        <span class="node-field-label">Account</span>
        <select
          v-model="accountSelection"
          class="node-select"
          @change="updateAccountSelection(accountSelection)"
        >
          <option value="personal-default">Personal default</option>
          <option value="specific-account">Specific account</option>
          <option value="team-default">Team default</option>
        </select>
      </label>

      <div class="node-section">
        <div class="node-section-head">
          <span class="node-section-title">Tool management</span>
          <span class="node-status">
            <span class="node-status-dot" :style="{ backgroundColor: tmColor }" aria-hidden="true"></span>
            {{ tmLabel }}
          </span>
        </div>
        <button type="button" class="node-link-btn" @click="openToolManager">Configure</button>
      </div>

      <ul v-if="data.errors?.length" class="node-errors">
        <li v-for="err in data.errors" :key="err.code">{{ err.message }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
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

const accentColor = computed(
  () => KIND_COLORS[(props.data.block?.kind as BlockKind) ?? 'utility'] ?? '#6B7280'
);

const MAX_PORTS = 2;

const visibleInputs = computed(() => (props.data.block?.inputs ?? []).slice(0, MAX_PORTS));
const visibleOutputs = computed(() => (props.data.block?.outputs ?? []).slice(0, MAX_PORTS));

const overflowCount = computed(() => {
  const inputs = props.data.block?.inputs?.length ?? 0;
  const outputs = props.data.block?.outputs?.length ?? 0;
  return Math.max(0, inputs - MAX_PORTS) + Math.max(0, outputs - MAX_PORTS);
});

// account_selection is a capability flag on the block definition, so only treat
// it as a stored choice when it actually holds one.
const accountSelection = ref(
  typeof props.data.block?.account_selection === 'string'
    ? props.data.block.account_selection
    : 'personal-default'
);

function updateAccountSelection(val: string) {
  const tm = props.data.block?.tool_management;
  if (tm) {
    // The store update is handled via the canvas component
    console.log('Update account selection:', val);
  }
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
.node-field {
  display: flex;
  flex-direction: column;
  gap: 3px;
  margin-top: 2px;
}

.node-field-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.node-select {
  width: 100%;
  padding: 3px 6px;
  font-family: inherit;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--surface-base);
  border: 1px solid var(--border);
  border-radius: 4px;
}

.node-select:focus {
  outline: none;
  border-color: var(--border-focus, var(--primary));
}

.node-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 8px;
  background: var(--surface-hover);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.node-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.node-section-title {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.node-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--text-secondary);
}

.node-status-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
}

.node-link-btn {
  align-self: flex-start;
  padding: 0;
  font-family: inherit;
  font-size: 10px;
  color: var(--primary);
  background: none;
  border: none;
  cursor: pointer;
}

.node-link-btn:hover {
  text-decoration: underline;
}
</style>
