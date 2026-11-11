<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from 'vue';
import { PropType } from 'vue';
import type {
  AgentChain,
  AgentPlan,
  BindingSource,
  ChainGate,
  FailurePolicy,
  GateMode,
  InputBinding,
  PlanSummary,
  PlanVersion,
  TriggerKind,
  TriggerPort,
} from '@/types/agent';
import {
  migratePlan,
  planIssues,
  planSummary,
  renameResultPort,
  renameTriggerPort,
} from '@/lib/planSchema';

const props = defineProps({
  plan: { type: Object as PropType<AgentPlan | null>, required: true, default: null },
  agentName: { type: String, required: true },
  versions: { type: Array as PropType<any[]>, required: true, default: () => [] },
  compareVersionId: { type: String as PropType<string | null>, required: false, default: null },
  compareVersion: { type: Object as PropType<PlanVersion | null>, required: false, default: null },
});

const emit = defineEmits<{
  update: [plan: AgentPlan];
  'save-version': [];
  'restore-version': [versionId: string];
  'compare-version': [versionId: string | null];
  'export-agent': [];
}>();

const plan = computed(() => props.plan);
const compareVersion = computed(() => props.compareVersion);
const showVersions = ref(false);

const triggers = computed<TriggerPort[]>(() => plan.value?.triggers ?? []);
const results = computed(() => plan.value?.results ?? []);
const chains = computed<AgentChain[]>(() => plan.value?.chains ?? []);
const goalText = computed(() => plan.value?.goal ?? '');
const toolsText = computed(() => (plan.value?.tools ?? []).join(', '));
const knowledgeText = computed(() => (plan.value?.knowledgeSources ?? []).join(', '));

const issues = computed(() => {
  const current = plan.value;
  if (!current || current.chains.length === 0) return [];
  return planIssues(current);
});

const view = ref<'cards' | 'graph'>('cards');
const paneRoot = ref<HTMLElement | null>(null);
const focusedChainId = ref<string | null>(null);
let focusTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * Jumps from a graph node to the editable card for that chain: switch back to
 * cards, scroll the chain into view, then focus its action field.
 */
async function focusChain(index: number) {
  const chain = chains.value[index];
  if (!chain) return;

  view.value = 'cards';
  await nextTick();

  const card = paneRoot.value?.querySelector<HTMLElement>(`[data-chain-id="${chain.id}"]`);
  if (!card) return;

  card.scrollIntoView({ block: 'center', behavior: 'smooth' });
  focusedChainId.value = chain.id;
  if (focusTimer) clearTimeout(focusTimer);
  focusTimer = setTimeout(() => {
    focusedChainId.value = null;
  }, 1600);

  const field = card.querySelector<HTMLElement>('.chain-capability')
    ?? card.querySelector<HTMLElement>('select, input, textarea');
  field?.focus({ preventScroll: true });
}

onBeforeUnmount(() => {
  if (focusTimer) clearTimeout(focusTimer);
});

// Graph layout: one lane per chain, fixed columns for trigger / action / result.
const GRAPH_WIDTH = 684;
const NODE_H = 34;
const ROW_H = 62;
const GRAPH_TOP = 10;
const X_TRIGGER = 12;
const W_TRIGGER = 148;
const X_ACTION = 200;
const W_ACTION = 132;
const X_RESULT = 372;
const W_RESULT = 150;
const X_GATE = 562;
const W_GATE = 110;

function clipLabel(value: string, max = 21): string {
  return value.length > max ? `${value.slice(0, max - 1)}…` : value;
}

interface GraphRow {
  chain: AgentChain;
  y: number;
  triggerLabel: string;
  triggerClass: string;
  actionLabel: string;
  resultLabel: string;
  resultTerminal: boolean;
  gateLabel: string;
  gated: boolean;
  title: string;
}

const graphRows = computed<GraphRow[]>(() =>
  chains.value.map((chain, index) => {
    const handoff = results.value.some(port => port.name === chain.trigger);
    const kind = triggers.value.find(port => port.name === chain.trigger)?.kind;
    return {
      chain,
      y: GRAPH_TOP + index * ROW_H,
      triggerLabel: clipLabel(chain.trigger ? `${handoff ? '↳ ' : ''}${chain.trigger}` : 'unbound'),
      triggerClass: handoff ? 'is-handoff' : `kind-${kind ?? 'none'}`,
      actionLabel: clipLabel(chain.capability || 'unbound'),
      resultLabel: clipLabel(chain.result || 'unbound'),
      resultTerminal: isTerminal(chain.result),
      gateLabel: clipLabel(`${chain.gate.mode} · ${chain.gate.onFailure}`, 16),
      gated: chain.gate.mode === 'approve',
      title: `${chain.trigger || 'unbound'} → ${chain.capability || 'unbound'} → ${chain.result || 'unbound'}`,
    };
  }),
);

function nodeCenterY(index: number): number {
  return GRAPH_TOP + index * ROW_H + NODE_H / 2;
}

