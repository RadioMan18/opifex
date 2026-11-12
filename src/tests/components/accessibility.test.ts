import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import { useGraphStore } from '@/stores/graphStore';
import BlockPalette from '@/components/BlockPalette.vue';
import NodeInspector from '@/components/NodeInspector.vue';
import type { BlockDefinition, BlockKind } from '@/types/agent';
import axe from 'axe-core';

function makeBlock(id: string, name: string, kind: BlockKind = 'utility', category: string = 'Test'): BlockDefinition {
  return {
    id,
    version: 1,
    name,
    description: 'A test block',
    category,
    kind,
    origin: 'user',
    inputs: [{ id: 'input', name: 'Input', type: 'any', direction: 'input', required: true }],
    outputs: [{ id: 'output', name: 'Output', type: 'any', direction: 'output', required: false }],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

describe('Accessibility (axe-core)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    const store = useGraphStore();
    store.setGraph({
      id: 'graph-1',
      name: 'Test Graph',
      description: undefined,
      schemaVersion: 1,
      nodes: [],
      edges: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });
    store.blocks = [
      makeBlock('b1', 'Input Block', 'trigger', 'Triggers'),
      makeBlock('b2', 'LLM Block', 'llm', 'LLM'),
      makeBlock('b3', 'Tool Block', 'tool', 'Tools'),
    ];
  });

  it('BlockPalette has proper ARIA labels and keyboard attributes', async () => {
    const wrapper = mount(BlockPalette, {
      props: { blocks: useGraphStore().blocks },
      attachTo: document.body,
    });

    // Check search input has aria-label
    const searchInput = wrapper.find('input');
    expect(searchInput.exists()).toBe(true);
    expect(searchInput.attributes('aria-label')).toBe('Search block palette');

    // Check draggable blocks exist
    const draggableBlocks = wrapper.findAll('[draggable="true"]');
    expect(draggableBlocks.length).toBeGreaterThan(0);

    // Run axe-core scan
    const results = await axe.run(wrapper.element as Element);
    const violations = results.violations ?? [];
    const criticalViolations = violations.filter((v: any) => v.impact === 'critical');
    const seriousViolations = violations.filter((v: any) => v.impact === 'serious');

    expect(criticalViolations.length).toBe(0);
    expect(seriousViolations.length).toBe(0);

    wrapper.unmount();
  });

  it('NodeInspector passes axe-core accessibility scan (no node selected)', async () => {
    const wrapper = mount(NodeInspector, {
      props: { nodeId: undefined },
      attachTo: document.body,
    });

    const results = await axe.run(wrapper.element as Element);
    const violations = results.violations ?? [];
    const criticalViolations = violations.filter((v: any) => v.impact === 'critical');
    const seriousViolations = violations.filter((v: any) => v.impact === 'serious');

    expect(criticalViolations.length).toBe(0);
    expect(seriousViolations.length).toBe(0);

    wrapper.unmount();
  });

  it('NodeInspector form inputs are properly labeled when node has parameters', async () => {
    const store = useGraphStore();
    // Add an LLM block which has parameters (model, temperature, max_tokens)
    const llmBlock: BlockDefinition = {
      id: 'b-llm',
      version: 1,
      name: 'LLM Prompt',
      description: 'LLM block',
      category: 'LLM',
      kind: 'llm',
      origin: 'user',
      inputs: [{ id: 'message', name: 'Message', type: 'string', direction: 'input', required: true }],
      outputs: [{ id: 'completion', name: 'Completion', type: 'string', direction: 'output', required: false }],
      parameters: [
        {
          id: 'model',
          name: 'model',
          type: 'string',
          label: 'Model',
          required: false,
          defaultValue: 'llama3.2',
          sensitive: false,
        },
      ],
      specJson: {},
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    store.blocks = [llmBlock, ...store.blocks];

    const nodeId = store.addNode({
      blockId: 'b-llm',
      blockVersion: 1,
      position: { x: 0, y: 0 },
      parameters: {},
      subGraph: undefined,
      isCollapsed: false,
    })!;

    const wrapper = mount(NodeInspector, {
      props: { nodeId },
      attachTo: document.body,
    });

    // Check that labels are associated with inputs
    const labels = wrapper.findAll('label');
    expect(labels.length).toBeGreaterThan(0);

    // Check that each label has a corresponding input
    for (const label of labels) {
      const forAttr = label.attributes('for');
      if (forAttr) {
        const input = wrapper.find(`#${forAttr}`);
        expect(input.exists()).toBe(true);
      }
    }

    const results = await axe.run(wrapper.element as Element);
    const violations = results.violations ?? [];
    const criticalViolations = violations.filter((v: any) => v.impact === 'critical');
    const seriousViolations = violations.filter((v: any) => v.impact === 'serious');

    expect(criticalViolations.length).toBe(0);
    expect(seriousViolations.length).toBe(0);

    wrapper.unmount();
  });
});
