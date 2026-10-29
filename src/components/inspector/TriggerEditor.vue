<template>
  <div class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-semibold">Triggers</h3>
      <button
        @click="showAddForm = true"
        class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
      >
        + Add Trigger
      </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-2">
      <div v-if="triggers.length === 0" class="flex items-center justify-center py-10">
        <p class="text-slate-500 text-sm">No triggers defined yet.</p>
      </div>

      <div v-for="trigger in triggers" :key="trigger.id" class="mb-3">
        <div
          class="p-3 rounded bg-slate-800 border border-slate-700 hover:border-slate-600 transition-colors"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-orange-500"></span>
              <span class="text-sm font-medium text-slate-200">{{ trigger.kind }}</span>
              <span
                class="text-[10px] text-slate-500 uppercase tracking-tighter"
                >{{ trigger.ai_managed ? 'AI Managed' : 'Manual' }}</span>
            </div>
            <button
              @click="removeTrigger(trigger.id)"
              class="text-[10px] text-red-400 hover:text-red-300 hover:underline"
            >
              Delete
            </button>
          </div>

          <div class="flex items-start gap-3 mb-2">
            <span class="text-[10px] text-slate-400">Config:</span>
            <div class="text-xs text-slate-300 break-all">
              {{ JSON.stringify(trigger.config, null, 2) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Trigger Form -->
    <div v-if="showAddForm" class="mt-4 p-4 rounded bg-slate-800/50 border border-slate-700/50">
      <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
        Add Trigger
      </h4>
      <div class="space-y-3">
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Trigger Kind</label>
          <select
            v-model="newTrigger.kind"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
          >
            <option value="app-trigger">App Trigger</option>
            <option value="webhook">Webhook</option>
            <option value="scheduled">Scheduled</option>
            <option value="one-time">One-Time</option>
            <option value="create-with-ai">Create With AI</option>
          </select>
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Config (JSON)</label>
          <textarea
            v-model="newTrigger.config_text"
            rows="4"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs font-mono text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder='{"key": "value"}'
          ></textarea>
        </div>
        <div class="flex items-center gap-2 mb-2">
          <input
            v-model="newTrigger.ai_managed"
            type="checkbox"
            class="rounded bg-slate-900 border-slate-600 text-indigo-600 focus:ring-indigo-500"
          />
          <label class="text-xs text-slate-300 cursor-pointer">
            AI Managed (agent can create/edit triggers during task)
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
            @click="saveTrigger"
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
import { createTrigger, listTriggers } from '@/services/ipc';
import type { Trigger, TriggerKind } from '@/types/agent';

const props = defineProps<{
  agentId: string | undefined;
}>();

const triggers = ref<Trigger[]>([]);
const showAddForm = ref(false);
const newTrigger = ref({
  kind: 'create-with-ai' as TriggerKind,
  config_text: '{}',
  ai_managed: false,
});

async function loadTriggers() {
  if (!props.agentId) return;
  try {
    triggers.value = (await listTriggers({ agent_id: props.agentId })) as Trigger[];
  } catch (e) {
    console.error('Failed to load triggers:', e);
  }
}

async function saveTrigger() {
  if (!props.agentId) return;

  try {
    const config = JSON.parse(newTrigger.value.config_text || '{}');
    const created = await createTrigger({
      agent_id: props.agentId,
      kind: newTrigger.value.kind,
      config,
      ai_managed: newTrigger.value.ai_managed,
    });
    triggers.value.push(created as Trigger);
    showAddForm.value = false;
    newTrigger.value = {
      kind: 'create-with-ai' as TriggerKind,
      config_text: '{}',
      ai_managed: false,
    };
  } catch (e: any) {
    console.error('Failed to create trigger:', e);
  }
}

async function removeTrigger(id: string) {
  if (!confirm('Delete this trigger?')) return;
  try {
    console.log('Delete trigger:', id);
  } catch (e: any) {
    console.error('Failed to delete trigger:', e);
  }
}

loadTriggers();
</script>

<style scoped>
</style>