const graphEdges = computed(() => {
  const producer = new Map<string, number>();
  chains.value.forEach((chain, index) => {
    if (chain.result && !producer.has(chain.result)) producer.set(chain.result, index);
  });

  const edges: Array<{ id: string; kind: 'handoff' | 'input'; path: string }> = [];
  const resultRight = X_RESULT + W_RESULT;
  const actionRight = X_ACTION + W_ACTION;

  chains.value.forEach((chain, index) => {
    const targetY = nodeCenterY(index);

    const upstream = producer.get(chain.trigger);
    if (upstream !== undefined && upstream !== index) {
      const sourceY = nodeCenterY(upstream);
      edges.push({
        id: `handoff-${index}`,
        kind: 'handoff',
        path: `M ${resultRight} ${sourceY} C ${resultRight + 46} ${sourceY}, ${X_TRIGGER - 46} ${targetY}, ${X_TRIGGER} ${targetY}`,
      });
    }

    chain.inputs.forEach((binding, bindingIndex) => {
      if (binding.from !== 'result' || !binding.ref) return;
      const sourceIndex = producer.get(binding.ref);
      if (sourceIndex === undefined || sourceIndex === index) return;
      const sourceY = nodeCenterY(sourceIndex);
      edges.push({
        id: `input-${index}-${bindingIndex}`,
        kind: 'input',
        path: `M ${resultRight} ${sourceY} C ${resultRight + 40} ${sourceY}, ${actionRight + 40} ${targetY}, ${actionRight} ${targetY}`,
      });
    });
  });

  return edges;
});

const graphHeight = computed(
  () => GRAPH_TOP * 2 + Math.max(chains.value.length - 1, 0) * ROW_H + NODE_H,
);

// Roving tabindex: exactly one lane is tabbable, arrows move between lanes.
const focusedLane = ref(0);
const activeLane = computed(() =>
  Math.min(focusedLane.value, Math.max(graphRows.value.length - 1, 0)),
);

const gateChain = computed<AgentChain | null>(() => chains.value[activeLane.value] ?? null);

/** Chain picker in the gate strip: selects a lane to edit without leaving the graph. */
function selectLane(event: Event) {
  const index = Number.parseInt((event.target as HTMLSelectElement).value, 10);
  if (Number.isFinite(index)) focusedLane.value = index;
}

function laneElement(index: number): SVGElement | null {
  return paneRoot.value?.querySelector<SVGElement>(`[data-lane-index="${index}"]`) ?? null;
}

async function moveLane(index: number, delta: number) {
  const next = index + delta;
  if (next < 0 || next >= graphRows.value.length) return;
  focusedLane.value = next;
  await nextTick();
  laneElement(next)?.focus();
}

const currentSummary = computed<PlanSummary | null>(() => (plan.value ? planSummary(plan.value) : null));
const previousSummary = computed<PlanSummary | null>(() => {
  const spec = compareVersion.value?.specJson;
  return spec ? planSummary(migratePlan(spec)) : null;
});

const diffFields = ['goal', 'steps', 'triggers', 'outputs', 'tools', 'knowledgeSources'] as const;
const diffLabels: Record<(typeof diffFields)[number], string> = {
  goal: 'Goal',
  steps: 'Actions',
  triggers: 'Triggers',
  outputs: 'Deliverables',
  tools: 'Tools',
  knowledgeSources: 'Knowledge Sources',
};
const diffRows = computed(() =>
  diffFields.map(field => ({
    field,
    label: diffLabels[field],
    current: currentSummary.value?.[field] || '—',
    previous: previousSummary.value?.[field] || '—',
  })),
);

function commit(next: AgentPlan) {
  emit('update', next);
}

function splitList(value: string): string[] {
  return value.split(',').map(item => item.trim()).filter(Boolean);
}

function uniqueName(existing: string[], base: string): string {
  if (!existing.includes(base)) return base;
  let suffix = 2;
  while (existing.includes(`${base}_${suffix}`)) suffix += 1;
  return `${base}_${suffix}`;
}

function isTerminal(name: string): boolean {
  return results.value.find(port => port.name === name)?.terminal ?? false;
}

function updateChain(index: number, patch: Partial<AgentChain>) {
  const current = plan.value;
  if (!current) return;
  commit({ ...current, chains: current.chains.map((chain, i) => (i === index ? { ...chain, ...patch } : chain)) });
}

function updateGate(index: number, patch: Partial<ChainGate>) {
  const chain = plan.value?.chains[index];
  if (!chain) return;
  const gate: ChainGate = { ...chain.gate, ...patch };
  if (gate.mode === 'auto' || !gate.approver) delete gate.approver;
  if (gate.onFailure !== 'retry') delete gate.maxRetries;
  updateChain(index, { gate });
}

function setGoal(event: Event) {
  const current = plan.value;
  if (!current) return;
  commit({ ...current, goal: (event.target as HTMLTextAreaElement).value });
}

function setTools(event: Event) {
  const current = plan.value;
  if (!current) return;
  commit({ ...current, tools: splitList((event.target as HTMLInputElement).value) });
}

function setKnowledge(event: Event) {
  const current = plan.value;
  if (!current) return;
  commit({ ...current, knowledgeSources: splitList((event.target as HTMLInputElement).value) });
}

function addTrigger() {
  const current = plan.value;
  if (!current) return;
  const name = uniqueName([...triggers.value.map(port => port.name), ...results.value.map(port => port.name)], 'new_trigger');
  commit({ ...current, triggers: [...current.triggers, { name, kind: 'event' }] });
}

function renameTrigger(index: number, event: Event) {
  const current = plan.value;
  const port = current?.triggers[index];
  const input = event.target as HTMLInputElement;
  if (!current || !port) return;
  const name = input.value.trim();
  const taken = current.triggers.some(other => other.name === name) || current.results.some(other => other.name === name);
  if (!name || taken) {
    input.value = port.name;
    return;
  }
  commit(renameTriggerPort(current, port.name, name));
}

