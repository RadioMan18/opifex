<template>
  <div class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-semibold">Connectors</h3>
      <button
        @click="showAddForm = true"
        class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
      >
        + Add Connector
      </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-2">
      <div v-if="connectors.length === 0" class="flex items-center justify-center py-10">
        <p class="text-slate-500 text-sm">No connectors defined yet.</p>
      </div>

      <div v-for="connector in connectors" :key="connector.id" class="mb-3">
        <div
          class="p-3 rounded bg-slate-800 border border-slate-700 hover:border-slate-600 transition-colors"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
              <span class="text-sm font-medium text-slate-200">{{ connector.name }}</span>
              <span
                class="text-[10px] text-slate-500 uppercase tracking-tighter"
                >{{ connector.category }}</span>
            </div>
            <button
              @click="removeConnector(connector.id)"
              class="text-[10px] text-red-400 hover:text-red-300 hover:underline"
            >
              Delete
            </button>
          </div>

          <div class="flex items-center gap-3 mb-2">
            <span class="text-[10px] text-slate-400">Account:</span>
            <span
              :class="[
                'text-[10px] font-mono',
                accountColor(connector.account_selection),
              ]"
            >
              {{ accountLabel(connector.account_selection) }}
            </span>
          </div>

          <div class="flex items-center gap-3 mb-2">
            <span class="text-[10px] text-slate-400">Tool Mgmt:</span>
            <span
              :class="[
                'text-[10px] font-mono',
                tmColor(connector.tool_management.preset),
              ]"
            >
              {{ tmLabel(connector.tool_management.preset) }}
            </span>
          </div>

          <div v-if="connector.tool_management.overrides.length > 0" class="mt-2">
            <div class="text-[10px] text-slate-500 mb-1">Tool Overrides:</span>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="ov in connector.tool_management.overrides"
                :key="ov.tool_id"
                class="px-1.5 py-0.5 rounded text-[9px]"
                :class="[
                  ov.mode === 'allow' ? 'bg-green-900/50 text-green-300' : '',
                  ov.mode === 'ask' ? 'bg-yellow-900/50 text-yellow-300' : '',
                  ov.mode === 'deny' ? 'bg-red-900/50 text-red-300' : '',
                ]"
              >
                {{ ov.tool_id }}: {{ ov.mode }}
              </span>
            </div>
          </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Connector Form -->
    <div v-if="showAddForm" class="mt-4 p-4 rounded bg-slate-800/50 border border-slate-700/50">
      <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
        Add Connector
      </h4>
      <div class="space-y-3">
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Name</label>
          <input
            v-model="newConnector.name"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Gmail, Slack, Salesforce"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Category</label>
          <input
            v-model="newConnector.category"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Email, Communication, CRM"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Account Selection</label>
          <select
            v-model="newConnector.account_selection"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
          >
            <option value="personal-default">Personal Default</option>
            <option value="specific-account">Specific Account</option>
            <option value="team-default">Team Default</option>
          </select>
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Tool Management Preset</label>
          <select
            v-model="newConnector.tool_management.preset"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
          >
            <option value="always-allow">Always Allow</option>
            <option value="ask-each-time">Ask Each Time</option>
            <option value="ask-for-writes-deletes">Ask for Writes/Deletes</option>
            <option value="custom">Custom</option>
          </select>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button
            @click="showAddForm = false"
            class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-slate-200"
          >
            Cancel
          </button>
          <button
            @click="saveConnector"
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
import {
  createConnector,
  deleteConnector,
  listConnectors,
} from '@/services/ipc';
import type {
  ConnectorBlock,
  AccountSelection,
  ToolApprovalMode,
  ToolOverride,
} from '@/types/agent';

const connectors = ref<ConnectorBlock[]>([]);
const showAddForm = ref(false);
const newConnector = ref({
  name: '',
  category: '',
  account_selection: 'personal-default' as AccountSelection,
  tool_management: {
    connector_id: '',
    preset: 'ask-for-writes-deletes' as ToolApprovalMode,
    overrides: [] as ToolOverride[],
  },
});

async function loadConnectors() {
  try {
    connectors.value = await listConnectors();
  } catch (e) {
    console.error('Failed to load connectors:', e);
  }
}

async function saveConnector() {
  if (!newConnector.value.name.trim()) return;

  try {
    const created = await createConnector({
      name: newConnector.value.name,
      category: newConnector.value.category,
      account_selection: newConnector.value.account_selection,
      tool_management: newConnector.value.tool_management,
    });
    connectors.value.push(created);
    showAddForm.value = false;
    newConnector.value = {
      name: '',
      category: '',
      account_selection: 'personal-default' as AccountSelection,
      tool_management: {
        connector_id: '',
        preset: 'ask-for-writes-deletes' as ToolApprovalMode,
        overrides: [] as ToolOverride[],
      },
    };
  } catch (e) {
    console.error('Failed to create connector:', e);
  }
}

async function removeConnector(id: string) {
  if (!confirm('Delete this connector?')) return;
  try {
    const success = await deleteConnector(id);
    if (success) {
      connectors.value = connectors.value.filter((c) => c.id !== id);
    }
  } catch (e) {
    console.error('Failed to delete connector:', e);
  }
}

function accountColor(selection: AccountSelection): string {
  switch (selection) {
    case 'personal-default': return 'text-green-400';
    case 'specific-account': return 'text-yellow-400';
    case 'team-default': return 'text-blue-400';
    default: return 'text-slate-400';
  }
}

function accountLabel(selection: AccountSelection): string {
  switch (selection) {
    case 'personal-default': return 'Personal Default';
    case 'specific-account': return 'Specific Account';
    case 'team-default': return 'Team Default';
    default: return 'Unknown';
  }
}

function tmColor(preset: ToolApprovalMode): string {
  switch (preset) {
    case 'always-allow': return 'text-green-400';
    case 'ask-each-time': return 'text-yellow-400';
    case 'ask-for-writes-deletes': return 'text-red-400';
    case 'custom': return 'text-purple-400';
    default: return 'text-slate-400';
  }
}

function tmLabel(preset: ToolApprovalMode): string {
  switch (preset) {
    case 'always-allow': return 'Always allow';
    case 'ask-each-time': return 'Ask each time';
    case 'ask-for-writes-deletes': return 'Ask for writes/deletes';
    case 'custom': return 'Custom';
    default: return 'Unknown';
  }
}

loadConnectors();
</script>

<style scoped>
</style>
