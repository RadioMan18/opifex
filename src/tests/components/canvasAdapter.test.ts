import { describe, it, expect } from 'vitest';
import { CanvasAdapter } from '@/services/canvasAdapter';

describe('CanvasAdapter', () => {
  it('initializes conformance vectors from shared JSON', () => {
    // CanvasAdapter.initialize() runs on import, so cache should be populated
    const result = CanvasAdapter.canConnect(
      { id: 'out1', name: 'out', type: 'string', direction: 'output', required: false },
      { id: 'in1', name: 'in', type: 'string', direction: 'input', required: true }
    );
    expect(result.allowed).toBe(true);
  });

  it('rejects same-direction connections', () => {
    const result = CanvasAdapter.canConnect(
      { id: 'out1', name: 'out', type: 'string', direction: 'output', required: false },
      { id: 'in1', name: 'in', type: 'string', direction: 'output', required: false }
    );
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('same direction');
  });

  it('rejects input-to-output connections', () => {
    const result = CanvasAdapter.canConnect(
      { id: 'in1', name: 'in', type: 'string', direction: 'input', required: false },
      { id: 'out1', name: 'out', type: 'string', direction: 'output', required: false }
    );
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('output ports can connect to input ports');
  });

  it('rejects incompatible types', () => {
    const result = CanvasAdapter.canConnect(
      { id: 'out1', name: 'out', type: 'string', direction: 'output', required: false },
      { id: 'in1', name: 'in', type: 'number', direction: 'input', required: true }
    );
    expect(result.allowed).toBe(false);
    expect(result.reason).toContain('Incompatible types');
  });

  it('allows any type to connect to any type', () => {
    const result = CanvasAdapter.canConnect(
      { id: 'out1', name: 'out', type: 'any', direction: 'output', required: false },
      { id: 'in1', name: 'in', type: 'number', direction: 'input', required: true }
    );
    expect(result.allowed).toBe(true);
  });

  it('allows any target from any source', () => {
    const result = CanvasAdapter.canConnect(
      { id: 'out1', name: 'out', type: 'string', direction: 'output', required: false },
      { id: 'in1', name: 'in', type: 'any', direction: 'input', required: true }
    );
    expect(result.allowed).toBe(true);
  });

  it('returns compatible target types', () => {
    const targets = CanvasAdapter.compatibleTargetTypes('string');
    expect(targets).toContain('string');
    expect(targets).not.toContain('number');
  });
});