function setTriggerKind(index: number, event: Event) {
  const current = plan.value;
  if (!current) return;
  const kind = (event.target as HTMLSelectElement).value as TriggerKind;
  commit({ ...current, triggers: current.triggers.map((port, i) => (i === index ? { ...port, kind } : port)) });
}

function setTriggerSchedule(index: number, event: Event) {
  const current = plan.value;
  if (!current) return;
  const schedule = (event.target as HTMLInputElement).value.trim() || undefined;
  commit({ ...current, triggers: current.triggers.map((port, i) => (i === index ? { ...port, schedule } : port)) });
}

function removeTrigger(index: number) {
  const current = plan.value;
  const port = current?.triggers[index];
  if (!current || !port) return;
  commit({
    ...current,
    triggers: current.triggers.filter((_, i) => i !== index),
    chains: current.chains.map(chain => ({
      ...chain,
      trigger: chain.trigger === port.name ? '' : chain.trigger,
      inputs: chain.inputs.map(binding =>
        binding.from === 'trigger' && binding.ref === port.name ? { ...binding, ref: undefined } : binding,
      ),
    })),
  });
}

function addResult() {
  const current = plan.value;
  if (!current) return;
  const name = uniqueName([...results.value.map(port => port.name), ...triggers.value.map(port => port.name)], 'new_result');
  commit({ ...current, results: [...current.results, { name, terminal: true }] });
}

function renameResult(index: number, event: Event) {
  const current = plan.value;
  const port = current?.results[index];
  const input = event.target as HTMLInputElement;
  if (!current || !port) return;
  const name = input.value.trim();
  const taken = current.results.some(other => other.name === name) || current.triggers.some(other => other.name === name);
  if (!name || taken) {
    input.value = port.name;
    return;
  }
  commit(renameResultPort(current, port.name, name));
}

function setResultTerminal(index: number, event: Event) {
  const current = plan.value;
  if (!current) return;
  const terminal = (event.target as HTMLInputElement).checked;
  commit({ ...current, results: current.results.map((port, i) => (i === index ? { ...port, terminal } : port)) });
}

function setResultSchema(index: number, event: Event) {
  const current = plan.value;
  if (!current) return;
  const schema = (event.target as HTMLInputElement).value.trim() || undefined;
  commit({ ...current, results: current.results.map((port, i) => (i === index ? { ...port, schema } : port)) });
}

function removeResult(index: number) {
  const current = plan.value;
  const port = current?.results[index];
  if (!current || !port) return;
  commit({
    ...current,
    results: current.results.filter((_, i) => i !== index),
    chains: current.chains.map(chain => ({
      ...chain,
      trigger: chain.trigger === port.name ? '' : chain.trigger,
      result: chain.result === port.name ? '' : chain.result,
      inputs: chain.inputs.map(binding =>
        binding.from === 'result' && binding.ref === port.name ? { ...binding, ref: undefined } : binding,
      ),
    })),
  });
}

function addChain() {
  const current = plan.value;
  if (!current) return;
  const resultName = uniqueName([...results.value.map(port => port.name), ...triggers.value.map(port => port.name)], 'new_result');
  const chain: AgentChain = {
    id: uniqueName(current.chains.map(existing => existing.id), 'chain'),
    trigger: current.triggers[0]?.name ?? '',
    capability: '',
    inputs: [],
    result: resultName,
    gate: { mode: 'auto', onFailure: 'stop' },
  };
  commit({
    ...current,
    results: [...current.results, { name: resultName, terminal: true }],
    chains: [...current.chains, chain],
  });
}

function removeChain(index: number) {
  const current = plan.value;
  if (!current) return;
  commit({ ...current, chains: current.chains.filter((_, i) => i !== index) });
}

function setChainTrigger(index: number, event: Event) {
  updateChain(index, { trigger: (event.target as HTMLSelectElement).value });
}

function setChainCapability(index: number, event: Event) {
  updateChain(index, { capability: (event.target as HTMLInputElement).value.trim() });
}

function setChainResult(index: number, event: Event) {
  updateChain(index, { result: (event.target as HTMLSelectElement).value });
}

function setDecision(index: number, event: Event) {
  const decision = (event.target as HTMLTextAreaElement).value.trim() || undefined;
  updateChain(index, { decision });
}

function setGateMode(index: number, event: Event) {
  const mode = (event.target as HTMLSelectElement).value as GateMode;
  updateGate(index, { mode, approver: mode === 'approve' ? (plan.value?.chains[index]?.gate.approver ?? '') : undefined });
}

function setApprover(index: number, event: Event) {
  updateGate(index, { approver: (event.target as HTMLInputElement).value.trim() });
}

function setFailure(index: number, event: Event) {
  const onFailure = (event.target as HTMLSelectElement).value as FailurePolicy;
  updateGate(index, { onFailure, maxRetries: onFailure === 'retry' ? (plan.value?.chains[index]?.gate.maxRetries ?? 3) : undefined });
}

function setMaxRetries(index: number, event: Event) {
  const parsed = Number.parseInt((event.target as HTMLInputElement).value, 10);
  updateGate(index, { maxRetries: Number.isFinite(parsed) && parsed > 0 ? parsed : 1 });
}

function addInput(index: number) {
  const chain = plan.value?.chains[index];
  if (!chain) return;
  updateChain(index, { inputs: [...chain.inputs, { port: '', from: 'parameter' }] });
}

