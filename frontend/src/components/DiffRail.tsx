// frontend/src/components/DiffRail.tsx
import { useMemo } from 'react';
import { buildDiffText } from '../lib/diffList';
import type { DiffEntry } from '../lib/diffList';
import type { DiffStatus } from '../lib/types';

const ENTRY_CLASS: Partial<Record<DiffStatus, string>> = {
  modified: 'rail__entry--mod',
  added: 'rail__entry--add',
  removed: 'rail__entry--rem',
  error: 'rail__entry--err',
};

const PREFIX: Partial<Record<DiffStatus, string>> = {
  modified: '改', added: '仅右', removed: '仅左', error: '错',
};

export function DiffRail({
  entries, onJump, collapsed, onToggle,
}: {
  entries: DiffEntry[];
  onJump(path: string): void;
  collapsed: boolean;
  onToggle(): void;
}) {
  const groups = useMemo(() => {
    const map = new Map<string, DiffEntry[]>();
    for (const e of entries) {
      const key = e.topGroup || '基本信息';
      (map.get(key) ?? map.set(key, []).get(key)!).push(e);
    }
    return [...map.entries()];
  }, [entries]);

  if (collapsed) {
    return (
      <button type="button" className="rail rail--collapsed" onClick={onToggle}
        aria-label="展开差异栏" title={`差异 ${entries.length}`}>
        <span className="rail__collapsed-count">{entries.length}</span>
      </button>
    );
  }

  return (
    <aside className="rail">
      <div className="rail__head">
        <span>差异 {entries.length}</span>
        <button type="button" className="rail__toggle" onClick={onToggle} aria-label="收起差异栏">⇥</button>
      </div>
      <div className="rail__body">
        {entries.length === 0 && <div className="rail__empty">无差异</div>}
        {groups.map(([group, list]) => (
          <div key={group} className="rail__group">
            <div className="rail__group-name">{group}</div>
            {list.map((e) => (
              <button key={e.path} type="button"
                className={`rail__entry ${ENTRY_CLASS[e.status] ?? ''}`.trim()}
                onClick={() => onJump(e.path)} title={e.label}>
                {PREFIX[e.status] ?? ''} · {e.label}
              </button>
            ))}
          </div>
        ))}
      </div>
      {entries.length > 0 && (
        <div className="rail__foot">
          <button type="button" className="rail__copy" aria-label="复制差异清单"
            onClick={() => void navigator.clipboard?.writeText(buildDiffText(entries))}>
            复制差异清单
          </button>
        </div>
      )}
    </aside>
  );
}
