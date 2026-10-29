<template>
  <div class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-semibold">Knowledge Sources</h3>
      <button
        @click="showAddForm = true"
        class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
      >
        + Add Source
      </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-2">
      <div v-if="sources.length === 0" class="flex items-center justify-center py-10">
        <p class="text-slate-500 text-sm">No knowledge sources defined yet.</p>
      </div>

      <div v-for="source in sources" :key="source.id" class="mb-3">
        <div
          class="p-3 rounded bg-slate-800 border border-slate-700 hover:border-slate-600 transition-colors"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-green-500"></span>
              <span class="text-sm font-medium text-slate-200">{{ source.name }}</span>
              <span
                class="text-[10px] text-slate-500 uppercase tracking-tighter"
                >{{ source.provider }}</span>
            </div>
            <button
              @click="removeSource(source.id)"
              class="text-[10px] text-red-400 hover:text-red-300 hover:underline"
            >
              Delete
            </button>
          </div>

          <div class="flex items-center gap-3 mb-2">
            <span class="text-[10px] text-slate-400">Sources:</span>
            <span class="text-[10px] text-slate-300 truncate max-w-[200px]"
              :title="source.source_ids.join(', ')">
              {{ source.source_ids.join(', ') }}
            </span>
          </div>

          <div class="flex items-center gap-3 mb-2">
            <span class="text-[10px] text-slate-400">Citations:</span>
            <span
              :class="source.citations_enabled ? 'text-green-400' : 'text-slate-500'"
            >
              {{ source.citations_enabled ? 'Enabled' : 'Disabled' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Knowledge Source Form -->
    <div v-if="showAddForm" class="mt-4 p-4 rounded bg-slate-800/50 border border-slate-700/50">
      <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
        Add Knowledge Source
      </h4>
      <div class="space-y-3">
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Name</label>
          <input
            v-model="newSource.name"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Company Wiki"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Provider</label>
          <input
            v-model="newSource.provider"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Notion, Google Drive, Slack"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Source IDs</label>
          <input
            v-model="newSource.source_ids_text"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. abc123, def456, ghi789"
          />
        </div>
        <div class="flex items-center gap-2 mb-2">
          <input
            v-model="newSource.citations_enabled"
            type="checkbox"
            class="rounded bg-slate-900 border-slate-600 text-indigo-600 focus:ring-indigo-500"
          />
          <label class="text-xs text-slate-300 cursor-pointer">
            Enable Citations
          </label>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button
            @click="showAddForm = false"
            class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-slate-200"
          >
            Cancel
          </button>
          <button
            @click="saveKnowledgeSource"
            class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { createKnowledgeSource, listKnowledgeSources } from '@/services/ipc';
import type { KnowledgeSource } from '@/types/agent';

const props = defineProps<{
  agentId: string | undefined;
}>();

const sources = ref<KnowledgeSource[]>([]);
const showAddForm = ref(false);
const newSource = ref({
  name: '',
  provider: '',
  source_ids_text: '',
  citations_enabled: false,
});

async function loadSources() {
  if (!props.agentId) return;
  try {
    sources.value = await listKnowledgeSources({ agent_id: props.agentId });
  } catch (e) {
    console.error('Failed to load knowledge sources:', e);
  }
}

async function saveKnowledgeSource() {
  if (!props.agentId || !newSource.value.name.trim()) return;

  try {
    const created = await createKnowledgeSource({
      agent_id: props.agentId,
      name: newSource.value.name,
      provider: newSource.value.provider,
      source_ids: newSource.value.source_ids_text
        .split(',')
        .map(s => s.trim())
        .filter(s => s.length > 0),
      citations_enabled: newSource.value.citations_enabled,
    });
    sources.value.push(created);
    showAddForm.value = false;
    newSource.value = {
      name: '',
      provider: '',
      source_ids_text: '',
      citations_enabled: false,
    };
  } catch (e: any) {
    console.error('Failed to create knowledge source:', e);
  }
}

async function removeSource(id: string) {
  if (!confirm('Delete this knowledge source?')) return;
  try {
    // deleteKnowledgeSource not yet exposed in IPC
    console.log('Delete knowledge source:', id);
  } catch (e: any) {
    console.error('Failed to delete knowledge source:', e);
  }
}

loadSources();
</script>

<style scoped>
</style>
