// frontend/src/components/DirectoryList.tsx
import type { ActiveFilter } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary } from '../lib/types';

const KIND_TO_DOT: Record<BatchListItemKind, string> = {
  different: 'mod',
  identical: 'eq',
  left_only: 'rem',
  right_only: 'add',
  error: 'err',
};

const KIND_BADGE: Record<BatchListItemKind, (count: number) => string> = {
  different: (n) => `${n} 处不同`,
  identical: () => '一致',
  left_only: () => '仅左侧',
  right_only: () => '仅右侧',
  error: () => '错误',
};

const CHIPS: { id: ActiveFilter; label: (counts: DirectorySummary['counts']) => string; chipClass?: string }[] = [
  { id: 'all', label: (c) => `全部 ${c.identical + c.different + c.left_only + c.right_only + c.error}` },
  { id: 'different', label: (c) => `不一致 ${c.different}`, chipClass: 'dirlist__chip--mod' },
  { id: 'left_only', label: (c) => `仅左 ${c.left_only}`, chipClass: 'dirlist__chip--rem' },
  { id: 'right_only', label: (c) => `仅右 ${c.right_only}`, chipClass: 'dirlist__chip--add' },
  { id: 'identical', label: (c) => `一致 ${c.identical}` },
  { id: 'error', label: (c) => `错误 ${c.error}`, chipClass: 'dirlist__chip--err' },
];

export function DirectoryList({
  summary,
  filteredItems,
  activeFilter,
  onFilter,
  onSelect,
}: {
  summary: DirectorySummary;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  onFilter(f: ActiveFilter): void;
  onSelect(item: BatchListItem): void;
}) {
  const c = summary.counts;
  const total = c.identical + c.different + c.left_only + c.right_only + c.error;

  return (
    <div className="dirlist">
      <div className="dirlist__stats">
        <div className="dirlist__stat dirlist__stat--mod">
          <span className="dirlist__stat-num">{c.different}</span>不一致
        </div>
        <div className="dirlist__stat dirlist__stat--rem">
          <span className="dirlist__stat-num">{c.left_only}</span>仅左
        </div>
        <div className="dirlist__stat dirlist__stat--add">
          <span className="dirlist__stat-num">{c.right_only}</span>仅右
        </div>
        <div className="dirlist__stat dirlist__stat--eq">
          <span className="dirlist__stat-num">{c.identical}</span>一致
        </div>
        <div className="dirlist__stat dirlist__stat--total">
          <span className="dirlist__stat-num">{total}</span>总计
        </div>
      </div>

      <div className="dirlist__chips">
        {CHIPS.map((chip) => (
          <button
            key={chip.id}
            type="button"
            className={`dirlist__chip ${chip.chipClass ?? ''}`}
            data-active={activeFilter === chip.id}
            onClick={() => onFilter(chip.id)}
          >
            {chip.label(c)}
          </button>
        ))}
      </div>

      <div className="dirlist__rows">
        {filteredItems.map((item) => (
          <div
            key={item.id}
            className={`dirlist__row${item.kind === 'identical' ? ' dirlist__row--eq' : ''}`}
            onClick={() => onSelect(item)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(item); }
            }}
          >
            <span className={`dirlist__dot dirlist__dot--${KIND_TO_DOT[item.kind]}`} />
            <span className="dirlist__name">{item.label}</span>
            <span className={`badge badge--${badgeKindFor(item.kind)}`}>{KIND_BADGE[item.kind](item.difference_count)}</span>
            <span className="dirlist__chev">›</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function badgeKindFor(k: BatchListItemKind): 'mod' | 'add' | 'rem' | 'err' | 'neu' {
  switch (k) {
    case 'different': return 'mod';
    case 'left_only': return 'rem';
    case 'right_only': return 'add';
    case 'error': return 'err';
    default: return 'neu';
  }
}
