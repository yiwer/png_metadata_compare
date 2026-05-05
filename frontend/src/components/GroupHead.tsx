// frontend/src/components/GroupHead.tsx
import type { ReactNode } from 'react';

export function GroupHead({
  label,
  count,
  level = 0,
  open = true,
  onToggle,
  trailing,
}: {
  label: string;
  count?: number;
  level?: number;
  open?: boolean;
  onToggle?: () => void;
  trailing?: ReactNode;
}) {
  const cls = `group-head${level > 0 ? ' group-head--nested' : ''}`;
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