function removeInput(chainIndex: number, inputIndex: number) {
  const chain = plan.value?.chains[chainIndex];
  if (!chain) return;
  updateChain(chainIndex, { inputs: chain.inputs.filter((_, i) => i !== inputIndex) });
}

function patchInput(chainIndex: number, inputIndex: number, patch: Partial<InputBinding>) {
  const chain = plan.value?.chains[chainIndex];
  if (!chain) return;
  updateChain(chainIndex, {
    inputs: chain.inputs.map((binding, i) => (i === inputIndex ? { ...binding, ...patch } : binding)),
  });
}

function setInputPort(chainIndex: number, inputIndex: number, event: Event) {
  patchInput(chainIndex, inputIndex, { port: (event.target as HTMLInputElement).value.trim() });
}

function setInputSource(chainIndex: number, inputIndex: number, event: Event) {
  const from = (event.target as HTMLSelectElement).value as BindingSource;
  patchInput(chainIndex, inputIndex, { from, ref: undefined, value: undefined });
}

function setInputRef(chainIndex: number, inputIndex: number, event: Event) {
  patchInput(chainIndex, inputIndex, { ref: (event.target as HTMLSelectElement).value || undefined });
}

function setInputValue(chainIndex: number, inputIndex: number, event: Event) {
  patchInput(chainIndex, inputIndex, { value: (event.target as HTMLInputElement).value.trim() || undefined });
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}
</script>

