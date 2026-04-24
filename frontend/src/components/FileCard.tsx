// frontend/src/components/FileCard.tsx
import type { BatchListItem, BatchListItemKind } from '../lib/types';

const STATUS_LABEL: Record<BatchListItemKind, string> = {
  different: '差异',
  identical: '相同',
  left_only: '仅左侧',
  right_only: '仅右侧',
  error: '错误',
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
  if (item.kind === 'error') return item.message ?? '解析失败';
  if (item.kind === 'identical') return '无变更';
  if (item.kind === 'left_only') return '右侧目录中不存在';
  if (item.kind === 'right_only') return '左侧目录中不存在';
  if (item.difference_count > 0)
    return `${item.difference_count} 处变更`;
  return '存在差异';
}
