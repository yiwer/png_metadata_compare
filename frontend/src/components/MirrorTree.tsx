// frontend/src/components/MirrorTree.tsx
import { useEffect, useMemo, useState } from 'react';
import { GroupHead } from './GroupHead';
import { buildMirrorRows, hasDiffDeep } from '../lib/treeModel';
import type { MirrorRow } from '../lib/treeModel';
import type { DiffNode, DiffStatus, JsonValue } from '../lib/types';

const STATUS_CLASS: Record<DiffStatus, string> = {
  unchanged: '',
  modified: 'kv--mod',
  added: 'kv--add',
  removed: 'kv--rem',
  reordered: 'kv--reord',
  error: 'kv--err',
};

export function MirrorTree({
  left,
  right,
  diffRoot,
  highlight,
  onlyDiff,
  leftLabel,
  rightLabel,
}: {
  left: JsonValue | null;
  right: JsonValue | null;
  diffRoot: DiffNode | null;
  highlight: boolean;
  onlyDiff: boolean;
  leftLabel: string;
  rightLabel: string;
}) {
  const rows = useMemo(() => buildMirrorRows(left, right, diffRoot), [left, right, diffRoot]);
  const [closed, setClosed] = useState<Set<string>>(() => collectDefaultClosed(rows));

  // Reset folding state whenever the underlying data changes
  useEffect(() => {
    setClosed(collectDefaultClosed(rows));
  }, [rows]);

  const toggle = (path: string) =>
    setClosed((cur) => {
      const next = new Set(cur);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  // When onlyDiff is on, force-open any subtree that contains a diff.
  const effectiveClosed = useMemo(() => {
    if (!onlyDiff) return closed;
    const next = new Set(closed);
    forceOpenIfHasDiff(rows, next);
    return next;
  }, [onlyDiff, closed, rows]);

  return (
    <div className="mirror-grid">
      <PaneHeader label={leftLabel} side="left" />
      <PaneHeader label={rightLabel} side="right" />
      <Pane
        rows={rows}
        side="left"
        closed={effectiveClosed}
        toggle={toggle}
        highlight={highlight}
        onlyDiff={onlyDiff}
      />
      <Pane
        rows={rows}
        side="right"
        closed={effectiveClosed}
        toggle={toggle}
        highlight={highlight}
        onlyDiff={onlyDiff}
      />
    </div>
  );
}

function PaneHeader({ label, side }: { label: string; side: 'left' | 'right' }) {
  return (
    <div className={`mirror-pane mirror-pane--${side}`} style={{ borderBottom: '1px solid var(--border-subtle)', padding: '6px 12px', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
      {side === 'left' ? '左 · ' : '右 · '}{label}
    </div>
  );
}

function Pane({
  rows,
  side,
  closed,
  toggle,
  highlight,
  onlyDiff,
}: {
  rows: MirrorRow[];
  side: 'left' | 'right';
  closed: Set<string>;
  toggle: (p: string) => void;
  highlight: boolean;
  onlyDiff: boolean;
}) {
  return (
    <div className={`mirror-pane mirror-pane--${side}`}>
      {rows.map((row) => (
        <RowView
          key={row.path || 'root'}
          row={row}
          side={side}
          level={0}
          closed={closed}
          toggle={toggle}
          highlight={highlight}
          onlyDiff={onlyDiff}
        />
      ))}
    </div>
  );
}

function RowView({
  row,
  side,
  level,
  closed,
  toggle,
  highlight,
  onlyDiff,
}: {
  row: MirrorRow;
  side: 'left' | 'right';
  level: number;
  closed: Set<string>;
  toggle: (p: string) => void;
  highlight: boolean;
  onlyDiff: boolean;
}) {
  if (onlyDiff && !hasDiffDeep(row)) return null;

  if (row.kind === 'leaf') {
    return <MirrorLeaf row={row} side={side} highlight={highlight} />;
  }

  // group
  if (row.variant === 'object-root') {
    return (
      <>
        {row.children?.map((c) => (
          <RowView
            key={c.path}
            row={c}
            side={side}
            level={level}
            closed={closed}
            toggle={toggle}
            highlight={highlight}
            onlyDiff={onlyDiff}
          />
        ))}
      </>
    );
  }

  const isOpen = !closed.has(row.path);
  return (
    <>
      <GroupHead
        label={row.label}
        count={row.variant === 'array' ? row.count : undefined}
        level={level}
        open={isOpen}
        onToggle={() => toggle(row.path)}
      />
      {isOpen && (
        <div className="tree__nested">
          {row.children?.map((c) => (
            <RowView
              key={c.path}
              row={c}
              side={side}
              level={level + 1}
              closed={closed}
              toggle={toggle}
              highlight={highlight}
              onlyDiff={onlyDiff}
            />
          ))}
        </div>
      )}
    </>
  );
}

function MirrorLeaf({ row, side, highlight }: { row: MirrorRow; side: 'left' | 'right'; highlight: boolean }) {
  const value = side === 'left' ? row.leftValue : row.rightValue;
  const otherValue = side === 'left' ? row.rightValue : row.leftValue;

  // placeholder when this side absent but the other side exists
  if (value === null && otherValue !== null) {
    return (
      <div className="kv kv--placeholder">
        <span className="kv__key">— — —</span>
        <span className="kv__val">仅另一侧存在</span>
      </div>
    );
  }
  if (value === null && otherValue === null) {
    // both null — render as a normal em-dash row
    return (
      <div className="kv">
        <span className="kv__key">{row.label}</span>
        <span className="kv__val">—</span>
      </div>
    );
  }

  const cls = highlight ? STATUS_CLASS[row.status] : '';
  return (
    <div className={`kv ${cls}`.trim()}>
      <span className="kv__key">{row.label}</span>
      <span className="kv__val">{value}</span>
    </div>
  );
}

function collectDefaultClosed(rows: MirrorRow[], into = new Set<string>()): Set<string> {
  for (const r of rows) {
    if (r.kind === 'group' && !r.defaultOpen && r.path) into.add(r.path);
    if (r.children) collectDefaultClosed(r.children, into);
  }
  return into;
}

function forceOpenIfHasDiff(rows: MirrorRow[], closed: Set<string>): void {
  for (const r of rows) {
    if (r.kind === 'group' && hasDiffDeep(r)) closed.delete(r.path);
    if (r.children) forceOpenIfHasDiff(r.children, closed);
  }
}
