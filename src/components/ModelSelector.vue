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
        <option value="__pull__">+ Download new model...</option>
      </select>
      <button
        class="btn-pull"
        @click="handlePull"
        :disabled="loading"
        title="Pull a new model from Ollama"
      >
        ⬇
      </button>
    </div>
    <div v-if="loading" class="loading-indicator">Pulling model...</div>
  </div>
</template>

<style scoped>
.model-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.model-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.model-select-wrap {
  position: relative;
  flex: 1;
}

.model-select {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid #d1d1d1;
  border-radius: 6px;
  font-size: 0.9rem;
  background: #ffffff;
}

.model-select:focus {
  outline: none;
  border-color: #0066cc;
}

.model-size {
  font-size: 0.7rem;
  color: #9ca3af;
  margin-left: 0.5rem;
}

.btn-pull {
  padding: 0.5rem;
  background: #e2e8f0;
  color: #1a1a1a;
  border: 1px solid #6b7280;
  border-radius: 6px;
  cursor: pointer;
}

.btn-pull:hover:not(:disabled) {
  background: #e5e7eb;
}

.btn-pull:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-indicator {
  font-size: 0.75rem;
  color: #0066cc;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
