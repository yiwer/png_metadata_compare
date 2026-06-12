// frontend/src/lib/treeModel.ts
import {
  arrayItemLabel,
  arrayKeyFn,
  fieldLabel,
  formatValue,
  isBlank,
  isKnownField,
  schemaChildKeys,
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
  leftRaw?: JsonValue;
  rightRaw?: JsonValue;
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
    const row = mergeArray(
      (leftIsArr ? left : []) as JsonValue[],
      (rightIsArr ? right : []) as JsonValue[],
      path,
      diffMap,
    );
    // 合成的 [] 只用于对齐子项；raw 必须反映真实缺失，否则
    // 「复制 JSON 子树」的 leftRaw ?? rightRaw 会被空数组截胡。
    row.leftRaw = leftIsArr ? left : undefined;
    row.rightRaw = rightIsArr ? right : undefined;
    return row;
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
  const push = (k: string) => {
    if (!seen.has(k)) {
      seen.add(k);
      keys.push(k);
    }
  };
  for (const src of [left, right]) {
    if (!src) continue;
    for (const k of Object.keys(src)) push(k);
  }
  for (const k of schemaChildKeys(path)) push(k);
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
    leftRaw: left ?? undefined,
    rightRaw: right ?? undefined,
    status,
    isUnknown: path !== '' && !isKnownField(path),
    defaultOpen: defaultOpenFor(variant),
    children,
  };
}

function normalizeArrayPath(path: string): string {
  return path.replace(/\[[^\]]*\]/g, '[*]');
}

/** 数组展示排序（不影响 keyed 匹配语义，只决定渲染顺序）。 */
function sortForDisplay(path: string, items: JsonValue[]): JsonValue[] {
  if (normalizeArrayPath(path) !== 'Lines[*].RouteStops') return items;
  // 浅拷贝再排，避免修改入参；按 Sequence 升序，缺失/非数字的丢到末尾。
  return [...items].sort((a, b) => {
    const av = (a as any)?.Sequence;
    const bv = (b as any)?.Sequence;
    const an = typeof av === 'number' ? av : Number.POSITIVE_INFINITY;
    const bn = typeof bv === 'number' ? bv : Number.POSITIVE_INFINITY;
    return an - bn;
  });
}

function mergeArrayItem(
  li: JsonValue | undefined,
  ri: JsonValue | undefined,
  itemPath: string,
  diffMap: Map<string, DiffStatus>,
  labelItem: string,
): MirrorRow {
  if (isObject(li) || isObject(ri)) {
    return mergeObject(
      (isObject(li) ? li : null) as Record<string, JsonValue> | null,
      (isObject(ri) ? ri : null) as Record<string, JsonValue> | null,
      itemPath,
      diffMap,
      'array-item',
      labelItem,
    );
  }
  return mergeLeaf(li as JsonValue, ri as JsonValue, itemPath, diffMap, labelItem);
}

function mergeArray(
  left: JsonValue[],
  right: JsonValue[],
  path: string,
  diffMap: Map<string, DiffStatus>,
): MirrorRow {
  // RouteStops 按 Sequence 升序展示——位置序由数据自身保证。
  const orderedLeft = sortForDisplay(path, left);
  const orderedRight = sortForDisplay(path, right);

  const keyFn = arrayKeyFn(path);
  const children: MirrorRow[] = keyFn
    ? buildKeyedChildren(orderedLeft, orderedRight, path, diffMap, keyFn)
    : buildPositionalChildren(orderedLeft, orderedRight, path, diffMap);

  const status = statusFor(path, diffMap);
  return {
    kind: 'group',
    path,
    label: fieldLabel(path),
    variant: 'array',
    count: children.length,
    leftValue: null,
    rightValue: null,
    leftRaw: left,
    rightRaw: right,
    status,
    isUnknown: !isKnownField(path),
    defaultOpen: false,
    children,
  };
}

function buildPositionalChildren(
  left: JsonValue[],
  right: JsonValue[],
  path: string,
  diffMap: Map<string, DiffStatus>,
): MirrorRow[] {
  const len = Math.max(left.length, right.length);
  const children: MirrorRow[] = [];
  for (let i = 0; i < len; i++) {
    const itemPath = `${path}[${i}]`;
    const li = left[i];
    const ri = right[i];
    const labelItem = arrayItemLabel(path, i, li ?? ri);
    children.push(mergeArrayItem(li, ri, itemPath, diffMap, labelItem));
  }
  return children;
}

