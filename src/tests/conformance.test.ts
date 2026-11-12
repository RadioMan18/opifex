import { describe, expect, it } from 'vitest';
import portConformanceVectors from './vectors/port_conformance.json';

interface PortConformanceVector {
  source_type: string;
  target_type: string;
  expected_result: boolean;
}

function canConnect(sourceType: string, targetType: string): boolean {
  if (sourceType === 'any' || targetType === 'any') {
    return true;
  }
  return sourceType === targetType;
}

describe('Shared Conformance Vectors (TypeScript)', () => {
  it('matches Rust conformance vectors 100%', () => {
    const vectors: PortConformanceVector[] = portConformanceVectors as PortConformanceVector[];

    for (const vec of vectors) {
      const result = canConnect(vec.source_type, vec.target_type);
      expect(result).toBe(vec.expected_result);
    }
  });

  it('counts all 20 vectors', () => {
    const vectors: PortConformanceVector[] = portConformanceVectors as PortConformanceVector[];
    expect(vectors.length).toBe(20);
  });
});
