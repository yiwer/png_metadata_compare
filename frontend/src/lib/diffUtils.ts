// frontend/src/lib/diffUtils.ts
import type { DiffNode, DiffStatus } from './types';

export function buildDiffPathMap(node: DiffNode, map = new Map<string, DiffStatus>()): Map<string, DiffStatus> {
  if (node.status !== 'unchanged') {
    map.set(node.path, node.status);
  }
  for (const child of node.children) {
    buildDiffPathMap(child, map);
  }
  return map;
}

export function totalDiffCount(summary: { modified: number; added: number; removed: number; reordered: number; error: number }): number {
  return summary.modified + summary.added + summary.removed + summary.reordered + summary.error;
}
