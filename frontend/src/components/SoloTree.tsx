// frontend/src/components/SoloTree.tsx
import { useEffect, useMemo, useState } from 'react';
import { GroupHead } from './GroupHead';
import { buildTree } from '../lib/treeModel';
import type { GroupNode, LeafNode, TreeNode } from '../lib/treeModel';
import type { JsonValue } from '../lib/types';

export function SoloTree({ value }: { value: JsonValue }) {
  const tree = useMemo(() => buildTree(value), [value]);
  const [closed, setClosed] = useState<Set<string>>(() => collectDefaultClosed(tree));

  // Reset folding state whenever the underlying data changes
  useEffect(() => {
    setClosed(collectDefaultClosed(tree));
  }, [tree]);

  const toggle = (path: string) =>
    setClosed((cur) => {
      const next = new Set(cur);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  return (
    <div className="solo-body" role="tree">
      {tree.children.map((child) => (
        <NodeView key={child.path} node={child} level={0} closed={closed} toggle={toggle} />
      ))}
    </div>
  );
}

function collectDefaultClosed(node: TreeNode, into = new Set<string>()): Set<string> {
  if (node.kind === 'group') {
    if (!node.defaultOpen && node.path) into.add(node.path);
    for (const c of node.children) collectDefaultClosed(c, into);
  }
  return into;
}

function NodeView({
  node,
  level,
  closed,
  toggle,
}: {
  node: TreeNode;
  level: number;
  closed: Set<string>;
  toggle: (p: string) => void;
}) {
  if (node.kind === 'leaf') return <LeafView leaf={node} />;
  return <GroupView group={node} level={level} closed={closed} toggle={toggle} />;
}

function LeafView({ leaf }: { leaf: LeafNode }) {
  return (
    <div className="kv">
      <span className="kv__key">{leaf.label}</span>
      <span className="kv__val">{leaf.value}</span>
    </div>
  );
}

function GroupView({
  group,
  level,
  closed,
  toggle,
}: {
  group: GroupNode;
  level: number;
  closed: Set<string>;
  toggle: (p: string) => void;
}) {
  const isOpen = !closed.has(group.path);
  const isObjectRoot = group.variant === 'object-root';

  // Object-root just renders its children flat (no head, no nested wrapper).
  if (isObjectRoot) {
    return (
      <>
        {group.children.map((c) => (
          <NodeView key={c.path} node={c} level={level} closed={closed} toggle={toggle} />
        ))}
      </>
    );
  }

  return (
    <>
      <GroupHead
        label={group.label}
        count={group.variant === 'array' ? group.count : undefined}
        level={level}
        open={isOpen}
        onToggle={() => toggle(group.path)}
      />
      {isOpen && (
        <div className="tree__nested">
          {group.children.map((c) => (
            <NodeView key={c.path} node={c} level={level + 1} closed={closed} toggle={toggle} />
          ))}
        </div>
      )}
    </>
  );
}
