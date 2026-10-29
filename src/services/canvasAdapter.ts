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
   * Check whether two nodes can be connected (node-to-node, Gumloop model).
   * Mapping (which input uses which output) happens inside the target node inspector.
   */
  static canConnectNodes(
    sourceNode: { block: { outputs: BlockPort[] } },
    targetNode: { block: { inputs: BlockPort[] } }
  ): ConformanceCheckResult {
    // Source must have at least one output
    if (!sourceNode.block.outputs || sourceNode.block.outputs.length === 0) {
      return { allowed: false, reason: 'Source node has no outputs.' };
    }

    // Target must have at least one input
    if (!targetNode.block.inputs || targetNode.block.inputs.length === 0) {
      return { allowed: false, reason: 'Target node has no inputs.' };
    }

    // Check type compatibility between at least one output and one input
    const hasCompatiblePair = sourceNode.block.outputs.some((out) =>
      targetNode.block.inputs.some((inp) => {
        if (out.type === 'any' || inp.type === 'any') return true;
        return out.type === inp.type;
      })
    );

    if (!hasCompatiblePair) {
      return {
        allowed: false,
        reason: 'No compatible output/input type pair between nodes.',
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
