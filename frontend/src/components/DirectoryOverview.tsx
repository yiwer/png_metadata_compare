// frontend/src/components/DirectoryOverview.tsx
import { FileCard } from './FileCard';
import { EmptyState } from './EmptyState';
import type { ActiveFilter } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary } from '../lib/types';

const FILTERS: { id: ActiveFilter; label: string }[] = [
  { id: 'all', label: '全部' },
  { id: 'different', label: '差异' },
  { id: 'identical', label: '相同' },
  { id: 'left_only', label: '仅左侧' },
  { id: 'right_only', label: '仅右侧' },
  { id: 'error', label: '错误' },
];

export function DirectoryOverview({
  leftInput,
  rightInput,
  directorySummary,
  filteredItems,
  activeFilter,
  isLoading,
  error,
  onLeftInput,
  onRightInput,
  onScan,
  onPickLeft,
  onPickRight,
  onFilter,
  onSelectItem,
}: {
  leftInput: string;
  rightInput: string;
  directorySummary: DirectorySummary | null;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  isLoading: boolean;
  error: string | null;
  onLeftInput(v: string): void;
  onRightInput(v: string): void;
  onScan(): void;
  onPickLeft(): void;
  onPickRight(): void;
  onFilter(f: ActiveFilter): void;
  onSelectItem(item: BatchListItem): void;
}) {
  const counts = directorySummary?.counts;

  return (
    <>
      <div className="toolbar">
        <div className="path-group">
          <span className="path-label">左目录</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={leftInput}
              onChange={(e) => onLeftInput(e.target.value)}
              placeholder="左侧目录路径…"
            />
            <button type="button" className="choose-btn" onClick={onPickLeft}>选择</button>
          </div>
        </div>
        <div className="cta-wrap">
          <div className="cta-outer">
            <button
              type="button"
              className="cta-btn"
              disabled={isLoading || !leftInput || !rightInput}
              onClick={onScan}
            >
              {isLoading ? '扫描中…' : '扫描'}
            </button>
          </div>
        </div>
        <div className="path-group">
          <span className="path-label">右目录</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={rightInput}
              onChange={(e) => onRightInput(e.target.value)}
              placeholder="右侧目录路径…"
            />
            <button type="button" className="choose-btn" onClick={onPickRight}>选择</button>
          </div>
        </div>
      </div>

      {error && <div className="status-banner status-banner--error">{error}</div>}

      {counts && (
        <div className="stats-bar">
          {counts.different > 0 && <StatChip kind="different" count={counts.different} />}
          {counts.identical > 0 && <StatChip kind="identical" count={counts.identical} />}
          {counts.left_only > 0 && <StatChip kind="left_only" count={counts.left_only} />}
          {counts.right_only > 0 && <StatChip kind="right_only" count={counts.right_only} />}
          {counts.error > 0 && <StatChip kind="error" count={counts.error} />}
        </div>
      )}

      {directorySummary && (
        <div className="filter-bar">
          {FILTERS.map((f) => (
            <button
              key={f.id}
              type="button"
              className={`filter-btn${activeFilter === f.id ? ' filter-btn--active' : ''}`}
              onClick={() => onFilter(f.id)}
            >
              {f.label}
            </button>
          ))}
          <span className="filter-count">{filteredItems.length} 个文件</span>
        </div>
      )}

      <div className="card-grid">
        {isLoading && !directorySummary && (
          <EmptyState title="扫描中…" body="正在查找并对比 PNG 文件，请稍候…" />
        )}
        {!directorySummary && !isLoading && (
          <EmptyState
            title="选择两个目录并扫描"
            body="选择左右目录路径，点击扫描后结果显示在此处。"
          />
        )}
        {filteredItems.length === 0 && directorySummary && !isLoading && (
          <EmptyState title="无结果" body="没有文件与当前筛选条件匹配。" />
        )}
        {filteredItems.map((item, index) => (
          <FileCard
            key={item.id}
            item={item}
            style={{ animationDelay: `${Math.min(index * 30, 300)}ms` }}
            disabled={!item.left_path || !item.right_path}
            onClick={() => onSelectItem(item)}
          />
        ))}
      </div>
    </>
  );
}

const KIND_LABEL: Record<BatchListItemKind, string> = {
  different: 'different',
  identical: 'identical',
  left_only: 'left-only',
  right_only: 'right-only',
  error: 'error',
};

function StatChip({ kind, count }: { kind: BatchListItemKind; count: number }) {
  return (
    <div className="stat-chip">
      <span className={`status-dot status-dot--${kind}`} />
      {count} {KIND_LABEL[kind]}
    </div>
  );
}
