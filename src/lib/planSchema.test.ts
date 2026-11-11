import { describe, expect, it } from 'vitest';
import {
  emptyPlan,
  migratePlan,
  needsPlanMigration,
  outputNames,
  planIssues,
  planSteps,
  planSummary,
  renameResultPort,
  renameTriggerPort,
  triggerNames,
} from '@/lib/planSchema';
import type { AgentPlan } from '@/types/agent';

const legacyPlan = {
  goal: 'Answer customer questions',
  steps: ['Understand the issue', 'Search the knowledge base', 'Draft a reply'],
  tools: ['knowledge_base'],
  knowledgeSources: ['faq.md'],
  triggers: ['customer_message'],
  outputs: ['customer_response'],
};

describe('migratePlan', () => {
  it('turns a legacy flat plan into a pipeline ending in its declared output', () => {
    const plan = migratePlan(legacyPlan);

    expect(plan.version).toBe(2);
    expect(plan.triggers).toEqual([{ name: 'customer_message', kind: 'event' }]);
    expect(plan.chains.map(chain => chain.capability)).toEqual(legacyPlan.steps);
    expect(plan.chains[0].trigger).toBe('customer_message');
    expect(plan.chains[1].trigger).toBe(plan.chains[0].result);
    expect(plan.chains[2].trigger).toBe(plan.chains[1].result);
    expect(plan.chains[2].result).toBe('customer_response');
    expect(plan.chains[2].gate).toEqual({ mode: 'auto', onFailure: 'stop' });
    expect(outputNames(plan)).toEqual(['customer_response']);
  });

  it('keeps every step as a chain when there are fewer outputs than steps', () => {
    const plan = migratePlan({ ...legacyPlan, outputs: ['customer_response', 'support_ticket'] });

    expect(plan.chains).toHaveLength(3);
    expect(plan.chains[0].result).toBe('step_1_result');
    expect(plan.results.find(port => port.name === 'step_1_result')?.terminal).toBe(false);
    expect(outputNames(plan)).toEqual(['customer_response', 'support_ticket']);
  });

  it('is idempotent', () => {
    const once = migratePlan(legacyPlan);
    expect(migratePlan(once)).toEqual(once);
  });

  it('leaves a schema v2 plan untouched', () => {
    const plan: AgentPlan = {
      version: 2,
      goal: 'Ship reviews',
      triggers: [{ name: 'pull_request_opened', kind: 'event' }],
      results: [{ name: 'approval_status', terminal: true, schema: 'enum' }],
      chains: [
        {
          id: 'cr-1',
          trigger: 'pull_request_opened',
          capability: 'approval_decider',
          inputs: [],
          result: 'approval_status',
          gate: { mode: 'approve', approver: 'maintainer', onFailure: 'escalate' },
        },
      ],
      tools: ['approval_decider'],
      knowledgeSources: [],
    };

    expect(migratePlan(plan)).toEqual(plan);
    expect(needsPlanMigration(plan)).toBe(false);
  });

  it('declares ports a chain references but the plan never lists', () => {
    const plan = migratePlan({
      version: 2,
      goal: 'Handoff',
      chains: [{ trigger: 'upstream', capability: 'do_work', result: 'out' }],
    });

    expect(plan.triggers).toEqual([]);
    expect(plan.results).toEqual([
      { name: 'upstream', terminal: false },
      { name: 'out', terminal: true },
    ]);
  });

  it('detects plans that still need migrating', () => {
    expect(needsPlanMigration(legacyPlan)).toBe(true);
    expect(needsPlanMigration(undefined)).toBe(true);
  });
});

describe('derived views', () => {
  it('projects chains back onto the flat pane fields', () => {
    const plan = migratePlan(legacyPlan);

    expect(planSteps(plan).map(step => step.action)).toEqual(legacyPlan.steps);
    expect(planSummary(plan)).toMatchObject({
      goal: 'Answer customer questions',
      triggers: 'customer_message',
      outputs: 'customer_response',
      tools: 'knowledge_base',
    });
  });
});

describe('rename ports', () => {
  it('carries a trigger rename through the chains that start from it', () => {
    const plan = migratePlan(legacyPlan);
    const renamed = renameTriggerPort(plan, 'customer_message', 'support_email');

    expect(triggerNames(renamed)).toEqual(['support_email']);
    expect(renamed.chains[0].trigger).toBe('support_email');
  });

  it('carries a result rename through handoffs', () => {
    const plan = migratePlan(legacyPlan);
    const renamed = renameResultPort(plan, 'step_1_result', 'issue_summary');

    expect(renamed.results.some(port => port.name === 'issue_summary')).toBe(true);
    expect(renamed.chains[0].result).toBe('issue_summary');
    expect(renamed.chains[1].trigger).toBe('issue_summary');
  });
});

describe('planIssues', () => {
  it('reports a chain that cannot run', () => {
    const plan: AgentPlan = {
      ...emptyPlan(),
      goal: 'Broken',
      triggers: [
        { name: 'a', kind: 'event' },
        { name: 'b', kind: 'event' },
        { name: 'c', kind: 'schedule' },
      ],
      results: [{ name: 'internal', terminal: false }],
      chains: [
        { id: 'c1', trigger: 'a', capability: '', inputs: [], result: '', gate: { mode: 'auto', onFailure: 'retry' } },
      ],
    };

    const codes = planIssues(plan).map(issue => issue.code);
    expect(codes).toContain('unbound-action');
    expect(codes).toContain('unbound-result');
    expect(codes).toContain('unused-trigger');
    expect(codes).toContain('unproduced-result');
    expect(codes).toContain('schedule-missing');
    expect(codes).toContain('retry-without-limit');
  });

  it('requires an approver for an approval gate', () => {
    const plan: AgentPlan = {
      ...emptyPlan(),
      goal: 'Send money',
      triggers: [{ name: 'invoice_received', kind: 'event' }],
      results: [{ name: 'payment', terminal: true }],
      chains: [
        {
          id: 'c1',
          trigger: 'invoice_received',
          capability: 'pay_invoice',
          inputs: [],
          result: 'payment',
          gate: { mode: 'approve', onFailure: 'stop' },
        },
      ],
    };

    expect(planIssues(plan).map(issue => issue.code)).toEqual(['missing-approver']);
  });

  it('passes a fully wired plan', () => {
    expect(planIssues(migratePlan(legacyPlan))).toEqual([]);
  });
});