<template>
  <div ref="paneRoot" class="plan-pane" v-if="plan">
    <div class="plan-header">
      <div class="plan-title">
        <svg class="plan-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="12 2 2 7 12 12 22 7 12 2"/>
          <polyline points="2 17 12 22 22 17"/>
          <polyline points="2 12 12 17 22 12"/>
        </svg>
        <h2>{{ props.agentName }} Plan</h2>
      </div>
      <div class="plan-actions">
        <button class="btn-version" @click="showVersions = !showVersions" :class="{ active: showVersions }">
          Versions
        </button>
        <button class="btn-save-version" @click="emit('save-version')" title="Save version to history & file">Save Version</button>
        <button class="btn-export-agent" @click="emit('export-agent')" title="Save agent to JSON file">Save Agent</button>
      </div>
    </div>

    <div class="plan-content">
      <div class="plan-section">
        <label class="section-label">Goal</label>
        <textarea
          class="plan-input"
          rows="2"
          :value="goalText"
          placeholder="What does this agent accomplish?"
          @change="setGoal"
        ></textarea>
      </div>

      <div class="plan-section">
        <div class="section-header">
          <label class="section-label">Triggers</label>
          <button class="btn-port-add" @click="addTrigger">+ Trigger</button>
        </div>
        <div class="port-list">
          <div v-for="(trigger, index) in triggers" :key="`trigger-${index}`" class="port-row">
            <input class="port-name" :value="trigger.name" @change="renameTrigger(index, $event)" />
            <select class="port-kind" :value="trigger.kind" @change="setTriggerKind(index, $event)">
              <option value="event">event</option>
              <option value="schedule">schedule</option>
              <option value="intent">intent</option>
            </select>
            <input
              v-if="trigger.kind === 'schedule'"
              class="port-schedule"
              :value="trigger.schedule || ''"
              placeholder="0 9 * * 1"
              @change="setTriggerSchedule(index, $event)"
            />
            <button class="btn-remove" @click="removeTrigger(index)" title="Remove trigger">×</button>
          </div>
          <p v-if="!triggers.length" class="plan-steps-empty">No triggers declared.</p>
        </div>
      </div>

      <div class="plan-section">
        <div class="section-header">
          <label class="section-label">Chains</label>
          <div class="section-actions">
            <div class="view-toggle" role="group" aria-label="Chain view">
              <button :class="{ active: view === 'cards' }" @click="view = 'cards'">Cards</button>
              <button :class="{ active: view === 'graph' }" @click="view = 'graph'">Graph</button>
            </div>
            <button class="btn-port-add" @click="addChain">+ Chain</button>
          </div>
        </div>

        <div v-if="view === 'cards'" class="chain-list">
          <div
            v-for="(chain, index) in chains"
            :key="chain.id"
            class="chain-card"
            :class="{ 'is-focused': focusedChainId === chain.id }"
            :data-chain-id="chain.id"
          >
            <div class="chain-head">
              <span class="chain-index">#{{ index + 1 }}</span>
              <span class="chain-flow">
                <span class="flow-chip">{{ chain.trigger || 'unbound' }}</span>
                <span class="flow-arrow">→</span>
                <span class="flow-chip">{{ chain.capability || 'unbound' }}</span>
                <span class="flow-arrow">→</span>
                <span class="flow-chip" :class="{ 'is-terminal': isTerminal(chain.result) }">{{ chain.result || 'unbound' }}</span>
              </span>
              <button class="btn-remove" @click="removeChain(index)" title="Remove chain">×</button>
            </div>

            <div class="chain-grid">
              <label class="field">
                <span class="field-label">Trigger</span>
                <select :value="chain.trigger" @change="setChainTrigger(index, $event)">
                  <option value="">— unbound —</option>
                  <optgroup label="Triggers">
                    <option v-for="trigger in triggers" :key="trigger.name" :value="trigger.name">{{ trigger.name }}</option>
                  </optgroup>
                  <optgroup label="Upstream results">
                    <option v-for="result in results" :key="result.name" :value="result.name">↳ {{ result.name }}</option>
                  </optgroup>
                </select>
              </label>

              <label class="field">
                <span class="field-label">Action</span>
                <input class="chain-capability" :value="chain.capability" placeholder="capability" @change="setChainCapability(index, $event)" />
              </label>

              <label class="field">
                <span class="field-label">Result</span>
                <select :value="chain.result" @change="setChainResult(index, $event)">
                  <option value="">— unbound —</option>
                  <option v-for="result in results" :key="result.name" :value="result.name">
                    {{ result.name }}{{ result.terminal ? ' (deliverable)' : '' }}
                  </option>
                </select>
              </label>

              <label class="field">
                <span class="field-label">Gate</span>
                <select :value="chain.gate.mode" @change="setGateMode(index, $event)">
                  <option value="auto">auto</option>
                  <option value="approve">approve</option>
                </select>
              </label>
            </div>

            <div class="chain-extra">
              <label v-if="chain.gate.mode === 'approve'" class="field">
                <span class="field-label">Approver</span>
                <input :value="chain.gate.approver || ''" placeholder="support_lead" @change="setApprover(index, $event)" />
              </label>
              <label class="field">
                <span class="field-label">On failure</span>
                <select :value="chain.gate.onFailure" @change="setFailure(index, $event)">
                  <option value="stop">stop</option>
                  <option value="escalate">escalate</option>
                  <option value="retry">retry</option>
                </select>
              </label>
              <label v-if="chain.gate.onFailure === 'retry'" class="field">
                <span class="field-label">Max retries</span>
                <input type="number" min="1" :value="chain.gate.maxRetries ?? 3" @change="setMaxRetries(index, $event)" />
              </label>
            </div>

            <label class="field field-wide">
              <span class="field-label">Decision</span>
              <textarea
                rows="2"
                :value="chain.decision || ''"
                placeholder="What may the model decide, and what must it not invent?"
                @change="setDecision(index, $event)"
              ></textarea>
            </label>

            <div class="bindings">
              <div class="bindings-head">
                <span class="field-label">Inputs</span>
                <button class="btn-port-add" @click="addInput(index)">+ Input</button>
              </div>
              <div v-for="(binding, inputIndex) in chain.inputs" :key="inputIndex" class="binding-row">
                <input class="binding-port" :value="binding.port" placeholder="input name" @change="setInputPort(index, inputIndex, $event)" />
                <select class="binding-source" :value="binding.from" @change="setInputSource(index, inputIndex, $event)">
                  <option value="trigger">trigger</option>
                  <option value="result">result</option>
                  <option value="parameter">parameter</option>
                  <option value="literal">literal</option>
                </select>
                <select
                  v-if="binding.from === 'trigger'"
                  class="binding-ref"
                  :value="binding.ref || ''"
                  @change="setInputRef(index, inputIndex, $event)"
                >
                  <option value="">— pick trigger —</option>
                  <option v-for="trigger in triggers" :key="trigger.name" :value="trigger.name">{{ trigger.name }}</option>
                </select>
                <select
                  v-else-if="binding.from === 'result'"
                  class="binding-ref"
                  :value="binding.ref || ''"
                  @change="setInputRef(index, inputIndex, $event)"
                >
                  <option value="">— pick result —</option>
                  <option v-for="result in results" :key="result.name" :value="result.name">{{ result.name }}</option>
                </select>
                <input
                  v-else
                  class="binding-ref"
                  :value="binding.value || ''"
                  :placeholder="binding.from === 'literal' ? 'literal value' : 'parameter name'"
                  @change="setInputValue(index, inputIndex, $event)"
                />
                <button class="btn-remove" @click="removeInput(index, inputIndex)" title="Remove input">×</button>
              </div>
              <p v-if="!chain.inputs.length" class="plan-steps-empty">No inputs bound.</p>
            </div>
          </div>
          <p v-if="!chains.length" class="plan-steps-empty">No chains yet. Add one to wire a trigger to an action.</p>
        </div>

        <div v-else class="chain-graph">
          <div v-if="chains.length" class="graph-gate">
            <span class="graph-gate-title">Gate</span>
            <select class="graph-gate-chain" :value="activeLane" @change="selectLane" aria-label="Chain to edit">
              <option v-for="(row, index) in graphRows" :key="row.chain.id" :value="index">
                #{{ index + 1 }} · {{ row.chain.capability || 'unbound' }}
              </option>
            </select>
            <template v-if="gateChain">
              <select :value="gateChain.gate.mode" aria-label="Gate mode" @change="setGateMode(activeLane, $event)">
                <option value="auto">auto</option>
                <option value="approve">approve</option>
              </select>
              <input
                v-if="gateChain.gate.mode === 'approve'"
                :value="gateChain.gate.approver || ''"
                placeholder="approver"
                aria-label="Approver"
                @change="setApprover(activeLane, $event)"
              />
              <select :value="gateChain.gate.onFailure" aria-label="On failure" @change="setFailure(activeLane, $event)">
                <option value="stop">stop</option>
                <option value="escalate">escalate</option>
                <option value="retry">retry</option>
              </select>
              <input
                v-if="gateChain.gate.onFailure === 'retry'"
                type="number"
                min="1"
                :value="gateChain.gate.maxRetries ?? 3"
                aria-label="Max retries"
                @change="setMaxRetries(activeLane, $event)"
              />
            </template>
          </div>
          <svg
            v-if="chains.length"
            class="graph-svg"
            :viewBox="`0 0 ${GRAPH_WIDTH} ${graphHeight}`"
            role="img"
            :aria-label="`${chains.length} chains: trigger to action to result`"
          >
            <defs>
              <marker id="chain-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                <path d="M 0 0 L 10 5 L 0 10 z" class="arrow-head" />
              </marker>
              <marker id="chain-arrow-accent" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                <path d="M 0 0 L 10 5 L 0 10 z" class="arrow-head accent" />
              </marker>
            </defs>

            <g class="graph-edges">
              <path
                v-for="edge in graphEdges"
                :key="edge.id"
                :d="edge.path"
                :class="['edge', edge.kind]"
                :marker-end="edge.kind === 'handoff' ? 'url(#chain-arrow-accent)' : 'url(#chain-arrow)'"
              />
            </g>

            <g class="graph-nodes">
              <g
                v-for="(row, index) in graphRows"
                :key="row.chain.id"
                class="graph-row"
                role="button"
                :tabindex="index === activeLane ? 0 : -1"
                :data-lane-index="index"
                :aria-label="`Edit chain ${index + 1}: ${row.title}`"
                :transform="`translate(0 ${row.y})`"
                @click="focusChain(index)"
                @focus="focusedLane = index"
                @keydown.enter.prevent="focusChain(index)"
                @keydown.space.prevent="focusChain(index)"
                @keydown.down.prevent="moveLane(index, 1)"
                @keydown.up.prevent="moveLane(index, -1)"
              >
                <title>{{ row.title }}</title>

                <rect :x="X_TRIGGER" y="0" :width="W_TRIGGER" :height="NODE_H" rx="8" :class="['node', 'node-trigger', row.triggerClass]" />
                <text :x="X_TRIGGER + W_TRIGGER / 2" :y="NODE_H / 2 + 4" class="node-label">{{ row.triggerLabel }}</text>

                <line :x1="X_TRIGGER + W_TRIGGER" :y1="NODE_H / 2" :x2="X_ACTION" :y2="NODE_H / 2" class="edge flow" marker-end="url(#chain-arrow)" />

                <rect :x="X_ACTION" y="0" :width="W_ACTION" :height="NODE_H" rx="8" :class="['node', 'node-action', { 'is-gated': row.gated }]" />
                <text :x="X_ACTION + W_ACTION / 2" :y="NODE_H / 2 + 4" class="node-label">{{ row.actionLabel }}</text>

                <line :x1="X_ACTION + W_ACTION" :y1="NODE_H / 2" :x2="X_RESULT" :y2="NODE_H / 2" class="edge flow" marker-end="url(#chain-arrow)" />

                <rect :x="X_RESULT" y="0" :width="W_RESULT" :height="NODE_H" rx="8" :class="['node', 'node-result', { 'is-terminal': row.resultTerminal }]" />
                <text :x="X_RESULT + W_RESULT / 2" :y="NODE_H / 2 + 4" class="node-label">{{ row.resultLabel }}</text>

                <line :x1="X_RESULT + W_RESULT" :y1="NODE_H / 2" :x2="X_GATE" :y2="NODE_H / 2" class="edge flow" marker-end="url(#chain-arrow)" />

                <rect :x="X_GATE" y="0" :width="W_GATE" :height="NODE_H" rx="8" :class="['node', 'node-gate', { 'is-gated': row.gated }]" />
                <text :x="X_GATE + W_GATE / 2" :y="NODE_H / 2 + 4" class="node-label">{{ row.gateLabel }}</text>
              </g>
            </g>
          </svg>
          <p v-if="chains.length" class="graph-hint">Pick a lane with ↑/↓ to edit its gate; click a node, or press Enter, to jump to its card.</p>
          <p v-else class="plan-steps-empty">No chains yet. Add one to wire a trigger to an action.</p>
        </div>
      </div>

      <div class="plan-section">
        <div class="section-header">
          <label class="section-label">Results</label>
          <button class="btn-port-add" @click="addResult">+ Result</button>
        </div>
        <div class="port-list">
          <div v-for="(result, index) in results" :key="`result-${index}`" class="port-row">
            <input class="port-name" :value="result.name" @change="renameResult(index, $event)" />
            <label class="port-terminal">
              <input type="checkbox" :checked="result.terminal" @change="setResultTerminal(index, $event)" />
              deliverable
            </label>
            <input class="port-schema" :value="result.schema || ''" placeholder="schema (optional)" @change="setResultSchema(index, $event)" />
            <button class="btn-remove" @click="removeResult(index)" title="Remove result">×</button>
          </div>
          <p v-if="!results.length" class="plan-steps-empty">No results declared.</p>
        </div>
      </div>

      <div class="plan-section">
        <label class="section-label">Capabilities</label>
        <label class="field field-wide">
          <span class="field-label">Tools</span>
          <input :value="toolsText" placeholder="web_search, ticket_system" @change="setTools" />
        </label>
        <label class="field field-wide">
          <span class="field-label">Knowledge sources</span>
          <input :value="knowledgeText" placeholder="faq.md, product_docs.pdf" @change="setKnowledge" />
        </label>
      </div>

      <div v-if="issues.length" class="plan-section issues">
        <div class="section-header">
          <label class="section-label">Checks ({{ issues.length }})</label>
        </div>
        <ul class="issue-list">
          <li v-for="(issue, index) in issues" :key="index" :class="issue.code">
            <span class="issue-code">{{ issue.code }}</span>
            {{ issue.message }}
          </li>
        </ul>
      </div>
    </div>

    <div v-if="showVersions" class="version-history">
      <h4>Plan Versions</h4>
      <div class="version-list">
        <div v-for="v of props.versions" :key="v.id" class="version-item">
          <div class="version-info">
            <div class="version-label">{{ v.label || 'Untitled version' }}</div>
            <div class="version-date">{{ formatDate(v.created_at) }}</div>
          </div>
          <button class="btn-restore" @click="emit('restore-version', v.id)">Restore</button>
          <button class="btn-compare" @click="emit('compare-version', v.id)">Compare</button>
        </div>
      </div>
      <div v-if="props.versions.length === 0" class="version-empty">
        No saved versions yet.
      </div>
    </div>

    <div v-if="compareVersion" class="plan-diff-view">
      <h4>Plan Version Comparison</h4>
      <div class="diff-info">
        <span class="diff-current">Current: {{ currentSummary?.goal || 'No plan' }}</span>
        <span class="diff-arrow">↔</span>
        <span class="diff-version">{{ compareVersion.label || 'Untitled version' }}: {{ previousSummary?.goal || 'No plan' }}</span>
      </div>
      <div class="diff-sections">
        <div v-for="row in diffRows" :key="row.field" class="diff-section">
          <h5>{{ row.label }}</h5>
          <div class="diff-current-text">{{ row.current }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ row.previous }}</div>
        </div>
      </div>
      <button class="btn-close-diff" @click="emit('compare-version', null)">Close Comparison</button>
    </div>
  </div>

  <div v-else class="plan-empty">
    <div class="empty-icon">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="12 2 2 7 12 12 22 7 12 2"/>
        <polyline points="2 17 12 22 22 17"/>
        <polyline points="2 12 12 17 22 12"/>
      </svg>
    </div>
    <p>Start a conversation with Oppy to build your agent plan.</p>
  </div>
