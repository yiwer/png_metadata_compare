import type { BatchCounts, BatchListItem, BatchListItemKind } from '../lib/types';

function summarizeItem(kind: BatchListItemKind, differenceCount: number, message: string | null) {
  if (message) {
    return message;
  }

  switch (kind) {
    case 'different':
      return `${differenceCount} fields changed`;
    case 'identical':
      return 'pixel + metadata match';
    case 'left_only':
      return 'available on the left side only';
    case 'right_only':
      return 'available on the right side only';
    case 'error':
      return 'metadata payload unreadable';
  }
}

export function ResultRail({
  counts,
  items,
  activeId,
  onSelect,
}: {
  counts: BatchCounts | null;
  items: BatchListItem[];
  activeId: string | null;
  onSelect(item: BatchListItem): void;
}) {
  return (
    <aside className="result-rail panel" aria-label="Result rail">
      <div className="panel-heading">
        <span>Result Rail</span>
        <strong>{items.length} items</strong>
      </div>
      <div className="summary-strip">
        <span>{counts?.different ?? 0} different</span>
        <span>{counts?.identical ?? 0} identical</span>
        <span>{counts?.error ?? 0} errors</span>
      </div>
      <div className="result-list">
        {items.length > 0 ? (
          items.map((row) => (
            <button
              key={row.id}
              type="button"
              className={`result-row${activeId === row.id ? ' is-selected' : ''}`}
              onClick={() => onSelect(row)}
            >
              <span className={`status-dot status-dot--${row.kind}`} aria-hidden="true" />
              <span className="result-copy">
                <strong>{row.label}</strong>
                <small>{summarizeItem(row.kind, row.difference_count, row.message)}</small>
              </span>
            </button>
          ))
        ) : (
          <div className="analysis-panel">
            <h2>No results yet</h2>
            <p>Run a compare to populate the result rail.</p>
          </div>
        )}
      </div>
    </aside>
  );
}
