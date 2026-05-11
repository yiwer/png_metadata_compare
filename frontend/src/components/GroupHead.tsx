// frontend/src/components/GroupHead.tsx
import type { ReactNode } from 'react';
import type { DiffStatus } from '../lib/types';

const STATUS_CLASS: Partial<Record<DiffStatus, string>> = {
  added: 'group-head--add',
  removed: 'group-head--rem',
  modified: 'group-head--mod',
  reordered: 'group-head--reord',
  error: 'group-head--err',
};

export function GroupHead({
  label,
  count,
  level = 0,
  open = true,
  onToggle,
  trailing,
  status,
  highlight,
}: {
  label: string;
  count?: number;
  level?: number;
  open?: boolean;
  onToggle?: () => void;
  trailing?: ReactNode;
  status?: DiffStatus;
  highlight?: boolean;
}) {
  const statusCls = highlight && status ? STATUS_CLASS[status] : undefined;
  const cls = [
    'group-head',
    level > 0 ? 'group-head--nested' : '',
    statusCls ?? '',
  ]
    .filter(Boolean)
    .join(' ');
  return (
    <div className={cls}>
      <button type="button" className="group-head__toggle" onClick={onToggle} aria-label={open ? '收起' : '展开'}>
        {open ? '▼' : '▶'}
      </button>
      <span>{label}</span>
      {typeof count === 'number' && <span className="group-head__count">({count} 项)</span>}
      <span style={{ flex: 1 }} />
      {trailing}
    </div>
  );
}
