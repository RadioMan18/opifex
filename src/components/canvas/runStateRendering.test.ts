import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import StepEdge from './StepEdge.vue';
import AgentNode from './AgentNode.vue';

const edgeProps = {
  id: 'e1',
  sourceX: 0,
  sourceY: 0,
  targetX: 100,
  targetY: 100,
  sourcePosition: 'right',
  targetPosition: 'left',
};

describe('StepEdge direction and run state', () => {
  it('points at the plain arrow when nothing ran', () => {
    const wrapper = mount(StepEdge, { props: edgeProps });
    const path = wrapper.find('path');

    expect(path.attributes('marker-end')).toBe('url(#opifex-arrow)');
    expect(path.classes()).not.toContain('step-edge--run-succeeded');
  });

  it('uses the success arrow for a succeeded edge', () => {
    const wrapper = mount(StepEdge, { props: { ...edgeProps, data: { runState: 'succeeded' } } });
    const path = wrapper.find('path');

    expect(path.attributes('marker-end')).toBe('url(#opifex-arrow-success)');
    expect(path.classes()).toContain('step-edge--run-succeeded');
  });

  it('uses the failure arrow for a failed edge', () => {
    const wrapper = mount(StepEdge, { props: { ...edgeProps, data: { runState: 'failed' } } });
    const path = wrapper.find('path');

    expect(path.attributes('marker-end')).toBe('url(#opifex-arrow-failed)');
    expect(path.classes()).toContain('step-edge--run-failed');
  });
});

describe('AgentNode run badge', () => {
  it('paints the run state on the card', () => {
    const wrapper = mount(AgentNode, {
      props: {
        id: 'n1',
        data: {
          label: 'Prompt Weaver',
          block: { kind: 'llm', inputs: [], outputs: [] },
          runState: 'failed',
        },
      },
      global: { stubs: { Handle: true } },
    });

    expect(wrapper.find('.node-card').classes()).toContain('node-card--run-failed');
    expect(wrapper.find('.node-run-badge').text()).toBe('failed');
  });

  it('shows no badge before a run', () => {
    const wrapper = mount(AgentNode, {
      props: {
        id: 'n1',
        data: { label: 'Prompt Weaver', block: { kind: 'llm', inputs: [], outputs: [] } },
      },
      global: { stubs: { Handle: true } },
    });

    expect(wrapper.find('.node-run-badge').exists()).toBe(false);
  });
});
