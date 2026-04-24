// frontend/src/components/FileCard.tsx
import type { BatchListItem, BatchListItemKind } from '../lib/types';

const STATUS_LABEL: Record<BatchListItemKind, string> = {
  different: 'Different',
  identical: 'Identical',
  left_only: 'Left Only',
  right_only: 'Right Only',
  error: 'Error',
};

export function FileCard({
  item,
  style,
  disabled,
  onClick,
}: {
  item: BatchListItem;
  style?: React.CSSProperties;
  disabled?: boolean;
  onClick(): void;
}) {
  const label = STATUS_LABEL[item.kind];
  const metaText = cardMeta(item);

  return (
    <button type="button" className={`file-card file-card--${item.kind}${disabled ? ' file-card--disabled' : ''}`} style={style} disabled={disabled} onClick={onClick}>
      <div className="card-header">
        <span className={`status-dot status-dot--${item.kind}`} />
        {label}
      </div>
      <div className="card-body">
        <div className="card-name">{item.label}</div>
        <div className={`card-meta${item.kind === 'error' ? ' card-meta--error' : ''}`}>{metaText}</div>
      </div>
    </button>
  );
}

function cardMeta(item: BatchListItem): string {
  if (item.kind === 'error') return item.message ?? 'parse failed';
  if (item.kind === 'identical') return 'no changes';
  if (item.kind === 'left_only') return 'not in right dir';
  if (item.kind === 'right_only') return 'not in left dir';
  if (item.difference_count > 0) return `${item.difference_count} change${item.difference_count !== 1 ? 's' : ''}`;
  return 'different';
}
