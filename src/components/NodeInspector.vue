<template>
  <div
    class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col"
  >
    <div class="flex items-center justify-between mb-2">
      <h3 class="text-sm font-semibold">Node Inspector</h3>
      <span
        v-if="node"
        class="text-xs text-slate-400 truncate max-w-[200px]"
        :title="nodeId"
      >
        {{ nodeId ?? '' }}
      </span>
    </div>

    <div v-if="!node" class="flex-1 flex items-center justify-center">
      <p class="text-slate-500 text-sm">Select a node to configure</p>
    </div>

    <div v-else class="flex-1 overflow-y-auto space-y-4">
      <div class="p-3 rounded bg-slate-800/50 border border-slate-700/50">
        <div class="flex items-center gap-2 mb-2">
          <span
            class="w-2 h-2 rounded-full"
            :style="{ backgroundColor: blockColor(node.block.kind ?? 'utility') }"
          ></span>
          <span class="text-xs font-medium text-slate-200">
            {{ node.block.name }}
          </span>
          <span class="text-[10px] text-slate-500 uppercase tracking-tighter">
            {{ node.block.kind }}
          </span>
        </div>
        <p
          v-if="node.block.description"
          class="text-[10px] text-slate-400 mb-2"
        >
          {{ node.block.description }}
        </p>
      </div>

      <!-- Input mapping (Gumloop: mapping happens inside the node, not on edges) -->
      <div v-if="node.block.inputs.length > 0">
        <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
          Input Mapping
        </h4>
        <div class="space-y-2">
          <template v-for="input in node.block.inputs" :key="input.id">
            <div
              class="p-2 rounded border bg-slate-800/50 border-slate-700/50"
            >
              <div class="flex items-center justify-between mb-1">
                <label
                  class="text-xs font-medium text-slate-300"
                  :for="`input-map-${input.id}`"
                >
                  {{ input.name }}
                </label>
                <span
                  v-if="input.required"
                  class="text-[10px] text-red-400 font-bold"
                >
                  required
                </span>
              </div>
              <select
                :id="`input-map-${input.id}`"
                v-model="inputMappings[input.id]"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                @change="
                  updateInputMapping(input.id, (($event.target as HTMLSelectElement)!).value)
                "
              >
                <option value="">None</option>
                <template v-for="upstream in upstreamNodes" :key="upstream.node.id">
                  <template v-for="out in upstream.block.outputs" :key="out.id">
                    <option :value="`${upstream.node.id}:${out.id}`">
                      {{ upstream.block.name }} → {{ out.name }}
                    </option>
                  </template>
                </template>
              </select>
              <p v-if="input.description" class="text-[10px] text-slate-500 mt-1">
                {{ input.description }}
              </p>
            </div>
          </template>
        </div>
      </div>

      <div v-if="node.block.parameters.length > 0">
        <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
          Parameters
        </h4>
        <div class="space-y-3">
          <template v-for="param in node.block.parameters" :key="param.id">
            <div
              :class="[
                'p-2.5 rounded border',
                param.sensitive
                  ? 'bg-slate-800/30 border-slate-700'
                  : 'bg-slate-800/50 border-slate-700/50',
              ]"
            >
              <div class="flex items-center justify-between mb-1">
                <label
                  class="text-xs font-medium text-slate-300"
                  :for="`param-${param.id}`"
                >
                  {{ param.label }}
                </label>
                <span
                  v-if="param.required"
                  class="text-[10px] text-red-400 font-bold"
                >
                  required
                </span>
              </div>

              <input
                v-if="param.type === 'string'"
                :id="`param-${param.id}`"
                :value="workingValues[param.id] ?? ''"
                type="text"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                :placeholder="String(param.defaultValue ?? '')"
                @input="updateParameter(param.id, (($event.target as HTMLInputElement)!).value)"
              />

              <textarea
                v-else-if="param.type === 'textarea'"
                :id="`param-${param.id}`"
                :value="String(workingValues[param.id] ?? '')"
                rows="3"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                :placeholder="String(param.defaultValue ?? '')"
                @input="updateParameter(param.id, (($event.target as HTMLTextAreaElement)!).value)"
              ></textarea>

              <input
                v-else-if="param.type === 'number'"
                :id="`param-${param.id}`"
                :value="workingValues[param.id] ?? ''"
                type="number"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                :placeholder="String(param.defaultValue ?? '')"
                @input="updateParameter(param.id, Number((($event.target as HTMLInputElement)!).value))"
              />

              <label
                v-else-if="param.type === 'boolean'"
                class="flex items-center gap-2 cursor-pointer"
              >
                <input
                  :id="`param-${param.id}`"
                  :checked="Boolean(workingValues[param.id])"
                  type="checkbox"
                  class="rounded bg-slate-900 border-slate-600 text-indigo-600 focus:ring-indigo-500"
                  @change="updateParameter(param.id, (($event.target as HTMLInputElement)!).checked)"
                />
                <span class="text-xs text-slate-300">
                  {{ workingValues[param.id] ? 'Enabled' : 'Disabled' }}
                </span>
              </label>

              <select
                v-else-if="param.type === 'select'"
                :id="`param-${param.id}`"
                :value="workingValues[param.id] ?? ''"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                @change="updateParameter(param.id, (($event.target as HTMLSelectElement)!).value)"
              >
                <option
                  v-for="opt in param.options ?? []"
                  :key="opt"
                  :value="opt"
                >
                  {{ opt }}
                </option>
              </select>

              <div
                v-else-if="param.type === 'multi-select'"
                class="flex flex-wrap gap-1"
              >
                <button
                  v-for="opt in param.options ?? []"
                  :key="opt"
                  :class="[
                    'px-2 py-1 text-xs rounded border',
                    isSelected(param.id, opt)
                      ? 'bg-indigo-600 border-indigo-500 text-white'
                      : 'bg-slate-800 border-slate-700 text-slate-300',
                  ]"
                  @click="toggleOption(param.id, opt)"
                >
                  {{ opt }}
                </button>
              </div>

              <textarea
                v-else-if="param.type === 'code'"
                :id="`param-${param.id}`"
                :value="String(workingValues[param.id] ?? '')"
                rows="6"
                class="w-full bg-black border border-slate-600 rounded px-2 py-1.5 text-xs font-mono text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                :placeholder="String(param.defaultValue ?? '')"
                spellcheck="false"
                @input="updateParameter(param.id, (($event.target as HTMLTextAreaElement)!).value)"
              ></textarea>

              <input
                v-else-if="param.type === 'credential'"
                :id="`param-${param.id}`"
                :value="String(workingValues[param.id] ?? '')"
                type="password"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                placeholder="••••••••"
                @input="updateParameter(param.id, (($event.target as HTMLInputElement)!).value)"
              />

              <input
                v-else
                :id="`param-${param.id}`"
                :value="workingValues[param.id] ?? ''"
                type="text"
                class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none transition-colors"
                :placeholder="String(param.defaultValue ?? '')"
                @input="updateParameter(param.id, (($event.target as HTMLInputElement)!).value)"
              />
            </div>
            <p v-if="param.description" class="text-[10px] text-slate-500 mt-1">
              {{ param.description }}
            </p>
          </template>
        </div>
      </div>

      <div v-else class="p-3 rounded bg-slate-800/20 border border-slate-800">
        <p class="text-[10px] text-slate-500 text-center">
          No configurable parameters
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useGraphStore } from '@/stores/graphStore';
import type { BlockDefinition } from '@/types/agent';

