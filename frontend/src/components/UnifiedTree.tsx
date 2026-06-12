// frontend/src/components/UnifiedTree.tsx
import { useEffect, useMemo, useRef, useState } from 'react';
import { GroupHead } from './GroupHead';
import { hasDiffDeep } from '../lib/treeModel';
import type { MirrorRow } from '../lib/treeModel';
import type { DiffStatus, Side } from '../lib/types';

export interface FocusRequest {
  path: string;
  seq: number;   // 单调递增，保证同一路径可重复触发
}

const ROW_STATUS: Partial<Record<DiffStatus, string>> = {
  modified: 'utree__row--modified',
  added: 'utree__row--added',
  removed: 'utree__row--removed',
  reordered: 'utree__row--reordered',
  error: 'utree__row--error',
};

export function UnifiedTree({
  rows, solo, highlight, onlyDiff, leftLabel, rightLabel, focusRequest,
}: {
  rows: MirrorRow[];
  solo: Side | null;
  highlight: boolean;
  onlyDiff: boolean;
  leftLabel: string;
  rightLabel: string;
  focusRequest: FocusRequest | null;
}) {
  // solo 数据没有 diff，可达的 onlyDiff 全局开关在此中和，否则整树为空
  const effOnlyDiff = solo ? false : onlyDiff;

  const [closed, setClosed] = useState<Set<string>>(() => collectDefaultClosed(rows));
  const bodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => { setClosed(collectDefaultClosed(rows)); }, [rows]);

  const handledSeqRef = useRef(0);

  // 展开祖先（一次提交）
  useEffect(() => {
    if (!focusRequest) return;
    setClosed((cur) => openAncestors(rows, focusRequest.path, cur));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focusRequest]);

  // 每次提交后尝试定位：祖先展开的那次提交必然命中（提交驱动，无 rAF 竞态）
  useEffect(() => {
    if (!focusRequest || handledSeqRef.current === focusRequest.seq) return;
    const el = bodyRef.current?.querySelector<HTMLElement>(
      `[data-path="${CSS.escape(focusRequest.path)}"]`,
    );
    if (!el) return;
    handledSeqRef.current = focusRequest.seq;
    el.scrollIntoView({ block: 'center' });
    el.classList.remove('utree__row--flash');
    void el.offsetWidth; // 重启 CSS 动画
    el.classList.add('utree__row--flash');
  });

  const toggle = (path: string) =>
    setClosed((cur) => {
      const next = new Set(cur);
      if (next.has(path)) next.delete(path); else next.add(path);
      return next;
    });

  const effectiveClosed = useMemo(
    () => (effOnlyDiff ? withDiffGroupsOpen(rows, closed) : closed),
    [effOnlyDiff, rows, closed],
  );

  return (
    <div className="utree">
      <div className="utree__cols utree__head" data-solo={solo ?? undefined}>
        <span className="utree__col-label">字段</span>
        {(solo === null || solo === 'left') && <span className="utree__col-side" title={leftLabel}>左 · <span>{leftLabel}</span></span>}
        {(solo === null || solo === 'right') && <span className="utree__col-side" title={rightLabel}>右 · <span>{rightLabel}</span></span>}
      </div>
      <div className="utree__body" ref={bodyRef}>
        {rows.map((row) => (
          <RowView key={row.path || 'root'} row={row} level={0} closed={effectiveClosed}
            toggle={toggle} highlight={highlight} onlyDiff={effOnlyDiff} solo={solo} />
        ))}
      </div>
    </div>
  );
}

