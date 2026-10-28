import conformanceVectors from '@/tests/vectors/port_conformance.json';
import type { BlockPort, PortType, ConformanceCheckResult } from '@/types/agent';

interface ConformanceVector {
  source_type: string;
  target_type: string;
  expected_result: boolean;
}

/**
 * CanvasAdapter provides client-side connection evaluation for the agent graph canvas.
 * It caches conformance vectors for 60fps drag feedback during edge creation.
 */
export class CanvasAdapter {
  private static vectorCache = new Map<string, boolean>();

  /**
   * Initialize the cache from shared conformance vectors (must match Rust opifex-core).
   */
  static initialize() {
    this.vectorCache.clear();
    for (const vec of conformanceVectors as ConformanceVector[]) {
      const key = `${vec.source_type}->${vec.target_type}`;
      this.vectorCache.set(key, vec.expected_result);
    }
  }

  /**
   * Check whether a source port can connect to a target port.
   * Returns { allowed: true } if compatible, or { allowed: false, reason } if not.
   */
  static canConnect(sourcePort: BlockPort, targetPort: BlockPort): ConformanceCheckResult {
    // Same direction ports cannot connect (input->input, output->output)
    if (sourcePort.direction === targetPort.direction) {
      return { allowed: false, reason: 'Cannot connect ports of the same direction.' };
    }

    // Only output -> input is valid
    if (sourcePort.direction !== 'output' || targetPort.direction !== 'input') {
      return { allowed: false, reason: 'Only output ports can connect to input ports.' };
    }

    const key = `${sourcePort.type}->${targetPort.type}`;
    const allowed = this.vectorCache.get(key) ?? (sourcePort.type === 'any' || targetPort.type === 'any');

    if (!allowed) {
      return {
        allowed: false,
        reason: `Incompatible types: ${sourcePort.type} cannot flow into ${targetPort.type}.`,
      };
    }

    return { allowed: true };
  }

  /**
   * Get all compatible target types for a given source port type.
   */
  static compatibleTargetTypes(sourceType: PortType): PortType[] {
    const targets: PortType[] = [];
    for (const [key, allowed] of this.vectorCache.entries()) {
      const [src] = key.split('->');
      if (src === sourceType && allowed) {
        const [, tgt] = key.split('->');
        targets.push(tgt as PortType);
      }
    }
    return targets;
  }
}

// Initialize on import
CanvasAdapter.initialize();