const store = useGraphStore();

const props = defineProps<{
  nodeId: string | undefined;
}>();

const graph = computed(() => store.graph);
const blocks = computed(() => store.blocks);

const node = computed(() => {
  if (!props.nodeId || !graph.value) return null;
  const n = graph.value.nodes.find((nd) => nd.id === props.nodeId);
  if (!n) return null;
  const block = blocks.value.find((b) => b.id === n.blockId);
  if (!block) return null;
  return { ...n, block } as { id: string; blockId: string; blockVersion: number; position: { x: number; y: number }; parameters: Record<string, unknown>; subGraph?: any; isCollapsed: boolean; block: BlockDefinition };
});

// Input mapping: input port id -> "sourceNodeId:sourcePortId"
const inputMappings = ref<Record<string, string>>({});

// Working values for parameter editing
const workingValues = ref<Record<string, unknown>>({});

// Upstream nodes with their blocks (nodes that have edges pointing TO this node)
const upstreamNodes = computed(() => {
  if (!props.nodeId || !graph.value) return [];
  const g = graph.value;
  return g.nodes
    .filter((n) => g.edges.some((e) => e.target === props.nodeId && e.source === n.id))
    .map((n) => {
      const block = blocks.value.find((b) => b.id === n.blockId);
      return { node: n, block };
    })
    .filter((u): u is { node: any; block: BlockDefinition } => u.block !== undefined);
});

watch(
  node,
  (newNode) => {
    if (newNode) {
      workingValues.value = { ...newNode.parameters };
      // Initialize input mappings from node parameters if stored there
      const existing = (newNode.parameters as any)?._inputMappings ?? {};
      inputMappings.value = { ...existing };
    } else {
      workingValues.value = {};
      inputMappings.value = {};
    }
  }
);

function updateParameter(paramId: string, value: unknown) {
  workingValues.value[paramId] = value;
  const n = node.value;
  if (n) {
    store.updateNodeParameters(n.id, { [paramId]: value });
  }
}

function updateInputMapping(inputId: string, value: string) {
  inputMappings.value[inputId] = value || '';
  const n = node.value;
  if (n) {
    const params = { ...n.parameters, _inputMappings: inputMappings.value };
    store.updateNodeParameters(n.id, params);
  }
}

function isSelected(paramId: string, opt: string): boolean {
  const val = workingValues.value[paramId];
  return Array.isArray(val) ? val.includes(opt) : val === opt;
}

function toggleOption(paramId: string, opt: string) {
  const val = workingValues.value[paramId];
  let newVal: unknown;
  if (Array.isArray(val)) {
    if (val.includes(opt)) {
      newVal = val.filter((o: string) => o !== opt);
    } else {
      newVal = [...val, opt];
    }
  } else {
    newVal = opt;
  }
  updateParameter(paramId, newVal);
}

function blockColor(kind: string): string {
  const colors: Record<string, string> = {
    trigger: '#10B981',
    llm: '#3B82F6',
    agent: '#8B5CF6',
    tool: '#F59E0B',
    control: '#EF4444',
    output: '#10B981',
    utility: '#6B7280',
    subgraph: '#A855F7',
  };
  return colors[kind] ?? '#6B7280';
}
</script>

<style scoped>
</style>