function RowView({
  row, level, closed, toggle, highlight, onlyDiff, solo,
}: {
  row: MirrorRow; level: number; closed: Set<string>;
  toggle: (p: string) => void; highlight: boolean; onlyDiff: boolean; solo: Side | null;
}) {
  if (onlyDiff && !hasDiffDeep(row)) return null;

  if (row.kind === 'leaf') {
    // In solo mode, skip leaves whose solo-side raw value is absent (schema-padded placeholders).
    if (solo === 'left' && row.leftRaw === undefined) return null;
    if (solo === 'right' && row.rightRaw === undefined) return null;
    return <Leaf row={row} highlight={highlight} solo={solo} />;
  }

  if (row.variant === 'object-root') {
    return (
      <>
        {row.children?.map((c) => (
          <RowView key={c.path} row={c} level={level} closed={closed} toggle={toggle}
            highlight={highlight} onlyDiff={onlyDiff} solo={solo} />
        ))}
      </>
    );
  }

  // 仅一侧存在的数组项：solo 不染色；非 solo 高亮时整组染色
  const headStatus = solo ? 'unchanged' : row.status;
  const isOpen = !closed.has(row.path);
  const raw = row.leftRaw ?? row.rightRaw;
  return (
    <>
      <GroupHead
        label={row.label}
        count={row.variant === 'array' ? row.count : undefined}
        level={level}
        dataPath={row.path}
        open={isOpen}
        onToggle={() => toggle(row.path)}
        status={headStatus}
        highlight={highlight}
        trailing={
          raw !== undefined ? (
            <button type="button" className="utree__copy" aria-label="复制 JSON 子树"
              onClick={(e) => { e.stopPropagation(); void navigator.clipboard?.writeText(JSON.stringify(raw, null, 2)); }}
            >⧉</button>
          ) : undefined
        }
      />
      {isOpen && (
        <div className="utree__nested">
          {row.children?.map((c) => (
            <RowView key={c.path} row={c} level={level + 1} closed={closed} toggle={toggle}
              highlight={highlight} onlyDiff={onlyDiff} solo={solo} />
          ))}
        </div>
      )}
    </>
  );
}

function Leaf({ row, highlight, solo }: { row: MirrorRow; highlight: boolean; solo: Side | null }) {
  const statusCls = !solo && highlight ? ROW_STATUS[row.status] ?? '' : '';
  const showLeft = solo === null || solo === 'left';
  const showRight = solo === null || solo === 'right';
  return (
    <div className={`utree__cols utree__row ${statusCls}`.trim()} data-path={row.path} data-solo={solo ?? undefined}>
      <span className="utree__key">
        {row.label}
        {row.isUnknown && <span className="utree__unknown">未识别</span>}
      </span>
      {showLeft && (
        <span className={`utree__val${statusCls && row.status === 'modified' ? ' utree__val--old' : ''}`}>
          {row.leftValue}
          <button type="button" className="utree__copy" aria-label="复制左值"
            onClick={() => void navigator.clipboard?.writeText(row.leftValue ?? '')}>⧉</button>
        </span>
      )}
      {showRight && (
        <span className={`utree__val${statusCls && row.status === 'modified' ? ' utree__val--new' : ''}`}>
          {row.rightValue}
          <button type="button" className="utree__copy" aria-label="复制右值"
            onClick={() => void navigator.clipboard?.writeText(row.rightValue ?? '')}>⧉</button>
        </span>
      )}
    </div>
  );
}

function collectDefaultClosed(rows: MirrorRow[], into = new Set<string>()): Set<string> {
  for (const r of rows) {
    // array-items are never initially closed — only top-level arrays are collapsed by default.
    if (r.kind === 'group' && !r.defaultOpen && r.path && r.variant !== 'array-item') into.add(r.path);
    if (r.children) collectDefaultClosed(r.children, into);
  }
  return into;
}

function withDiffGroupsOpen(rows: MirrorRow[], closed: Set<string>): Set<string> {
  const next = new Set(closed);
  const walk = (rs: MirrorRow[]) => {
    for (const r of rs) {
      if (r.kind === 'group' && hasDiffDeep(r)) next.delete(r.path);
      if (r.children) walk(r.children);
    }
  };
  walk(rows);
  return next;
}

/** 打开 target 的所有祖先分组（段前缀判定：后随 '.' 或 '['）。 */
function openAncestors(rows: MirrorRow[], target: string, closed: Set<string>): Set<string> {
  const next = new Set(closed);
  const walk = (rs: MirrorRow[]) => {
    for (const r of rs) {
      if (r.kind === 'group' && r.path && isAncestorPath(r.path, target)) next.delete(r.path);
      if (r.children) walk(r.children);
    }
  };
  walk(rows);
  return next;
}

function isAncestorPath(parent: string, child: string): boolean {
  if (!child.startsWith(parent)) return false;
  const rest = child.slice(parent.length);
  return rest === '' || rest.startsWith('.') || rest.startsWith('[');
}
