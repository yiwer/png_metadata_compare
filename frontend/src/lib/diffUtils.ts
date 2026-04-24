// frontend/src/lib/diffUtils.ts
import type { DiffNode, DiffStatus, DiffSummary } from './types';

/** Only nodes with status !== 'unchanged' are included in the map. */
export function buildDiffPathMap(node: DiffNode, map = new Map<string, DiffStatus>()): Map<string, DiffStatus> {
  if (node.status !== 'unchanged') {
    map.set(node.path, node.status);
  }
  for (const child of node.children) {
    buildDiffPathMap(child, map);
  }
  return map;
}

export function totalDiffCount(summary: DiffSummary): number {
  return summary.modified + summary.added + summary.removed + summary.reordered + summary.error;
}