</template>

<style scoped>
.plan-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--surface-card);
}

.plan-header {
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
  background: var(--surface-hover);
}

.plan-icon {
  width: 24px;
  height: 24px;
  color: var(--primary);
}

.plan-title h2 {
  margin: 0;
  font-size: var(--type-xl);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.plan-actions {
  margin-left: auto;
  display: flex;
  gap: var(--space-2);
}

.plan-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}

.plan-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.section-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.plan-input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  font-family: inherit;
  resize: vertical;
  min-height: 64px;
  transition: border-color 0.2s, box-shadow 0.2s;
  background: var(--surface-card);
  color: var(--text-primary);
}

.plan-input:focus,
.field input:focus,
.field select:focus,
.field textarea:focus,
.port-row input:focus,
.port-row select:focus,
.binding-row input:focus,
.binding-row select:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.1);
}

.plan-steps-empty {
  color: var(--text-muted);
  font-style: italic;
  margin: 0;
  font-size: var(--type-sm);
}

.btn-port-add {
  padding: var(--space-1) var(--space-3);
  font-size: var(--type-xs);
  font-weight: 600;
  background: var(--surface-card);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.btn-port-add:hover {
  background: var(--surface-hover);
  color: var(--primary);
  border-color: var(--border-hover);
}

.btn-remove {
  width: 24px;
  height: 24px;
  line-height: 1;
  font-size: 16px;
  background: transparent;
  color: var(--text-muted);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  flex-shrink: 0;
}

.btn-remove:hover {
  color: var(--danger);
  border-color: var(--danger);
}

.port-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.port-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-hover);
}

