// frontend/src/components/Sidebar.tsx
import { memo, useEffect, useRef, useState } from 'react';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import type { ActiveFilter, SortKey } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary, ScanProgress } from '../lib/types';

const KIND_DOT: Record<BatchListItemKind, string> = {
  different: 'mod', identical: 'eq', left_only: 'rem', right_only: 'add', error: 'err',
};
const KIND_TAG: Record<BatchListItemKind, (n: number) => string> = {
  different: (n) => String(n), identical: () => '一致', left_only: () => '仅左', right_only: () => '仅右', error: () => '错误',
};
const CHIPS: { id: ActiveFilter; label: (c: DirectorySummary['counts']) => string }[] = [
  { id: 'different', label: (c) => `不一致 ${c.different}` },
  { id: 'left_only', label: (c) => `仅左 ${c.left_only}` },
  { id: 'right_only', label: (c) => `仅右 ${c.right_only}` },
  { id: 'identical', label: (c) => `一致 ${c.identical}` },
  { id: 'error', label: (c) => `错误 ${c.error}` },
  { id: 'all', label: (c) => `全部 ${c.identical + c.different + c.left_only + c.right_only + c.error}` },
];

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function Sidebar({
  leftDir, rightDir, summary, filteredItems, activeFilter, searchQuery, sortKey,
  selectedItemId, isLoading, scanProgress,
  onFilter, onSearch, onSort, onSelect, onPickLeft, onPickRight, onCancelScan,
}: {
  leftDir: string;
  rightDir: string;
  summary: DirectorySummary | null;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  searchQuery: string;
  sortKey: SortKey;
  selectedItemId: string | null;
  isLoading: boolean;
  scanProgress: ScanProgress | null;
  onFilter(f: ActiveFilter): void;
  onSearch(q: string): void;
  onSort(k: SortKey): void;
  onSelect(item: BatchListItem): void;
  onPickLeft(): void;
  onPickRight(): void;
  onCancelScan(): void;
}) {
  const searchRef = useRef<HTMLInputElement>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; item: BatchListItem } | null>(null);

  // Ctrl+F 经全局事件聚焦搜索框
  useEffect(() => {
    const onFocus = () => searchRef.current?.focus();
    document.addEventListener('wb:focusSearch', onFocus);
    return () => document.removeEventListener('wb:focusSearch', onFocus);
  }, []);

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') setMenu(null); };
    window.addEventListener('click', close);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('click', close);
      window.removeEventListener('keydown', onKey);
    };
  }, [menu]);

  const counts = summary?.counts ?? null;
  const totalPairs = counts
    ? counts.identical + counts.different + counts.left_only + counts.right_only + counts.error
    : 0;
  const diffPos = (() => {
    if (!summary || !selectedItemId) return null;
    const diffs = summary.items.filter((i) => i.kind === 'different');
    const idx = diffs.findIndex((i) => i.id === selectedItemId);
    return idx >= 0 ? `${idx + 1} / ${diffs.length}` : null;
  })();

  return (
    <aside className="sidebar">
      <div className="sidebar__slots">
        <DirChip side="左" path={leftDir} onPick={onPickLeft} />
        <DirChip side="右" path={rightDir} onPick={onPickRight} />
      </div>

      <div className="sidebar__search">
        <input ref={searchRef} type="text" placeholder="搜索文件名…" value={searchQuery}
          onChange={(e) => onSearch(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Escape') { onSearch(''); (e.target as HTMLInputElement).blur(); } }} />
      </div>

      {counts && (
        <div className="sidebar__chips">
          {CHIPS.map((chip) => (
            <button key={chip.id} type="button" className="sidebar__chip"
              data-active={activeFilter === chip.id} onClick={() => onFilter(chip.id)}>
              {chip.label(counts)}
            </button>
          ))}
          <button type="button" className="sidebar__sort" title="切换排序"
            onClick={() => onSort(sortKey === 'diff-desc' ? 'name-asc' : 'diff-desc')}>
            {sortKey === 'diff-desc' ? '↓差异' : 'A-Z'}
          </button>
        </div>
      )}

      {isLoading && (
        <div className="sidebar__progress" role="status" aria-live="polite">
          <span>
            {scanProgress?.stage === 'comparing' && scanProgress.total > 0
              ? `已比对 ${scanProgress.done} / ${scanProgress.total}`
              : '正在扫描目录…'}
          </span>
          <button type="button" className="sidebar__cancel" onClick={onCancelScan}>取消</button>
          <div className="sidebar__progress-track">
            <div className="sidebar__progress-fill" style={
              scanProgress?.stage === 'comparing' && scanProgress.total > 0
                ? { width: `${Math.round((scanProgress.done / scanProgress.total) * 100)}%` }
                : undefined
            } />
          </div>
        </div>
      )}

      <div className="sidebar__rows">
        {filteredItems.length === 0 && !isLoading && (
          <div className="sidebar__empty">
            {searchQuery
              ? <><span>无匹配</span> <button type="button" onClick={() => onSearch('')}>清空搜索</button></>
              : counts && counts.different === 0 && activeFilter === 'all' && totalPairs > 0 && summary?.items.every((i) => i.kind === 'identical')
                ? '两侧完全一致'
                : '无条目'}
          </div>
        )}
        {filteredItems.map((item) => (
          <Row key={item.id} item={item} selected={item.id === selectedItemId}
            onSelect={onSelect}
            onMenu={(e) => { e.preventDefault(); setMenu({ x: e.clientX, y: e.clientY, item }); }} />
        ))}
      </div>

      <div className="sidebar__foot">
        {summary ? `${totalPairs} 对${diffPos ? ` · ${diffPos} 不一致` : ''}` : '未扫描'}
      </div>

      {menu && (
        <div className="sidebar__menu" style={{ left: Math.min(menu.x, window.innerWidth - 180), top: Math.min(menu.y, window.innerHeight - 72) }}>
          <button type="button" onClick={() => {
            void navigator.clipboard?.writeText([menu.item.left_path, menu.item.right_path].filter(Boolean).join('\n'));
          }}>复制路径</button>
          <button type="button" onClick={() => {
            const p = menu.item.left_path ?? menu.item.right_path ?? menu.item.label;
            void revealItemInDir(p).catch(() => { /* 文件可能已被移动/删除：静默 */ });
          }}>在资源管理器中显示</button>
        </div>
      )}
    </aside>
  );
}

function DirChip({ side, path, onPick }: { side: string; path: string; onPick(): void }) {
  return (
    <div className="sidebar__slot">
      <span className="sidebar__slot-side">{side}</span>
      <button type="button" className="sidebar__slot-path" title={path || '未选择'} onClick={onPick}>
        {path ? `…\\${basename(path)}` : '选择目录'}
      </button>
    </div>
  );
}

const Row = memo(function Row({
  item, selected, onSelect, onMenu,
}: {
  item: BatchListItem; selected: boolean;
  onSelect(item: BatchListItem): void;
  onMenu(e: React.MouseEvent): void;
}) {
  return (
    <div className="sidebar__row" data-selected={selected ? 'true' : undefined} role="button" tabIndex={0}
      title={[item.left_path, item.right_path].filter(Boolean).join('\n')}
      onClick={() => onSelect(item)} onContextMenu={onMenu}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(item); } }}>
      <span className={`sidebar__dot sidebar__dot--${KIND_DOT[item.kind]}`} />
      <span className="sidebar__name">{item.label}</span>
      <span className={`sidebar__tag sidebar__tag--${KIND_DOT[item.kind]}`}>{KIND_TAG[item.kind](item.difference_count)}</span>
    </div>
  );
});
