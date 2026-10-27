<script setup lang="ts">
import { PropType } from 'vue';
import type { OllamaModel } from '@/types/agent';

const props = defineProps({
  models: { type: Array as PropType<OllamaModel[]>, required: true },
  defaultModel: { type: String, required: true },
  onChange: { type: Function as PropType<(model: string) => void>, required: true },
  onPull: { type: Function as PropType<(modelName: string) => void>, required: true },
  loading: { type: Boolean, required: true },
});

function handlePull() {
  const modelName = prompt('Enter model name to pull (e.g., llama3.1, mistral, codellama):');
  if (modelName && modelName.trim()) {
    props.onPull(modelName.trim());
  }
}

function formatSize(bytes: number | null): string {
  if (!bytes) return '';
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `(${gb.toFixed(1)} GB)`;
  const mb = bytes / (1024 * 1024);
  return `(${mb.toFixed(1)} MB)`;
}
</script>

<template>
  <div class="model-selector">
    <label class="model-label">Model</label>
    <div class="model-select-wrap">
      <select
        :value="defaultModel"
        class="model-select"
        @change="(e) => props.onChange((e.target as HTMLSelectElement).value)"
        :disabled="loading"
      >
        <option v-for="m of models" :key="m.name" :value="m.name">
          {{ m.name }}
          <span class="model-size" v-if="m.size">{{ formatSize(m.size) }}</span>
        </option>
        <option value="__pull__">Pull new model…</option>
      </select>
      <button
        class="btn-pull"
        @click="handlePull"
        :disabled="loading"
        title="Pull a new model from Ollama"
      >
        Pull
      </button>
    </div>
    <div v-if="loading" class="loading-indicator">Pulling model…</div>
  </div>
</template>

<style scoped>
.model-selector {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.model-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.model-select-wrap {
  position: relative;
  flex: 1;
}

.model-select {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  background: var(--surface-card);
  color: var(--text-primary);
  min-width: 140px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.model-select:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.1);
}

.model-size {
  font-size: var(--type-xs);
  color: var(--text-muted);
  margin-left: var(--space-2);
}

.btn-pull {
  padding: var(--space-2);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s;
}

.btn-pull:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-pull:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-indicator {
  font-size: var(--type-xs);
  color: var(--primary);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>