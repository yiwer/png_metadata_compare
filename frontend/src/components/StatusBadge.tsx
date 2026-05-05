// frontend/src/components/StatusBadge.tsx
import type { ReactNode } from 'react';
import type { DiffStatus } from '../lib/types';

export type StatusKind = DiffStatus;

const CLASS: Record<StatusKind, string> = {
  modified: 'badge--mod',
  added: 'badge--add',
  removed: 'badge--rem',
  error: 'badge--err',
  reordered: 'badge--neu',
  unchanged: 'badge--neu',
};

export function StatusBadge({ kind, children }: { kind: StatusKind; children: ReactNode }) {
  return <span className={`badge ${CLASS[kind]}`}>{children}</span>;
}
