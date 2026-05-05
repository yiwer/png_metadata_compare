// frontend/src/lib/treeModel.ts
import {
  arrayItemLabel,
  fieldLabel,
  formatValue,
  isKnownField,
} from './labels';
import type { DiffNode, DiffStatus, JsonValue } from './types';

export type TreeNodeVariant = 'object-root' | 'object' | 'array' | 'array-item';

export interface LeafNode {
  kind: 'leaf';
  path: string;
  label: string;
  value: string;
  raw: JsonValue;
  isUnknown: boolean;
}

export interface GroupNode {
  kind: 'group';
  path: string;
  label: string;
  variant: TreeNodeVariant;
  count?: number;
  defaultOpen: boolean;
  children: TreeNode[];
  raw: JsonValue;
  isUnknown: boolean;
}

export type TreeNode = LeafNode | GroupNode;

const isObject = (v: unknown): v is Record<string, JsonValue> =>
  v !== null && typeof v === 'object' && !Array.isArray(v);

function defaultOpenFor(variant: TreeNodeVariant): boolean {
  // Object roots and direct object groups stay open; arrays / array items default closed.
  return variant === 'object-root' || variant === 'object';
}

export function buildTree(value: JsonValue): GroupNode {
  return buildObject(value, '', 'object-root');
}

function buildObject(
  value: JsonValue,
  path: string,
  variant: TreeNodeVariant,
  labelOverride?: string,
): GroupNode {
  const obj = isObject(value) ? value : {};
  const label = labelOverride ?? (path ? fieldLabel(path) : '');
  const children: TreeNode[] = Object.entries(obj).map(([k, v]) => {
    const childPath = path ? `${path}.${k}` : k;
    return buildAny(v, childPath);
  });
  return {
    kind: 'group',
    path,
    label,
    variant,
    defaultOpen: defaultOpenFor(variant),
    children,
    raw: value,
    isUnknown: path !== '' && !isKnownField(path),
  };
}

function buildArray(value: JsonValue[], path: string): GroupNode {
  const children: TreeNode[] = value.map((item, idx) => {
    const itemPath = `${path}[${idx}]`;
    if (isObject(item)) {
      return buildObject(item, itemPath, 'array-item', arrayItemLabel(path, idx, item));
    }
    return buildLeaf(item as JsonValue, itemPath);
  });
  return {
    kind: 'group',
    path,
    label: fieldLabel(path),
    variant: 'array',
    count: value.length,
    defaultOpen: false,
    children,
    raw: value,
    isUnknown: !isKnownField(path),
  };
}

function buildLeaf(value: JsonValue, path: string): LeafNode {
  return {
    kind: 'leaf',
    path,
    label: fieldLabel(path),
    value: formatValue(path, value),
    raw: value,
    isUnknown: !isKnownField(path),
  };
}

function buildAny(value: JsonValue, path: string): TreeNode {
  if (Array.isArray(value)) return buildArray(value, path);
  if (isObject(value)) return buildObject(value, path, 'object');
  return buildLeaf(value, path);
}

// =============================================================
// Mirror merge
// =============================================================

export interface MirrorRow {
  kind: 'leaf' | 'group';
  path: string;
  label: string;
  variant?: TreeNodeVariant;
  count?: number;
  leftValue: string | null;
  rightValue: string | null;
  status: DiffStatus;
  isUnknown: boolean;
  defaultOpen: boolean;
  children?: MirrorRow[];
}

function buildDiffMap(node: DiffNode | null, into = new Map<string, DiffStatus>()): Map<string, DiffStatus> {
  if (!node) return into;
  if (node.path) into.set(node.path, node.status);
  for (const c of node.children) buildDiffMap(c, into);
  return into;
}

export function buildMirrorRows(
  left: JsonValue | null,
  right: JsonValue | null,
  diffRoot: DiffNode | null,
): MirrorRow[] {
  const diffMap = buildDiffMap(diffRoot);
  const root = mergeAny(left, right, '', diffMap, 'object-root');
  return [root];
}

function statusFor(path: string, diffMap: Map<string, DiffStatus>): DiffStatus {
  return diffMap.get(path) ?? 'unchanged';
}

