// frontend/src/lib/diffList.ts
import type { MirrorRow } from './treeModel';
import type { DiffStatus } from './types';

export interface DiffEntry {
  path: string;
  topGroup: string;          // 顶层分组标签（聚类用），顶层字段为 ''
  label: string;             // 展示名（含中间层级，' › ' 连接）
  status: DiffStatus;
  leftValue: string | null;
  rightValue: string | null;
}

export function buildDiffEntries(rows: MirrorRow[]): DiffEntry[] {
  const out: DiffEntry[] = [];
  const walk = (rs: MirrorRow[], ancestors: string[]) => {
    for (const r of rs) {
      if (r.kind === 'leaf') {
        if (r.status === 'unchanged' || r.status === 'reordered') continue;
        out.push(entryFor(r, ancestors, r.label));
        continue;
      }
      const isItemAddRem = r.variant === 'array-item' && (r.status === 'added' || r.status === 'removed');
      if (isItemAddRem) {
        out.push(entryFor(r, ancestors, r.label));
        continue;
      }
      if (r.children) {
        const nextAncestors = r.variant === 'object-root' || !r.label ? ancestors : [...ancestors, r.label];
        walk(r.children, nextAncestors);
      }
    }
  };
  walk(rows, []);
  return out;
}

function entryFor(r: MirrorRow, ancestors: string[], leafLabel: string): DiffEntry {
  const [top, ...rest] = ancestors;
  return {
    path: r.path,
    topGroup: top ?? '',
    label: [...rest, leafLabel].join(' › '),
    status: r.status,
    leftValue: r.leftValue,
    rightValue: r.rightValue,
  };
}

export function buildDiffText(entries: DiffEntry[]): string {
  return entries
    .map((e) => {
      const name = e.topGroup ? `${e.topGroup} › ${e.label}` : e.label;
      if (e.status === 'modified') return `${name}: ${e.leftValue} → ${e.rightValue}`;
      if (e.status === 'removed') return `${name}: ${e.leftValue}（仅左）`;
      if (e.status === 'added') return `${name}: ${e.rightValue}（仅右）`;
      return `${name}: ${e.leftValue ?? e.rightValue ?? ''}（${e.status}）`;
    })
    .join('\n');
}
