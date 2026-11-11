import type {
  AgentChain,
  AgentPlan,
  AgentPlanStep,
  BindingSource,
  ChainGate,
  FailurePolicy,
  GateMode,
  InputBinding,
  PlanSummary,
  ResultPort,
  TriggerKind,
  TriggerPort,
} from '@/types/agent';

export const PLAN_VERSION = 2 as const;

const TRIGGER_KINDS: readonly TriggerKind[] = ['event', 'schedule', 'intent'];
const GATE_MODES: readonly GateMode[] = ['auto', 'approve'];
const FAILURE_POLICIES: readonly FailurePolicy[] = ['stop', 'retry', 'escalate'];
const BINDING_SOURCES: readonly BindingSource[] = ['trigger', 'result', 'parameter', 'literal'];

type JsonObject = Record<string, unknown>;

interface PlanBase {
  goal: string;
  tools: string[];
  knowledgeSources: string[];
}

function asObject(value: unknown): JsonObject | null {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as JsonObject)
    : null;
}

function asText(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}

function asTextList(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .map(item => (typeof item === 'string' ? item.trim() : ''))
    .filter(item => item.length > 0);
}

function oneOf<T>(value: unknown, allowed: readonly T[], fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

function portName(value: unknown): string {
  if (typeof value === 'string') return value.trim();
  const record = asObject(value);
  return record ? asText(record.name) : '';
}

export function emptyPlan(): AgentPlan {
  return {
    version: PLAN_VERSION,
    goal: '',
    triggers: [],
    results: [],
    chains: [],
    tools: [],
    knowledgeSources: [],
  };
}

/** True when the document is already schema v2. */
export function isPlanV2(raw: unknown): boolean {
  const record = asObject(raw);
  if (!record) return false;
  return record.version === PLAN_VERSION || Array.isArray(record.chains);
}

/** True when stored data still uses the flat `steps` / `triggers` / `outputs` shape. */
export function needsPlanMigration(raw: unknown): boolean {
  return !isPlanV2(raw);
}

function normalizeTriggerPort(value: unknown): TriggerPort | null {
  const record = asObject(value);
  const name = portName(value);
  if (!name) return null;
  const kind = oneOf(record?.kind, TRIGGER_KINDS, 'event');
  const port: TriggerPort = { name, kind };
  const description = asText(record?.description);
  if (description) port.description = description;
  const schedule = asText(record?.schedule);
  if (schedule) port.schedule = schedule;
  return port;
}

function normalizeResultPort(value: unknown): ResultPort | null {
  const record = asObject(value);
  const name = portName(value);
  if (!name) return null;
  const port: ResultPort = { name, terminal: record ? record.terminal !== false : true };
  const description = asText(record?.description);
  if (description) port.description = description;
  const schema = asText(record?.schema);
  if (schema) port.schema = schema;
  return port;
}

function normalizeGate(value: unknown): ChainGate {
  const record = asObject(value);
  const gate: ChainGate = {
    mode: oneOf(record?.mode, GATE_MODES, 'auto'),
    onFailure: oneOf(record?.onFailure, FAILURE_POLICIES, 'stop'),
  };
  const approver = asText(record?.approver);
  if (approver) gate.approver = approver;
  if (typeof record?.maxRetries === 'number' && Number.isFinite(record.maxRetries) && record.maxRetries >= 0) {
    gate.maxRetries = Math.floor(record.maxRetries);
  }
  return gate;
}

function normalizeBinding(value: unknown): InputBinding | null {
  const record = asObject(value);
  if (!record) {
    const port = asText(value);
    return port ? { port, from: 'parameter' } : null;
  }
  const port = asText(record.port) || asText(record.name);
  if (!port) return null;
  const binding: InputBinding = {
    port,
    from: oneOf(record.from, BINDING_SOURCES, 'parameter'),
  };
  const ref = asText(record.ref);
  if (ref) binding.ref = ref;
  const literal = record.value === undefined || record.value === null ? '' : String(record.value).trim();
  if (literal) binding.value = literal;
  return binding;
}

function normalizeChain(value: unknown, index: number): AgentChain | null {
  const record = asObject(value);
  if (!record) return null;
  const capability = asText(record.capability) || asText(record.action);
  if (!capability) return null;
  const rawInputs = Array.isArray(record.inputs) ? record.inputs : [];
  return {
    id: asText(record.id) || `chain-${index + 1}`,
    trigger: portName(record.trigger),
    capability,
    inputs: rawInputs
      .map(normalizeBinding)
      .filter((binding): binding is InputBinding => binding !== null),
    result: portName(record.result),
    decision: asText(record.decision) || undefined,
    gate: normalizeGate(record.gate),
  };
}

function uniqueChainId(preferred: string, taken: AgentChain[]): string {
  const base = preferred || `chain-${taken.length + 1}`;
  if (!taken.some(chain => chain.id === base)) return base;
  let suffix = 2;
  while (taken.some(chain => chain.id === `${base}-${suffix}`)) suffix += 1;
  return `${base}-${suffix}`;
}

function baseFrom(source: JsonObject): PlanBase {
  return {
    goal: asText(source.goal),
    tools: asTextList(source.tools),
    knowledgeSources: asTextList(source.knowledgeSources),
  };
}

/** Schema v2 input: declared ports plus chains. */
function fromChainsPlan(source: JsonObject, base: PlanBase): AgentPlan {
  const triggers: TriggerPort[] = [];
  for (const raw of Array.isArray(source.triggers) ? source.triggers : []) {
    const port = normalizeTriggerPort(raw);
    if (port && !triggers.some(existing => existing.name === port.name)) triggers.push(port);
  }

  const results: ResultPort[] = [];
  const addResult = (port: ResultPort) => {
    if (port.name && !results.some(existing => existing.name === port.name)) results.push(port);
  };
  for (const raw of Array.isArray(source.results) ? source.results : []) {
    const port = normalizeResultPort(raw);
    if (port) addResult(port);
  }

  const chains: AgentChain[] = [];
  const consumed = new Set<string>();
  for (const raw of Array.isArray(source.chains) ? source.chains : []) {
    const chain = normalizeChain(raw, chains.length);
    if (!chain) continue;
    consumed.add(chain.trigger);
    for (const binding of chain.inputs) {
      if (binding.ref) consumed.add(binding.ref);
    }
    chains.push(chain);
  }

  for (const chain of chains) {
    // A chain that starts from an undeclared port is a handoff payload, not a
    // trigger, so it is declared as an internal result rather than invented.
    if (chain.trigger && !triggers.some(port => port.name === chain.trigger)) {
      addResult({ name: chain.trigger, terminal: false });
    }
    if (chain.result) {
      addResult({ name: chain.result, terminal: !consumed.has(chain.result) });
    }
  }

  for (const chain of chains) {
    chain.id = uniqueChainId(chain.id, chains.filter(other => other !== chain));
  }

  return {
    version: PLAN_VERSION,
    goal: base.goal,
    triggers,
    results,
    chains,
    tools: base.tools,
    knowledgeSources: base.knowledgeSources,
  };
}

/**
 * Legacy flat plan: ordered `steps` plus parallel `triggers` / `outputs` lists.
 *
 * Steps become a pipeline — the first one starts at the first declared trigger,
 * each later one starts at the previous chain's result. Declared outputs attach
 * to the tail of the pipeline, because that is where a step list produces them.
 */
function fromLegacyPlan(source: JsonObject, base: PlanBase): AgentPlan {
  const triggers: TriggerPort[] = asTextList(source.triggers).map(name => ({ name, kind: 'event' }));
  const outputs = asTextList(source.outputs);
  const rawSteps = Array.isArray(source.steps) ? source.steps : [];

  const results: ResultPort[] = [];
  const addResult = (port: ResultPort) => {
    if (!results.some(existing => existing.name === port.name)) results.push(port);
  };

  const chains: AgentChain[] = [];
  const firstOutput = Math.max(0, rawSteps.length - outputs.length);

  rawSteps.forEach((raw, index) => {
    const record = asObject(raw);
    const capability = typeof raw === 'string' ? raw.trim() : asText(record?.action);
    if (!capability) return;

    const declaredOutput = index >= firstOutput ? outputs[index - firstOutput] : undefined;
    const result = declaredOutput ?? `step_${index + 1}_result`;
    addResult({ name: result, terminal: declaredOutput !== undefined });

    const previous = chains[chains.length - 1];
    const trigger = chains.length === 0 ? (triggers[0]?.name ?? '') : (previous?.result ?? '');

    const gate: ChainGate = record?.requiresApproval === true
      ? { mode: 'approve', onFailure: 'stop' }
      : { mode: 'auto', onFailure: 'stop' };
    const approvalContext = asText(record?.approvalContext);
    if (gate.mode === 'approve' && approvalContext) gate.approver = approvalContext;

    chains.push({
      id: `chain-${chains.length + 1}`,
      trigger,
      capability,
      inputs: asTextList(record?.inputs).map(port => ({ port, from: 'parameter' as BindingSource })),
      result,
      gate,
    });
  });

  for (const output of outputs) addResult({ name: output, terminal: true });

  return {
    version: PLAN_VERSION,
    goal: base.goal,
    triggers,
    results,
    chains,
    tools: base.tools,
    knowledgeSources: base.knowledgeSources,
  };
}

/** Normalizes any stored or model-produced plan into schema v2. Idempotent. */
export function migratePlan(raw: unknown): AgentPlan {
  const source = asObject(raw);
  if (!source) return emptyPlan();
  const base = baseFrom(source);
  if (Array.isArray(source.chains)) return fromChainsPlan(source, base);
  if (Array.isArray(source.steps) || Array.isArray(source.triggers) || Array.isArray(source.outputs)) {
    return fromLegacyPlan(source, base);
  }
  return { ...emptyPlan(), ...base };
}

function formatBinding(binding: InputBinding): string {
  const source = binding.ref ?? binding.value ?? binding.from;
  return `${binding.port} ← ${source}`;
}

export function planSteps(plan: AgentPlan): AgentPlanStep[] {
  return plan.chains.map(chain => ({
    action: chain.capability,
    inputs: chain.inputs.map(formatBinding),
    requiresApproval: chain.gate.mode === 'approve',
    approvalContext: chain.gate.approver,
  }));
}

export function triggerNames(plan: AgentPlan): string[] {
  return plan.triggers.map(port => port.name);
}

export function outputNames(plan: AgentPlan): string[] {
  return plan.results.filter(port => port.terminal).map(port => port.name);
}

export function planSummary(plan: AgentPlan): PlanSummary {
  return {
    goal: plan.goal ?? '',
    steps: plan.chains.map(chain => chain.capability).join(', '),
    triggers: triggerNames(plan).join(', '),
    outputs: outputNames(plan).join(', '),
    tools: plan.tools.join(', '),
    knowledgeSources: plan.knowledgeSources.join(', '),
  };
}

/** Renames a declared trigger and every reference to it, so wiring survives edits. */
export function renameTriggerPort(plan: AgentPlan, from: string, to: string): AgentPlan {
  const name = to.trim();
  if (!name || name === from) return plan;
  return {
    ...plan,
    triggers: plan.triggers.map(port => (port.name === from ? { ...port, name } : port)),
    chains: plan.chains.map(chain => ({
      ...chain,
      trigger: chain.trigger === from ? name : chain.trigger,
      inputs: chain.inputs.map(binding =>
        binding.from === 'trigger' && binding.ref === from ? { ...binding, ref: name } : binding,
      ),
    })),
  };
}

/** Renames a declared result, including the handoffs and inputs that read it. */
export function renameResultPort(plan: AgentPlan, from: string, to: string): AgentPlan {
  const name = to.trim();
  if (!name || name === from) return plan;
  return {
    ...plan,
    results: plan.results.map(port => (port.name === from ? { ...port, name } : port)),
    chains: plan.chains.map(chain => ({
      ...chain,
      trigger: chain.trigger === from ? name : chain.trigger,
      result: chain.result === from ? name : chain.result,
      inputs: chain.inputs.map(binding =>
        binding.from === 'result' && binding.ref === from ? { ...binding, ref: name } : binding,
      ),
    })),
  };
}

export type PlanIssueCode =
  | 'missing-goal'
  | 'no-chains'
  | 'duplicate-port'
  | 'schedule-missing'
  | 'unused-trigger'
  | 'unproduced-result'
  | 'unbound-trigger'
  | 'unbound-action'
  | 'unbound-result'
  | 'duplicate-chain'
  | 'dangling-input'
  | 'missing-approver'
  | 'retry-without-limit'
  | 'orphan-result';

export interface PlanIssue {
  code: PlanIssueCode;
  message: string;
  chainId?: string;
  port?: string;
}

/**
 * Semantic validation. Shape is enforced by the backend; this checks that the
 * wiring actually holds together and that dangerous actions are gated.
 */
export function planIssues(plan: AgentPlan): PlanIssue[] {
  const issues: PlanIssue[] = [];
  if (!plan.goal || !plan.goal.trim()) {
    issues.push({ code: 'missing-goal', message: 'The plan has no goal.' });
  }
  if (plan.chains.length === 0) {
    issues.push({ code: 'no-chains', message: 'The plan has no trigger → action → result chains.' });
  }

  const triggerNamesSet = new Set(plan.triggers.map(port => port.name));
  const resultByName = new Map(plan.results.map(port => [port.name, port]));

  for (const port of plan.triggers) {
    if (port.kind === 'schedule' && !port.schedule) {
      issues.push({ code: 'schedule-missing', port: port.name, message: `Scheduled trigger '${port.name}' has no schedule.` });
    }
  }
  const duplicateTriggers = plan.triggers.filter((port, i) => plan.triggers.findIndex(other => other.name === port.name) !== i);
  for (const port of duplicateTriggers) {
    issues.push({ code: 'duplicate-port', port: port.name, message: `Trigger '${port.name}' is declared more than once.` });
  }
  const duplicateResults = plan.results.filter((port, i) => plan.results.findIndex(other => other.name === port.name) !== i);
  for (const port of duplicateResults) {
    issues.push({ code: 'duplicate-port', port: port.name, message: `Result '${port.name}' is declared more than once.` });
  }

  const produced = new Set(plan.chains.map(chain => chain.result).filter(Boolean));
  const consumed = new Set<string>();
  for (const chain of plan.chains) {
    if (chain.trigger) consumed.add(chain.trigger);
    for (const binding of chain.inputs) {
      if (binding.ref) consumed.add(binding.ref);
    }
  }

  for (const name of triggerNamesSet) {
    if (!consumed.has(name)) {
      issues.push({ code: 'unused-trigger', port: name, message: `Trigger '${name}' does not start any chain.` });
    }
  }
  for (const port of plan.results) {
    if (!produced.has(port.name)) {
      issues.push({ code: 'unproduced-result', port: port.name, message: `No chain produces '${port.name}'.` });
    }
    if (!port.terminal && produced.has(port.name) && !consumed.has(port.name)) {
      issues.push({ code: 'orphan-result', port: port.name, message: `Internal result '${port.name}' is never consumed.` });
    }
  }

  const seenPairs = new Set<string>();
  for (const chain of plan.chains) {
    if (!chain.trigger) {
      issues.push({ code: 'unbound-trigger', chainId: chain.id, message: 'Chain has no trigger.' });
    } else if (!triggerNamesSet.has(chain.trigger) && !resultByName.has(chain.trigger)) {
      issues.push({ code: 'unbound-trigger', chainId: chain.id, port: chain.trigger, message: `Chain starts from unknown port '${chain.trigger}'.` });
    }
    if (!chain.capability) {
      issues.push({ code: 'unbound-action', chainId: chain.id, message: 'Chain has no action.' });
    }
    if (!chain.result) {
      issues.push({ code: 'unbound-result', chainId: chain.id, message: 'Chain has no result.' });
    }

    const pair = `${chain.trigger}→${chain.result}`;
    if (chain.trigger && chain.result && seenPairs.has(pair)) {
      issues.push({ code: 'duplicate-chain', chainId: chain.id, message: `Another chain already wires ${pair}.` });
    }
    seenPairs.add(pair);

    for (const binding of chain.inputs) {
      if (binding.from === 'trigger' && binding.ref && !triggerNamesSet.has(binding.ref)) {
        issues.push({ code: 'dangling-input', chainId: chain.id, port: binding.ref, message: `Input '${binding.port}' reads unknown trigger '${binding.ref}'.` });
      }
      if (binding.from === 'result' && binding.ref && !resultByName.has(binding.ref)) {
        issues.push({ code: 'dangling-input', chainId: chain.id, port: binding.ref, message: `Input '${binding.port}' reads unknown result '${binding.ref}'.` });
      }
    }

    if (chain.gate.mode === 'approve' && !chain.gate.approver) {
      issues.push({ code: 'missing-approver', chainId: chain.id, message: 'Approval gate has no approver.' });
    }
    if (chain.gate.onFailure === 'retry' && (chain.gate.maxRetries ?? 0) < 1) {
      issues.push({ code: 'retry-without-limit', chainId: chain.id, message: 'Retry policy needs maxRetries ≥ 1.' });
    }
  }

  return issues;
}
