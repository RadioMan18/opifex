import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BlockPalette from '@/components/BlockPalette.vue';
import type { BlockDefinition, BlockKind } from '@/types/agent';

function makeBlock(
  id: string,
  name: string,
  kind: BlockKind = 'utility',
  category: string = 'Test'
): BlockDefinition {
  return {
    id,
    version: 1,
    name,
    description: 'A test block',
    category,
    kind,
    origin: 'user',
    inputs: [
      { id: 'input', name: 'Input', type: 'any', direction: 'input', required: true },
    ],
    outputs: [
      { id: 'output', name: 'Output', type: 'any', direction: 'output', required: false },
    ],
    parameters: [],
    specJson: {},
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

describe('BlockPalette', () => {
  it('renders all blocks grouped by category', () => {
    const blocks = [
      makeBlock('b1', 'Input Block', 'trigger', 'Triggers'),
      makeBlock('b2', 'LLM Block', 'llm', 'LLM'),
      makeBlock('b3', 'Tool Block', 'tool', 'Tools'),
    ];

    const wrapper = mount(BlockPalette, {
      props: { blocks },
    });

    expect(wrapper.text()).toContain('Triggers');
    expect(wrapper.text()).toContain('LLM');
    expect(wrapper.text()).toContain('Tools');
    expect(wrapper.text()).toContain('Input Block');
    expect(wrapper.text()).toContain('LLM Block');
    expect(wrapper.text()).toContain('Tool Block');
  });

  it('filters blocks by search query', async () => {
    const blocks = [
      makeBlock('b1', 'HTTP Request', 'tool', 'Tools'),
      makeBlock('b2', 'LLM Prompt', 'llm', 'LLM'),
    ];

    const wrapper = mount(BlockPalette, {
      props: { blocks },
    });

    await wrapper.find('input').setValue('HTTP');
    expect(wrapper.text()).toContain('HTTP Request');
    expect(wrapper.text()).not.toContain('LLM Prompt');
  });

  it('emits block-drag-start when block is dragged', async () => {
    const blocks = [makeBlock('b1', 'Test Block', 'trigger', 'Triggers')];
    const wrapper = mount(BlockPalette, {
      props: { blocks },
    });

    const blockEl = wrapper.find('.cursor-pointer');
    await blockEl.trigger('dragstart', {
      dataTransfer: {
        setData: () => {},
        effectAllowed: '',
      },
    } as unknown as DragEvent);

    const emittedEvents = wrapper.emitted('block-drag-start');
    expect(emittedEvents).toBeTruthy();
    const firstEmit = emittedEvents as unknown[][];
    expect((firstEmit[0][0] as { id: string }).id).toBe('b1');
  });

  it('shows "No blocks found" when search returns nothing', async () => {
    const blocks = [makeBlock('b1', 'Test Block', 'trigger', 'Triggers')];
    const wrapper = mount(BlockPalette, {
      props: { blocks },
    });

    await wrapper.find('input').setValue('Nonexistent');
    expect(wrapper.text()).toContain('No blocks found');
  });
});
