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
  // 就近分组：丢弃冗余的顶层容器名（如「停靠线路」——线路条目的标签自带线路身份），
  // 深度 1 时直接用该祖先；深度 ≥2 时用去掉首层后的链。
  const topGroup =
    ancestors.length === 0 ? '' :
    ancestors.length === 1 ? ancestors[0] :
    ancestors.slice(1).join(' › ');
  return {
    path: r.path,
    topGroup,
    label: leafLabel,
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
      if (e.status === 'removed') {
        return e.leftValue !== null ? `${name}: ${e.leftValue}（仅左）` : `${name}（仅左侧整项）`;
      }
      if (e.status === 'added') {
        return e.rightValue !== null ? `${name}: ${e.rightValue}（仅右）` : `${name}（仅右侧整项）`;
      }
      return `${name}: ${e.leftValue ?? e.rightValue ?? ''}（错误）`;
    })
    .join('\n');
}
