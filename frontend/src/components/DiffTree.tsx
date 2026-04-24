import type { DiffNode } from '../lib/types';

export function DiffTree({
  root,
  activePath,
  onSelect,
}: {
  root: DiffNode;
  activePath: string | null;
  onSelect(path: string): void;
}) {
  return (
    <div className="tree" role="tree" aria-label="Diff tree">
      <TreeNode node={root} activePath={activePath} onSelect={onSelect} />
    </div>
  );
}

function TreeNode({
  node,
  activePath,
  onSelect,
}: {
  node: DiffNode;
  activePath: string | null;
  onSelect(path: string): void;
}) {
  return (
    <div className="tree__node">
      <button
        type="button"
        className={activePath === node.path ? 'tree__button tree__button--active' : 'tree__button'}
        onClick={() => onSelect(node.path)}
      >
        <span>{node.summary}</span>
        <small className="tree__meta">
          {node.path} · {node.status}
        </small>
      </button>
      {node.children.length > 0 ? (
        <div className="tree__children">
          {node.children.map((child) => (
            <TreeNode
              key={`${child.path}-${child.summary}`}
              node={child}
              activePath={activePath}
              onSelect={onSelect}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
