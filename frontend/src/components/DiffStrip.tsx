// frontend/src/components/DiffStrip.tsx
import type { DiffNode, DiffStatus } from '../lib/types';

function collectChanges(node: DiffNode, acc: DiffNode[] = []): DiffNode[] {
  if (node.status !== 'unchanged' && node.status !== 'error') {
    acc.push(node);
  }
  for (const child of node.children) {
    collectChanges(child, acc);
  }
  return acc;
}

const STATUS_SYMBOL: Record<DiffStatus, string> = {
  modified: '~',
  added: '+',
  removed: '−',
  reordered: '⇄',
  error: '!',
  unchanged: '',
};

export function DiffStrip({ root }: { root: DiffNode | null }) {
  if (!root) {
    return (
      <div className="diff-strip diff-strip--empty">
        <span className="diff-strip__label">Diff</span>
      </div>
    );
  }

  const changes = collectChanges(root);

  if (changes.length === 0) {
    return (
      <div className="diff-strip diff-strip--none">
        <span className="diff-strip__label">Diff</span>
        <span className="diff-strip__no-changes">No changes</span>
      </div>
    );
  }

  return (
    <div className="diff-strip">
      <span className="diff-strip__label">Changes ({changes.length})</span>
      <div className="diff-strip__list">
        {changes.map((node) => (
          <div key={node.path} className={`diff-row diff-row--${node.status}`}>
            <span className="diff-row__symbol">{STATUS_SYMBOL[node.status]}</span>
            <span className="diff-row__path">{shortPath(node.path)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function shortPath(path: string): string {
  const parts = path.split('.');
  return parts[parts.length - 1] ?? path;
}
