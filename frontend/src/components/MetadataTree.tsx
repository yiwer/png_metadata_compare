// frontend/src/components/MetadataTree.tsx
import { useState } from 'react';
import type { DiffStatus, JsonValue } from '../lib/types';

export function MetadataTree({
  value,
  prefix = '',
  diffPathMap,
  highlight = false,
}: {
  value: JsonValue;
  prefix?: string;
  diffPathMap?: Map<string, DiffStatus>;
  highlight?: boolean;
}) {
  if (Array.isArray(value)) {
    return (
      <div className="meta-branch">
        {value.map((entry, index) => {
          const path = `${prefix}[${index}]`;
          return (
            <MetadataTree key={path} value={entry} prefix={path} diffPathMap={diffPathMap} highlight={highlight} />
          );
        })}
      </div>
    );
  }

  if (value && typeof value === 'object') {
    return (
      <div className="meta-branch">
        {Object.entries(value).map(([key, child]) => {
          const path = prefix ? `${prefix}.${key}` : key;
          return (
            <MetadataNode key={path} path={path} label={key} child={child} diffPathMap={diffPathMap} highlight={highlight} />
          );
        })}
      </div>
    );
  }

  const label = prefix || 'value';
  const status = diffPathMap?.get(label);
  return (
    <div className={nodeClass(status, highlight)}>
      <span className="node-key">{label}</span>
      <span className="node-val">{String(value)}</span>
      {highlight && status && status !== 'unchanged' && (
        <span className={`node-badge node-badge--${status}`}>{statusSymbol(status)}</span>
      )}
    </div>
  );
}

function MetadataNode({
  path,
  label,
  child,
  diffPathMap,
  highlight,
}: {
  path: string;
  label: string;
  child: JsonValue;
  diffPathMap?: Map<string, DiffStatus>;
  highlight: boolean;
}) {
  const [open, setOpen] = useState(true);
  const hasChildren = child !== null && typeof child === 'object';
  const status = diffPathMap?.get(path);

  return (
    <div className={`meta-node ${nodeClass(status, highlight)}`}>
      <button
        type="button"
        className="meta-row"
        onClick={() => { if (hasChildren) setOpen((v) => !v); }}
      >
        {hasChildren && <span className="meta-toggle">{open ? '▼' : '▶'}</span>}
        <span className="node-dot" data-status={highlight && status ? status : undefined} />
        <span className="node-key">{label}</span>
        {!hasChildren && <span className="node-val">{String(child)}</span>}
        {highlight && status && status !== 'unchanged' && (
          <span className={`node-badge node-badge--${status}`}>{statusSymbol(status)}</span>
        )}
      </button>
      {hasChildren && open && (
        <div className="meta-children">
          <MetadataTree value={child} prefix={path} diffPathMap={diffPathMap} highlight={highlight} />
        </div>
      )}
    </div>
  );
}

function nodeClass(status: DiffStatus | undefined, highlight: boolean): string {
  if (!highlight || !status || status === 'unchanged') return '';
  return `node--${status}`;
}

function statusSymbol(status: DiffStatus): string {
  switch (status) {
    case 'added': return '+';
    case 'removed': return '−';
    case 'modified': return '~';
    case 'reordered': return '⇄';
    default: return '!';
  }
}