.port-row input,
.port-row select,
.field input,
.field select,
.field textarea,
.binding-row input,
.binding-row select {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  font-size: var(--type-sm);
  font-family: inherit;
}

.port-name {
  flex: 1;
  min-width: 120px;
  font-weight: 600;
}

.port-kind {
  width: 110px;
}

.port-schedule {
  width: 140px;
}

.port-schema {
  flex: 1;
  min-width: 100px;
}

.port-terminal {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--type-xs);
  color: var(--text-secondary);
  white-space: nowrap;
}

.section-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-left: auto;
}

.view-toggle {
  display: flex;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.view-toggle button {
  padding: 2px var(--space-3);
  font-size: var(--type-xs);
  font-weight: 600;
  background: var(--surface-card);
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
}

.view-toggle button.active {
  background: var(--primary);
  color: var(--primary-fg);
}

.chain-graph {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-hover);
  padding: var(--space-2);
  overflow-x: auto;
}

.graph-svg {
  width: 100%;
  height: auto;
  display: block;
  min-width: 620px;
}

.graph-gate {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-2);
  padding: var(--space-2);
  margin-bottom: var(--space-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-card);
}

.graph-gate-title {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.graph-gate select,
.graph-gate input {
  padding: 2px var(--space-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  font-size: var(--type-xs);
  font-family: inherit;
}

.graph-gate-chain {
  max-width: 240px;
  font-weight: 600;
}

.graph-gate input[type='number'] {
  width: 64px;
}

.graph-row {
  cursor: pointer;
  outline: none;
}

.graph-row:hover .node,
.graph-row:focus-visible .node {
  stroke-width: 2;
}

.graph-hint {
  margin: var(--space-1) 0 0;
  font-size: var(--type-xs);
  color: var(--text-muted);
  text-align: center;
}

.node {
  fill: var(--surface-card);
  stroke: var(--border);
  stroke-width: 1;
}

.node-trigger.kind-schedule {
  stroke: var(--success);
}

.node-trigger.kind-intent {
  stroke: var(--primary);
}

.node-trigger.is-handoff {
  stroke: var(--text-muted);
  stroke-dasharray: 4 3;
}

.node-action.is-gated {
  stroke: var(--primary);
  stroke-width: 2;
}

.node-gate.is-gated {
  stroke: var(--primary);
  stroke-width: 2;
  fill: var(--surface-hover);
}

.node-result.is-terminal {
  stroke: var(--primary);
  stroke-width: 2;
}

.node-label {
  fill: var(--text-primary);
  font-size: 11px;
  font-weight: 600;
  font-family: inherit;
  text-anchor: middle;
}

.edge {
  fill: none;
  stroke: var(--text-muted);
  stroke-width: 1.5;
}

.edge.flow {
  stroke: var(--border-hover);
}

.edge.handoff {
  stroke: var(--primary);
}

.edge.input {
  stroke: var(--text-muted);
  stroke-width: 1.2;
  stroke-dasharray: 4 3;
}

.arrow-head {
  fill: var(--text-muted);
}

.arrow-head.accent {
  fill: var(--primary);
}

.chain-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.chain-card {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  padding: var(--space-3) var(--space-4) var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.chain-card.is-focused {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.18);
}

.chain-head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  border-bottom: 1px dashed var(--border);
  padding-bottom: var(--space-2);
}

.chain-index {
  font-size: var(--type-xs);
  font-weight: 700;
  color: var(--text-muted);
}

.chain-flow {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.flow-chip {
  font-size: var(--type-xs);
  font-weight: 600;
  padding: 2px var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--surface-hover);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.flow-chip.is-terminal {
  color: var(--primary);
  border-color: var(--primary);
}

.flow-arrow {
  color: var(--text-muted);
  font-size: var(--type-xs);
}

.chain-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.chain-extra {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.field-wide {
  grid-column: 1 / -1;
}

.field-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.field textarea {
  resize: vertical;
  min-height: 48px;
}

.bindings {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  border-top: 1px dashed var(--border);
  padding-top: var(--space-3);
}

.bindings-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.binding-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.binding-port {
  width: 130px;
}

.binding-source {
  width: 110px;
}

.binding-ref {
  flex: 1;
  min-width: 120px;
}

.issues .section-label {
  color: var(--text-secondary);
}

.issue-list {
  margin: 0;
  padding-left: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  font-size: var(--type-sm);
  color: var(--text-secondary);
}

.issue-code {
  font-size: var(--type-xs);
  font-weight: 700;
  color: var(--text-muted);
  margin-right: var(--space-1);
}

.issue-list li.missing-approver .issue-code,
.issue-list li.unbound-trigger .issue-code,
.issue-list li.unbound-action .issue-code,
.issue-list li.unbound-result .issue-code {
  color: var(--danger);
}

.version-history {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
}

.version-history h4 {
  margin: 0 0 var(--space-3) 0;
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  max-height: 200px;
  overflow-y: auto;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.version-label {
  font-size: var(--type-base);
  font-weight: 500;
  color: var(--text-primary);
}

.version-date {
  font-size: var(--type-xs);
  color: var(--text-secondary);
}

.btn-restore {
  padding: var(--space-1) var(--space-3);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 600;
}

.btn-restore:hover {
  background: var(--primary-hover);
}

.btn-compare {
  padding: var(--space-1) var(--space-3);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 500;
}

.btn-compare:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.version-empty {
  font-size: var(--type-sm);
  color: var(--text-muted);
  text-align: center;
  padding: var(--space-4);
}

.btn-version, .btn-save-version, .btn-export-agent {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: background 0.2s, border-color 0.2s;
}

.btn-version {
  background: var(--surface-card);
  color: var(--text-secondary);
}

.btn-version:hover {
  background: var(--surface-hover);
}

.btn-version.active {
  background: var(--surface-hover);
  color: var(--primary);
}

.btn-save-version {
  background: var(--success);
  color: var(--success-fg);
}

.btn-save-version:hover {
  background: #7BB3E8;
}

.btn-export-agent {
  background: var(--primary);
  color: var(--primary-fg);
}

.btn-export-agent:hover {
  background: var(--primary-hover);
}

.plan-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  padding: var(--space-8);
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
  margin-bottom: var(--space-3);
}

.plan-empty p {
  font-size: var(--type-base);
  margin: 0;
}

.plan-diff-view {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
}

.diff-info {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  margin-bottom: var(--space-4);
  padding: var(--space-3);
  background: var(--surface-card);
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
}

.diff-current,
.diff-version {
  font-size: var(--type-sm);
  color: var(--text-secondary);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-current {
  font-weight: 600;
  color: var(--text-primary);
}

.diff-version {
  font-weight: 600;
  color: var(--primary);
}

.diff-arrow {
  font-size: var(--type-xs);
  color: var(--text-muted);
}

.diff-sections {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.diff-section {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3);
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}

.diff-section h5 {
  margin: 0;
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 140px;
}

.diff-current-text,
.diff-version-text {
  font-size: var(--type-sm);
  color: var(--text-primary);
  word-break: break-word;
}

.diff-current-text {
  font-weight: 500;
}

.diff-version-text {
  font-weight: 500;
}

.btn-close-diff {
  margin-top: var(--space-4);
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--danger);
  color: var(--danger-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
}

.btn-close-diff:hover {
  background: var(--danger);
}
</style>