function mergeAny(
  left: JsonValue | null | undefined,
  right: JsonValue | null | undefined,
  path: string,
  diffMap: Map<string, DiffStatus>,
  variantHint?: TreeNodeVariant,
  labelOverride?: string,
): MirrorRow {
  const leftIsObj = isObject(left);
  const rightIsObj = isObject(right);
  const leftIsArr = Array.isArray(left);
  const rightIsArr = Array.isArray(right);

  if (leftIsArr || rightIsArr) {
    return mergeArray(
      (leftIsArr ? left : []) as JsonValue[],
      (rightIsArr ? right : []) as JsonValue[],
      path,
      diffMap,
    );
  }

  if (leftIsObj || rightIsObj) {
    return mergeObject(
      (leftIsObj ? left : null) as Record<string, JsonValue> | null,
      (rightIsObj ? right : null) as Record<string, JsonValue> | null,
      path,
      diffMap,
      variantHint ?? 'object',
      labelOverride,
    );
  }

  return mergeLeaf(left as JsonValue, right as JsonValue, path, diffMap);
}

function mergeObject(
  left: Record<string, JsonValue> | null,
  right: Record<string, JsonValue> | null,
  path: string,
  diffMap: Map<string, DiffStatus>,
  variant: TreeNodeVariant,
  labelOverride?: string,
): MirrorRow {
  const keys: string[] = [];
  const seen = new Set<string>();
  for (const src of [left, right]) {
    if (!src) continue;
    for (const k of Object.keys(src)) {
      if (!seen.has(k)) {
        seen.add(k);
        keys.push(k);
      }
    }
  }
  const children: MirrorRow[] = keys.map((k) => {
    const child = path ? `${path}.${k}` : k;
    return mergeAny(left?.[k], right?.[k], child, diffMap);
  });
  const status = statusFor(path, diffMap);
  return {
    kind: 'group',
    path,
    label: labelOverride ?? (path ? fieldLabel(path) : ''),
    variant,
    leftValue: null,
    rightValue: null,
    status,
    isUnknown: path !== '' && !isKnownField(path),
    defaultOpen: defaultOpenFor(variant),
    children,
  };
}

function mergeArray(
  left: JsonValue[],
  right: JsonValue[],
  path: string,
  diffMap: Map<string, DiffStatus>,
): MirrorRow {
  const len = Math.max(left.length, right.length);
  const children: MirrorRow[] = [];
  for (let i = 0; i < len; i++) {
    const itemPath = `${path}[${i}]`;
    const li = left[i];
    const ri = right[i];
    const item = li ?? ri;
    const labelItem = arrayItemLabel(path, i, item);
    if (isObject(li) || isObject(ri)) {
      children.push(
        mergeObject(
          (isObject(li) ? li : null) as Record<string, JsonValue> | null,
          (isObject(ri) ? ri : null) as Record<string, JsonValue> | null,
          itemPath,
          diffMap,
          'array-item',
          labelItem,
        ),
      );
    } else {
      children.push(mergeLeaf(li as JsonValue, ri as JsonValue, itemPath, diffMap, labelItem));
    }
  }
  const status = statusFor(path, diffMap);
  return {
    kind: 'group',
    path,
    label: fieldLabel(path),
    variant: 'array',
    count: len,
    leftValue: null,
    rightValue: null,
    status,
    isUnknown: !isKnownField(path),
    defaultOpen: false,
    children,
  };
}

function mergeLeaf(
  left: JsonValue | undefined,
  right: JsonValue | undefined,
  path: string,
  diffMap: Map<string, DiffStatus>,
  labelOverride?: string,
): MirrorRow {
  const status = statusFor(path, diffMap);
  return {
    kind: 'leaf',
    path,
    label: labelOverride ?? fieldLabel(path),
    leftValue: left === undefined ? null : formatValue(path, left),
    rightValue: right === undefined ? null : formatValue(path, right),
    status,
    isUnknown: !isKnownField(path),
    defaultOpen: true,
  };
}

export function hasDiffDeep(row: MirrorRow): boolean {
  if (row.status !== 'unchanged') return true;
  if (row.children) return row.children.some(hasDiffDeep);
  return false;
}