/**
 * Keyed-array merging that mirrors the backend's compare_keyed_array:
 * - business-key segments instead of numeric indices
 * - consumptive matching for duplicate keys, with #N suffix when count > 1
 * - keys with no business key fall back to positional segments so the row still appears
 *
 * Iteration order: left occurrences first (in input order), then any right-only keys
 * (in right input order). Items missing the business key are appended at the end with
 * positional segments.
 */
function buildKeyedChildren(
  left: JsonValue[],
  right: JsonValue[],
  path: string,
  diffMap: Map<string, DiffStatus>,
  keyFn: (item: unknown) => string | null,
): MirrorRow[] {
  const leftIndex = new Map<string, JsonValue[]>();
  const rightIndex = new Map<string, JsonValue[]>();
  const orphanLeft: Array<{ pos: number; value: JsonValue }> = [];
  const orphanRight: Array<{ pos: number; value: JsonValue }> = [];

  left.forEach((value, pos) => {
    const k = keyFn(value);
    if (k === null) orphanLeft.push({ pos, value });
    else (leftIndex.get(k) ?? leftIndex.set(k, []).get(k)!).push(value);
  });
  right.forEach((value, pos) => {
    const k = keyFn(value);
    if (k === null) orphanRight.push({ pos, value });
    else (rightIndex.get(k) ?? rightIndex.set(k, []).get(k)!).push(value);
  });

  // Iteration order: keys as encountered in left, then right-only keys
  const seenKeys = new Set<string>();
  const orderedKeys: string[] = [];
  const enqueue = (k: string) => {
    if (!seenKeys.has(k)) {
      seenKeys.add(k);
      orderedKeys.push(k);
    }
  };
  for (const item of left) {
    const k = keyFn(item);
    if (k !== null) enqueue(k);
  }
  for (const item of right) {
    const k = keyFn(item);
    if (k !== null) enqueue(k);
  }

  const children: MirrorRow[] = [];
  for (const key of orderedKeys) {
    const lefts = leftIndex.get(key) ?? [];
    const rights = rightIndex.get(key) ?? [];
    const max = Math.max(lefts.length, rights.length);
    const withSuffix = max > 1;
    for (let i = 0; i < max; i++) {
      const segment = withSuffix ? `${key}#${i + 1}` : key;
      const itemPath = `${path}[${segment}]`;
      const li = lefts[i];
      const ri = rights[i];
      const labelItem = arrayItemLabel(path, i, li ?? ri);
      children.push(mergeArrayItem(li, ri, itemPath, diffMap, labelItem));
    }
  }

  // Items lacking a business key on either side: surface them positionally so the
  // user still sees them. Backend would emit error nodes; here we just render rows.
  for (const { pos, value } of orphanLeft) {
    const itemPath = `${path}[${pos}]`;
    const labelItem = arrayItemLabel(path, pos, value);
    children.push(mergeArrayItem(value, undefined, itemPath, diffMap, labelItem));
  }
  for (const { pos, value } of orphanRight) {
    const itemPath = `${path}[${pos}]`;
    const labelItem = arrayItemLabel(path, pos, value);
    children.push(mergeArrayItem(undefined, value, itemPath, diffMap, labelItem));
  }

  return children;
}

function mergeLeaf(
  left: JsonValue | undefined,
  right: JsonValue | undefined,
  path: string,
  diffMap: Map<string, DiffStatus>,
  labelOverride?: string,
): MirrorRow {
  const status = statusFor(path, diffMap);
  // 缺失字段 / null / "" 都归一展示，不再保留 JS null 触发"仅另一侧存在"占位
  return {
    kind: 'leaf',
    path,
    label: labelOverride ?? fieldLabel(path),
    leftValue: formatValue(path, isBlank(left) ? null : left),
    rightValue: formatValue(path, isBlank(right) ? null : right),
    leftRaw: left,
    rightRaw: right,
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
