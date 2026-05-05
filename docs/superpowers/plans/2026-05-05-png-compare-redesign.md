# PNG 元数据对比工具 — 全面重设计 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按 spec `docs/superpowers/specs/2026-05-05-png-compare-redesign-design.md` 落地一套全新的 UI/UX：中文字段映射、单 PNG 拖入即看、左右镜像对比、单列表目录概览、黑底高对比视觉系统。

**Architecture:** 新增两个纯数据层模块（`labels.ts` 字段映射、`treeModel.ts` 树构建/合并）；按职责拆出原子组件（`StatusBadge` / `GroupHead`）与三个核心视图组件（`SoloTree` / `MirrorTree` / `DirectoryList`）；外壳组件（`Slot` / `SlotBar`）+ App 装配；`useWorkbench` 增 solo 视图、模式自动切换、键盘快捷键。视觉 token 完整重写为高对比深色系。后端 API 不变。

**Tech Stack:** TypeScript 5.7 / React 18 / Vite 6 / Vitest 3 / @testing-library/react 16 / Tauri 2 (前端 dist 由 Tauri 包装).

---

## 文件结构（决策已锁定）

```
frontend/src/
├── lib/
│   ├── labels.ts                  [NEW] 字段中文映射 + 摘要标签生成器 + 值格式化
│   ├── labels.test.ts             [NEW]
│   ├── treeModel.ts               [NEW] JsonValue→TreeNode；左右合并→MirrorRow；折叠状态查询
│   ├── treeModel.test.ts          [NEW]
│   ├── api.ts                     [keep]
│   ├── types.ts                   [keep]
│   └── diffUtils.ts               [delete after Task 9]
├── components/
│   ├── StatusBadge.tsx            [NEW] 状态徽章
│   ├── StatusBadge.test.tsx       [NEW]
│   ├── GroupHead.tsx              [NEW] 分组小标题（顶层 / 嵌套）
│   ├── GroupHead.test.tsx         [NEW]
│   ├── SoloTree.tsx               [NEW] 单边树
│   ├── SoloTree.test.tsx          [NEW]
│   ├── MirrorTree.tsx             [NEW] 1v1 镜像同步对比
│   ├── MirrorTree.test.tsx        [NEW]
│   ├── Slot.tsx                   [NEW] 单个槽位（4 状态 + 拖放）
│   ├── Slot.test.tsx              [NEW]
│   ├── SlotBar.tsx                [NEW] 顶部槽位条 + 折叠
│   ├── SlotBar.test.tsx           [NEW]
│   ├── DirectoryList.tsx          [NEW] 重写目录概览
│   ├── DirectoryList.test.tsx     [NEW]
│   ├── ImagePane.tsx              [keep] 仅样式重做
│   ├── RawJsonPanel.tsx           [keep] 仅样式重做
│   ├── DiffTree.tsx               [DELETE Task 11]
│   ├── MetadataTree.tsx           [DELETE Task 11]
│   ├── PairComparison.tsx         [DELETE Task 11]
│   ├── DirectoryOverview.tsx      [DELETE Task 11]
│   ├── EmptyState.tsx             [DELETE Task 11]
│   ├── StatusBanner.tsx           [DELETE Task 11]
│   ├── DiffStrip.tsx              [DELETE Task 11]
│   ├── FileCard.tsx               [DELETE Task 11]
│   ├── ViewModeStrip.tsx          [DELETE Task 11]
│   └── workbench-sync.test.tsx    [DELETE Task 11]
├── features/workbench/
│   ├── useWorkbench.ts            [REWRITE Task 8]
│   └── useWorkbench.test.tsx      [UPDATE Task 8]
├── styles/
│   ├── tokens.css                 [REPLACE Task 3]
│   └── app.css                    [REPLACE Task 3]
├── App.tsx                        [REWRITE Task 10]
├── App.test.tsx                   [UPDATE Task 10]
├── main.tsx                       [keep]
└── build-config.test.ts           [keep]
```

---

## 任务总览

| # | 任务 | 依赖 |
|---|---|---|
| 1 | 字段中文映射与值格式化（labels.ts）+ 单测 | — |
| 2 | 树模型与镜像合并（treeModel.ts）+ 单测 | 1 |
| 3 | 视觉 token 重写（tokens.css / app.css） | — |
| 4 | 原子组件 StatusBadge + GroupHead | 3 |
| 5 | SoloTree（单边树） | 1, 2, 4 |
| 6 | MirrorTree（1v1 镜像） | 5 |
| 7 | Slot + SlotBar（槽位 + 顶栏） | 3 |
| 8 | useWorkbench 改造（solo / 模式切换 / 键盘） | 1, 2 |
| 9 | DirectoryList（目录概览） | 4, 8 |
| 10 | App.tsx 装配 + 控件条 + 视图切换 | 5, 6, 7, 8, 9 |
| 11 | 删除旧组件 / 旧样式 / 旧测试 | 10 |
| 12 | 端到端 smoke + 构建/类型检查 | 11 |

---

## Task 1 — 字段中文映射与值格式化

**Files:**
- Create: `frontend/src/lib/labels.ts`
- Test: `frontend/src/lib/labels.test.ts`

### 1.1 设计的接口

```ts
// 字段路径形式："StopName" / "Lines[*].LineName" / "Lines[*].RouteStops[*].BuildingType"
// `[*]` 表示数组通配符
export interface FieldDef {
  /** 中文标签 */
  label: string;
  /** 把原始值转成展示字符串。null 处理在内部完成。 */
  format?: (value: unknown) => string;
}

export const FIELD_DEFS: Record<string, FieldDef>;

/** 按运行时路径查表，自动把 [N] 归一化为 [*] */
export function fieldLabel(path: string): string;

/** 同上，但返回 FieldDef（含 format） */
export function fieldDef(path: string): FieldDef | undefined;

/** 数组元素的人类摘要标签：分站①、线路 1 · B932、#3 · 翻身地铁站 */
export function arrayItemLabel(arrayPath: string, index: number, item: unknown): string;

/** 把任意 JSON 值格式化为展示字符串，应用 path 对应的 format 规则；
 *  null/undefined → "—"；未知字段使用通用规则。 */
export function formatValue(path: string, value: unknown): string;

/** 该字段是否在已知 schema 中（用于 "未识别" 徽章） */
export function isKnownField(path: string): boolean;
```

- [ ] **Step 1.1 — 写失败测试 `labels.test.ts`**

```ts
// frontend/src/lib/labels.test.ts
import { describe, it, expect } from 'vitest';
import {
  fieldLabel,
  fieldDef,
  formatValue,
  arrayItemLabel,
  isKnownField,
} from './labels';

describe('fieldLabel', () => {
  it('returns Chinese label for top-level fields', () => {
    expect(fieldLabel('StopName')).toBe('中文站名');
    expect(fieldLabel('DirectionOnRoad')).toBe('在道路的方位');
    expect(fieldLabel('FrameSize')).toBe('站架尺寸');
    expect(fieldLabel('RenderTime')).toBe('渲染时间');
  });

  it('normalizes array-element paths via [*]', () => {
    expect(fieldLabel('Lines[3].LineName')).toBe('线路名称');
    expect(fieldLabel('Lines[0].RouteStops[5].Name')).toBe('站点名称');
    expect(fieldLabel('GroupItems[1].SequenceNo')).toBe('分站序号');
  });

  it('falls back to the raw key for unknown paths', () => {
    expect(fieldLabel('UnknownField')).toBe('UnknownField');
    expect(fieldLabel('Lines[0].UnknownNested')).toBe('UnknownNested');
  });
});

describe('isKnownField', () => {
  it('flags known and unknown', () => {
    expect(isKnownField('StopName')).toBe(true);
    expect(isKnownField('Lines[*].LineName')).toBe(true);
    expect(isKnownField('Lines[2].LineName')).toBe(true);
    expect(isKnownField('UnknownField')).toBe(false);
  });
});

describe('formatValue', () => {
  it('formats null → em dash', () => {
    expect(formatValue('OriName', null)).toBe('—');
    expect(formatValue('Lines[0].NextStop', null)).toBe('—（已到终点）');
    expect(formatValue('Lines[0].ScheduledServiceDescription', null)).toBe('—（非定时班车）');
  });

  it('formats empty string → "(空)"', () => {
    expect(formatValue('StopName', '')).toBe('(空)');
  });

  it('renders booleans contextually', () => {
    expect(formatValue('HasHints', true)).toBe('是');
    expect(formatValue('HasHints', false)).toBe('否');
    expect(formatValue('IsBack', true)).toBe('反面');
    expect(formatValue('IsBack', false)).toBe('正面');
    expect(formatValue('GroupItems[0].IsCurrent', true)).toBe('是（当前）');
    expect(formatValue('GroupItems[0].IsCurrent', false)).toBe('否');
  });

  it('appends unit to Distance', () => {
    expect(formatValue('GroupItems[0].Distance', 150)).toBe('150 米');
  });

  it('translates BuildingType enum', () => {
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '地铁')).toBe('地铁换乘');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '公交')).toBe('公交换乘');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '医院')).toBe('医院');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '长途')).toBe('长途客运');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', null)).toBe('—');
    expect(formatValue('Lines[0].RouteStops[0].BuildingType', '')).toBe('—');
  });

  it('passes through plain string / number / unknown booleans', () => {
    expect(formatValue('StopName', '翻身地铁站')).toBe('翻身地铁站');
    expect(formatValue('StopId', 1234)).toBe('1234');
    expect(formatValue('Lines[0].IsStarting', true)).toBe('是');
    expect(formatValue('Lines[0].IsStarting', false)).toBe('否');
  });
});

describe('arrayItemLabel', () => {
  it('uses SequenceNo for GroupItems', () => {
    expect(arrayItemLabel('GroupItems', 0, { SequenceNo: '①' })).toBe('分站 ①');
    expect(arrayItemLabel('GroupItems', 1, { SequenceNo: '②' })).toBe('分站 ②');
    expect(arrayItemLabel('GroupItems', 0, {})).toBe('分站 1');
  });

  it('uses LineName for Lines', () => {
    expect(arrayItemLabel('Lines', 0, { LineName: 'B932' })).toBe('线路 1 · B932');
    expect(arrayItemLabel('Lines', 2, { LineName: 'M375' })).toBe('线路 3 · M375');
    expect(arrayItemLabel('Lines', 0, {})).toBe('线路 1 · ?');
  });

  it('uses Sequence + Name for RouteStops', () => {
    expect(arrayItemLabel('Lines[*].RouteStops', 2, { Sequence: 3, Name: '翻身地铁站' }))
      .toBe('#3 · 翻身地铁站');
    expect(arrayItemLabel('Lines[*].RouteStops', 0, {})).toBe('#1 · ?');
  });

  it('falls back to "项 N" for unknown arrays', () => {
    expect(arrayItemLabel('UnknownArray', 4, {})).toBe('项 5');
  });
});

describe('fieldDef', () => {
  it('returns undefined for unknown path', () => {
    expect(fieldDef('NotAField')).toBeUndefined();
  });
  it('returns the def for known path', () => {
    expect(fieldDef('StopName')).toBeDefined();
    expect(fieldDef('Lines[2].LineName')).toBeDefined();
  });
});
```

- [ ] **Step 1.2 — 跑测试确认全部失败**

Run: `npm --prefix frontend run test -- --run labels.test`
Expected: 全部测试失败（"Cannot find module './labels'"）

- [ ] **Step 1.3 — 实现 `labels.ts`**

```ts
// frontend/src/lib/labels.ts

export interface FieldDef {
  label: string;
  format?: (value: unknown) => string;
}

const yesNo = (v: unknown): string => (v === true ? '是' : v === false ? '否' : String(v));

const BUILDING_TYPE: Record<string, string> = {
  地铁: '地铁换乘',
  公交: '公交换乘',
  医院: '医院',
  长途: '长途客运',
};

export const FIELD_DEFS: Record<string, FieldDef> = {
  // 顶层
  StopId: { label: '站点编号' },
  StopName: { label: '中文站名' },
  StopEngName: { label: '英文站名' },
  OriName: { label: '原站名' },
  RoadName: { label: '所在道路' },
  DirectionOnRoad: { label: '在道路的方位' },
  DistrictName: { label: '所属行政区' },
  StreetCommitteeName: { label: '所属街道' },
  QRCode: { label: '二维码地址' },
  HasHints: { label: '含温馨提示', format: yesNo },
  Hints: { label: '温馨提示内容' },
  IsGroupPrint: { label: '含分站信息', format: yesNo },
  GroupItems: { label: '分站列表' },
  IsBack: { label: '站牌朝向', format: (v) => (v === true ? '反面' : v === false ? '正面' : String(v)) },
  FrameSize: { label: '站架尺寸' },
  Lines: { label: '停靠线路' },
  RenderTime: { label: '渲染时间' },

  // GroupItems 元素
  'GroupItems[*].SequenceNo': { label: '分站序号' },
  'GroupItems[*].LineNames': { label: '该分站线路' },
  'GroupItems[*].Distance': {
    label: '距当前站',
    format: (v) => (typeof v === 'number' ? `${v} 米` : String(v)),
  },
  'GroupItems[*].IsCurrent': {
    label: '是否当前站',
    format: (v) => (v === true ? '是（当前）' : v === false ? '否' : String(v)),
  },

  // Lines 元素
  'Lines[*].LineName': { label: '线路名称' },
  'Lines[*].Direction': { label: '开往方向' },
  'Lines[*].FirstStopName': { label: '线路起点站' },
  'Lines[*].LastStopName': { label: '线路终点站' },
  'Lines[*].NextStop': {
    label: '下一站',
    format: (v) => (v === null || v === undefined ? '—（已到终点）' : String(v)),
  },
  'Lines[*].CurrentStopSequence': { label: '当前站站序' },
  'Lines[*].IsStarting': { label: '当前站是起点', format: yesNo },
  'Lines[*].IsEnding': { label: '当前站是终点', format: yesNo },
  'Lines[*].HeadBusCorpName': { label: '运营企业' },
  'Lines[*].TicketType': { label: '票制类型' },
  'Lines[*].PriceDescription': { label: '票价描述' },
  'Lines[*].ServiceTimeDescription': { label: '服务时间' },
  'Lines[*].ScheduledServiceDescription': {
    label: '发车时刻表',
    format: (v) => (v === null || v === undefined ? '—（非定时班车）' : String(v)),
  },
  'Lines[*].LinePattern': { label: '线路模式' },
  'Lines[*].RouteStops': { label: '途经站点' },

  // RouteStops 元素
  'Lines[*].RouteStops[*].Name': { label: '站点名称' },
  'Lines[*].RouteStops[*].Sequence': { label: '站点序号' },
  'Lines[*].RouteStops[*].BuildingType': {
    label: '附近设施',
    format: (v) => {
      if (v === null || v === undefined || v === '') return '—';
      const s = String(v);
      return BUILDING_TYPE[s] ?? s;
    },
  },
  'Lines[*].RouteStops[*].RoadName': { label: '所在道路' },
};

/** 把运行时路径（含数字下标）归一化为 schema key（用 `[*]` 占位）。 */
function normalizePath(path: string): string {
  return path.replace(/\[\d+\]/g, '[*]');
}

/** 取路径最后一段作为最后兜底标签（如 "UnknownNested"）。 */
function tailKey(path: string): string {
  const lastDot = path.lastIndexOf('.');
  return lastDot === -1 ? path : path.slice(lastDot + 1);
}

export function fieldDef(path: string): FieldDef | undefined {
  return FIELD_DEFS[normalizePath(path)];
}

export function fieldLabel(path: string): string {
  return fieldDef(path)?.label ?? tailKey(path);
}

export function isKnownField(path: string): boolean {
  return FIELD_DEFS[normalizePath(path)] !== undefined;
}

export function formatValue(path: string, value: unknown): string {
  const def = fieldDef(path);
  if (def?.format) return def.format(value);

  if (value === null || value === undefined) return '—';
  if (value === '') return '(空)';
  if (typeof value === 'boolean') return value ? '是' : '否';
  return String(value);
}

export function arrayItemLabel(arrayPath: string, index: number, item: unknown): string {
  const norm = normalizePath(arrayPath);
  const obj = (item ?? {}) as Record<string, unknown>;
  switch (norm) {
    case 'GroupItems':
      return `分站 ${obj.SequenceNo ?? index + 1}`;
    case 'Lines':
      return `线路 ${index + 1} · ${obj.LineName ?? '?'}`;
    case 'Lines[*].RouteStops':
      return `#${obj.Sequence ?? index + 1} · ${obj.Name ?? '?'}`;
    default:
      return `项 ${index + 1}`;
  }
}
```

- [ ] **Step 1.4 — 跑测试确认全部通过**

Run: `npm --prefix frontend run test -- --run labels.test`
Expected: PASS（约 ~25 个 assertion）

- [ ] **Step 1.5 — 提交**

```bash
git add frontend/src/lib/labels.ts frontend/src/lib/labels.test.ts
git commit -m "feat(labels): add Chinese field mapping and value formatter

Implements §1 of the redesign spec — schema-aware Chinese labels for all
StopPlateMetadata fields, contextualized boolean rendering, BuildingType
enum translation, and array-item summary labels."
```

---

## Task 2 — 树模型与镜像合并

**Files:**
- Create: `frontend/src/lib/treeModel.ts`
- Test: `frontend/src/lib/treeModel.test.ts`

### 2.1 接口设计

```ts
import type { JsonValue, DiffNode, DiffStatus } from './types';

export type TreeNodeVariant = 'object-root' | 'object' | 'array' | 'array-item';

export interface LeafNode {
  kind: 'leaf';
  path: string;        // 运行时路径（包含真实下标）
  label: string;       // 中文标签或回退键
  value: string;       // 已用 formatValue 格式化的展示串
  raw: JsonValue;      // 原始值（用于复制 JSON 子树）
  isUnknown: boolean;  // 不在 schema 中
}

export interface GroupNode {
  kind: 'group';
  path: string;
  label: string;
  variant: TreeNodeVariant;
  count?: number;            // 仅 array variants
  defaultOpen: boolean;
  children: TreeNode[];
  raw: JsonValue;
  isUnknown: boolean;
}

export type TreeNode = LeafNode | GroupNode;

/** 顶层入口：把 JSON 元数据构造为单根分组（variant=object-root） */
export function buildTree(value: JsonValue): GroupNode;

/** —— 镜像合并 —— */

export interface MirrorRow {
  kind: 'leaf' | 'group';
  path: string;
  label: string;
  variant?: TreeNodeVariant;
  count?: number;             // 当左右数量不同，取 max
  /** 叶子值：缺失侧为 null（→ 渲染为占位行） */
  leftValue: string | null;
  rightValue: string | null;
  status: DiffStatus;
  isUnknown: boolean;
  defaultOpen: boolean;
  children?: MirrorRow[];
}

/** 把左右两棵树合并为镜像行，使用 diff_root 着色。 */
export function buildMirrorRows(
  left: JsonValue | null,
  right: JsonValue | null,
  diffRoot: DiffNode | null,
): MirrorRow[];

/** 给定 MirrorRow 子树，返回是否包含至少一处非 unchanged 状态（含子树）。 */
export function hasDiffDeep(row: MirrorRow): boolean;
```

设计要点：
- `buildTree` 用于 SoloTree 渲染（单边）。
- `buildMirrorRows` 把两边对齐到同一 schema 树，缺失侧值为 null。
- 数组的下标是按位置对齐（不做语义合并）。后端 diff 已计算好，把 `path → status` 拍到对应 row。
- `RouteStops`、`Lines`、`GroupItems` 等数组默认折叠（`defaultOpen=false`）；`object-root` 和它的直接 object 子分组默认展开。

- [ ] **Step 2.1 — 写失败测试 `treeModel.test.ts`**

```ts
// frontend/src/lib/treeModel.test.ts
import { describe, it, expect } from 'vitest';
import { buildTree, buildMirrorRows, hasDiffDeep } from './treeModel';
import type { DiffNode, JsonValue } from './types';

describe('buildTree', () => {
  it('builds an object-root with leaf children', () => {
    const tree = buildTree({ StopName: '翻身地铁站', StopId: 1234 });
    expect(tree.kind).toBe('group');
    expect(tree.variant).toBe('object-root');
    expect(tree.children).toHaveLength(2);

    const stopName = tree.children[0];
    expect(stopName).toMatchObject({
      kind: 'leaf',
      path: 'StopName',
      label: '中文站名',
      value: '翻身地铁站',
    });
    const stopId = tree.children[1];
    expect(stopId).toMatchObject({
      kind: 'leaf',
      path: 'StopId',
      label: '站点编号',
      value: '1234',
    });
  });

  it('marks unknown fields with isUnknown', () => {
    const tree = buildTree({ Foo: 'bar' });
    expect(tree.children[0]).toMatchObject({ label: 'Foo', isUnknown: true });
  });

  it('renders null/empty/booleans through formatValue', () => {
    const tree = buildTree({ HasHints: false, Hints: null, OriName: '' });
    expect((tree.children[0] as any).value).toBe('否');
    expect((tree.children[1] as any).value).toBe('—');
    expect((tree.children[2] as any).value).toBe('(空)');
  });

  it('builds an array group with array-item children using arrayItemLabel', () => {
    const tree = buildTree({
      Lines: [
        { LineName: 'B932', Direction: '终点A' },
        { LineName: 'M375', Direction: '终点B' },
      ],
    } as JsonValue);

    const linesGroup = tree.children[0];
    expect(linesGroup.kind).toBe('group');
    expect((linesGroup as any).variant).toBe('array');
    expect((linesGroup as any).count).toBe(2);
    expect((linesGroup as any).label).toBe('停靠线路');
    expect((linesGroup as any).defaultOpen).toBe(false);

    const item0 = (linesGroup as any).children[0];
    expect(item0.variant).toBe('array-item');
    expect(item0.label).toBe('线路 1 · B932');
    expect(item0.path).toBe('Lines[0]');
    expect(item0.children).toHaveLength(2);
    expect(item0.children[0]).toMatchObject({
      path: 'Lines[0].LineName',
      label: '线路名称',
      value: 'B932',
    });
  });

  it('handles deeply nested arrays (RouteStops)', () => {
    const tree = buildTree({
      Lines: [
        {
          LineName: 'B932',
          RouteStops: [
            { Name: '上川路口', Sequence: 2, BuildingType: null, RoadName: '福城路' },
          ],
        },
      ],
    } as JsonValue);

    const lines = tree.children[0] as any;
    const line0 = lines.children[0];
    const routeStops = line0.children.find((c: any) => c.path === 'Lines[0].RouteStops');
    expect(routeStops.variant).toBe('array');
    expect(routeStops.children[0].label).toBe('#2 · 上川路口');
    const routeStop0 = routeStops.children[0];
    const buildingType = routeStop0.children.find((c: any) => c.path.endsWith('.BuildingType'));
    expect(buildingType.value).toBe('—');
  });

  it('uses default-open for object groups but default-closed for arrays', () => {
    const tree = buildTree({
      RoadName: '创业一路',
      Lines: [{ LineName: 'B932' }],
      GroupItems: [{ SequenceNo: '①' }],
    } as JsonValue);
    const lines = tree.children.find((c) => c.path === 'Lines') as any;
    const groups = tree.children.find((c) => c.path === 'GroupItems') as any;
    expect(lines.defaultOpen).toBe(false);
    expect(groups.defaultOpen).toBe(false);
  });
});

describe('buildMirrorRows', () => {
  const noDiff: DiffNode = {
    path: '', status: 'unchanged', left_value: null, right_value: null, summary: '', children: [],
  };

  it('renders leaves with both values when keys match', () => {
    const rows = buildMirrorRows({ StopName: '翻身' }, { StopName: '翻身' }, noDiff);
    expect(rows[0].children![0]).toMatchObject({
      kind: 'leaf', path: 'StopName', label: '中文站名',
      leftValue: '翻身', rightValue: '翻身', status: 'unchanged',
    });
  });

  it('marks left-only leaves with status removed and rightValue null', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'Hints', status: 'removed', left_value: '"x"', right_value: null, summary: '', children: [] }],
    };
    const rows = buildMirrorRows({ Hints: 'x' }, {}, diff);
    const leaf = rows[0].children![0];
    expect(leaf).toMatchObject({
      path: 'Hints', leftValue: 'x', rightValue: null, status: 'removed',
    });
  });

  it('marks right-only leaves with status added and leftValue null', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'Hints', status: 'added', left_value: null, right_value: '"y"', summary: '', children: [] }],
    };
    const rows = buildMirrorRows({}, { Hints: 'y' }, diff);
    const leaf = rows[0].children![0];
    expect(leaf).toMatchObject({
      path: 'Hints', leftValue: null, rightValue: 'y', status: 'added',
    });
  });

  it('marks modified leaves with both values present and status modified', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [{ path: 'FrameSize', status: 'modified', left_value: '"1050x1660"', right_value: '"1200x1800"', summary: '', children: [] }],
    };
    const rows = buildMirrorRows(
      { FrameSize: '1050x1660' },
      { FrameSize: '1200x1800' },
      diff,
    );
    const leaf = rows[0].children![0];
    expect(leaf).toMatchObject({
      path: 'FrameSize', leftValue: '1050x1660', rightValue: '1200x1800', status: 'modified',
    });
  });

  it('aligns array elements by index and pads the missing side', () => {
    const diff: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '',
      children: [
        {
          path: 'Lines', status: 'modified', left_value: null, right_value: null, summary: '', children: [
            { path: 'Lines[1]', status: 'added', left_value: null, right_value: null, summary: '', children: [] },
          ],
        },
      ],
    };
    const rows = buildMirrorRows(
      { Lines: [{ LineName: 'B932' }] },
      { Lines: [{ LineName: 'B932' }, { LineName: 'M375' }] },
      diff,
    );
    const lines = rows[0].children![0];
    expect(lines.kind).toBe('group');
    expect(lines.children).toHaveLength(2);
    expect(lines.children![1].status).toBe('added');
    // The missing left item still produces a row whose leaves render as null on left.
    const itemLeaves = lines.children![1].children!;
    expect(itemLeaves.every((l) => l.leftValue === null)).toBe(true);
  });
});

describe('hasDiffDeep', () => {
  it('returns true when any descendant is non-unchanged', () => {
    const row = {
      kind: 'group' as const, path: '', label: 'r', leftValue: null, rightValue: null,
      status: 'unchanged' as const, isUnknown: false, defaultOpen: true,
      children: [
        {
          kind: 'leaf' as const, path: 'a', label: 'a', leftValue: '1', rightValue: '2',
          status: 'modified' as const, isUnknown: false, defaultOpen: true,
        },
      ],
    };
    expect(hasDiffDeep(row)).toBe(true);
  });
  it('returns false when whole subtree unchanged', () => {
    const row = {
      kind: 'leaf' as const, path: 'a', label: 'a', leftValue: '1', rightValue: '1',
      status: 'unchanged' as const, isUnknown: false, defaultOpen: true,
    };
    expect(hasDiffDeep(row)).toBe(false);
  });
});
```

- [ ] **Step 2.2 — 跑测试确认失败**

Run: `npm --prefix frontend run test -- --run treeModel.test`
Expected: 全部失败（"Cannot find module './treeModel'"）

- [ ] **Step 2.3 — 实现 `treeModel.ts`**

```ts
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
```

- [ ] **Step 2.4 — 跑测试确认全部通过**

Run: `npm --prefix frontend run test -- --run treeModel.test`
Expected: PASS (~12 个测试)

- [ ] **Step 2.5 — 提交**

```bash
git add frontend/src/lib/treeModel.ts frontend/src/lib/treeModel.test.ts
git commit -m "feat(treeModel): build tree + mirror merge from metadata + diff

Adds buildTree (single-side rendering) and buildMirrorRows (1v1 mirror with
left/right values and per-row diff status) — pure data layer used by the
upcoming Solo/MirrorTree components."
```

---

## Task 3 — 视觉 token 重写

**Files:**
- Modify: `frontend/src/styles/tokens.css` (整体替换)
- Modify: `frontend/src/styles/app.css` (整体替换)

- [ ] **Step 3.1 — 整体替换 `tokens.css`**

```css
/* frontend/src/styles/tokens.css */
:root {
  /* Surface */
  --bg-page:        #000000;
  --bg-elevated:    #0a0a0a;
  --bg-overlay:     #141414;

  /* Text */
  --text-primary:   #ffffff;
  --text-secondary: #b8c0d0;
  --text-tertiary:  #6c7480;
  --text-disabled:  #3a3f4a;

  /* Borders */
  --border-subtle:  #1a1a1a;
  --border-default: #2a2a2a;
  --border-emph:    #444a55;
  --border-focus:   #5b8dff;

  /* Diff status */
  --mod-bg:    #3a2a00;
  --mod-text:  #ffd23f;
  --mod-arrow: #ffd23f;

  --add-bg:    #0d2d18;
  --add-text:  #5eed90;
  --add-emph:  #5eed90;

  --rem-bg:    #330808;
  --rem-text:  #ff6b6b;
  --rem-emph:  #ff6b6b;

  --reord-bord: #5b8dff;

  --err-bg:    #2a0d2a;
  --err-text:  #ff7ce0;

  /* Group title */
  --group-head:        #ffb547;
  --group-rule:        rgba(255, 181, 71, 0.15);
  --group-bg:          rgba(255, 181, 71, 0.06);
  --group-head-nested: rgba(255, 181, 71, 0.75);
  --group-rule-nested: rgba(255, 181, 71, 0.40);

  /* Accent */
  --accent:    #5b8dff;
  --accent-bg: rgba(91, 141, 255, 0.15);

  /* Fonts */
  --font-ui:   "Inter", "PingFang SC", "Microsoft YaHei", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", "SF Mono", "Cascadia Code", ui-monospace, Consolas, monospace;

  /* Sizes */
  --fs-xs:  11px;  --lh-xs: 1.4;
  --fs-sm:  12px;  --lh-sm: 1.5;
  --fs-md:  13px;  --lh-md: 1.5;
  --fs-lg:  14px;  --lh-lg: 1.4;
  --fs-xl:  18px;  --lh-xl: 1.3;
  --fs-2xl: 24px;  --lh-2xl: 1.2;

  /* Spacing (8px base) */
  --sp-1: 4px;
  --sp-2: 8px;
  --sp-3: 12px;
  --sp-4: 16px;
  --sp-5: 24px;
  --sp-6: 32px;

  /* Radii */
  --r-sm: 4px;
  --r-md: 6px;
  --r-lg: 10px;
}
```

- [ ] **Step 3.2 — 整体替换 `app.css`**

```css
/* frontend/src/styles/app.css */
@import './tokens.css';

* { box-sizing: border-box; margin: 0; padding: 0; }

html, body, #root {
  height: 100%;
  background: var(--bg-page);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--fs-md);
  line-height: var(--lh-md);
  -webkit-font-smoothing: antialiased;
}

body { overflow: hidden; }

button { font: inherit; color: inherit; cursor: pointer; background: transparent; border: 0; }
input { font: inherit; color: inherit; background: transparent; border: 0; }

/* focus ring */
:focus-visible {
  outline: 2px solid var(--border-focus);
  outline-offset: 1px;
}

/* deep scrollbar */
::-webkit-scrollbar { width: 10px; height: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border-default); border-radius: 5px; }
::-webkit-scrollbar-thumb:hover { background: var(--border-emph); }

/* App shell */
.app-shell {
  display: grid;
  grid-template-rows: 36px auto auto 1fr;
  height: 100vh;
}

/* === Top bar === */
.topbar {
  display: flex;
  align-items: center;
  padding: 0 var(--sp-3);
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
}
.topbar-left   { display: flex; align-items: center; gap: var(--sp-3); flex: 0 0 auto; }
.topbar-center { flex: 1; display: flex; justify-content: center; gap: var(--sp-2); color: var(--text-secondary); font-size: var(--fs-sm); }
.topbar-right  { flex: 0 0 auto; }
.brand-icon    { width: 18px; height: 18px; }
.brand         { font-weight: 600; letter-spacing: 0.04em; font-size: var(--fs-sm); }
.topbar-vsep   { width: 1px; height: 16px; background: var(--border-default); }
.win-controls  { display: flex; gap: 2px; }
.win-btn       { width: 36px; height: 28px; color: var(--text-secondary); }
.win-btn:hover { background: var(--bg-overlay); color: var(--text-primary); }
.win-btn--close:hover { background: #c42b1c; color: #fff; }
.mode-toggle   { display: inline-flex; gap: 0; border: 1px solid var(--border-default); border-radius: var(--r-sm); overflow: hidden; }
.mode-btn      { padding: 4px 12px; font-size: var(--fs-xs); color: var(--text-secondary); }
.mode-btn--active { background: var(--accent-bg); color: var(--text-primary); }
.back-btn      { padding: 4px 10px; font-size: var(--fs-xs); color: var(--text-secondary); border: 1px solid var(--border-default); border-radius: var(--r-sm); }
.back-btn:hover { background: var(--bg-overlay); color: var(--text-primary); }

/* === Reusable atoms === */
.badge {
  font: 500 var(--fs-xs)/var(--lh-xs) var(--font-ui);
  letter-spacing: 0.02em;
  padding: 1px 8px;
  border-radius: var(--r-sm);
  white-space: nowrap;
}
.badge--mod { background: rgba(255, 210, 63, 0.16); color: var(--mod-text); }
.badge--add { background: rgba(94, 237, 144, 0.16); color: var(--add-text); }
.badge--rem { background: rgba(255, 107, 107, 0.16); color: var(--rem-text); }
.badge--err { background: rgba(255, 124, 224, 0.16); color: var(--err-text); }
.badge--neu { background: rgba(184, 192, 208, 0.10); color: var(--text-secondary); }

.group-head {
  font: 500 var(--fs-xs)/var(--lh-xs) var(--font-ui);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--group-head);
  padding: 6px 10px;
  background: var(--group-bg);
  border-left: 2px solid var(--group-head);
  border-radius: 2px;
  margin: var(--sp-3) 0 var(--sp-1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
}
.group-head--nested {
  font-size: 10px;
  background: transparent;
  color: var(--group-head-nested);
  border-left-color: var(--group-rule-nested);
  margin: var(--sp-2) 0 2px;
}
.group-head__count { color: var(--text-tertiary); font-size: 10px; letter-spacing: 0; text-transform: none; }
.group-head__toggle { color: var(--text-tertiary); margin-right: var(--sp-1); }

/* === Tree key/value rows === */
.kv {
  display: grid;
  grid-template-columns: 130px 1fr;
  column-gap: var(--sp-4);
  padding: 4px 10px;
  border-bottom: 1px solid var(--border-subtle);
  font: var(--fs-sm)/var(--lh-sm) var(--font-mono);
}
.kv__key   { color: var(--text-secondary); }
.kv__val   { color: var(--text-primary); word-break: break-all; }
.kv:hover  { background: var(--bg-overlay); }

.kv--mod   { background: var(--mod-bg); }
.kv--mod .kv__val::before { content: "← "; color: var(--mod-arrow); margin-right: 2px; }
.kv--add   { background: var(--add-bg); }
.kv--rem   { background: var(--rem-bg); }
.kv--placeholder { background: transparent; color: var(--text-tertiary); }
.kv--placeholder .kv__key { font-style: italic; }
.kv--placeholder .kv__val { color: var(--text-tertiary); }
.kv--reord { border-left: 4px solid var(--reord-bord); padding-left: 6px; }

/* nested indentation */
.tree__nested { border-left: 1px solid var(--border-subtle); padding-left: var(--sp-4); margin-left: var(--sp-2); }

/* === Solo body & content area === */
.solo-body { overflow: auto; padding: var(--sp-3); }
.solo-status { font-size: var(--fs-xs); color: var(--text-tertiary); padding: 6px var(--sp-3); border-bottom: 1px solid var(--border-subtle); background: var(--bg-elevated); }

/* === Mirror tree === */
.mirror-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  background: var(--border-subtle);
  overflow: auto;
  height: 100%;
}
.mirror-pane { background: var(--bg-page); padding: var(--sp-3); }

/* mirror-aligned variants for the right pane place arrow on the right */
.mirror-pane--right .kv--mod .kv__val::before { content: "→ "; color: var(--mod-arrow); }

/* === Slots / SlotBar === */
.slotbar { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-2); padding: var(--sp-2) var(--sp-3); border-bottom: 1px solid var(--border-subtle); background: var(--bg-elevated); transition: max-height 200ms ease-out; }
.slotbar--collapsed { display: flex; align-items: center; justify-content: space-between; height: 40px; }

.slot {
  border: 1.5px dashed var(--border-default);
  border-radius: var(--r-md);
  padding: var(--sp-3);
  min-height: 64px;
  font-size: var(--fs-xs);
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: var(--sp-1);
  text-align: center;
  transition: border-color 120ms;
}
.slot--full {
  border-style: solid;
  border-color: var(--border-emph);
  background: var(--bg-elevated);
  align-items: flex-start;
  justify-content: flex-start;
  text-align: left;
  flex-direction: row;
  gap: var(--sp-3);
}
.slot--dragover { border-color: var(--accent); background: var(--accent-bg); }
.slot--error { border-color: var(--rem-emph); background: var(--rem-bg); }

.slot__icon { font-size: 22px; opacity: 0.5; }
.slot__name { font: var(--fs-sm)/var(--lh-sm) var(--font-mono); color: var(--text-primary); flex: 1; word-break: break-all; }
.slot__sub  { font-size: 10px; color: var(--text-tertiary); margin-top: 2px; }
.slot__pick { padding: 4px 10px; border: 1px solid var(--border-default); border-radius: var(--r-sm); font-size: var(--fs-xs); color: var(--text-secondary); }
.slot__pick:hover { background: var(--bg-overlay); color: var(--text-primary); }
.slot__clear { color: var(--text-tertiary); font-size: 16px; padding: 0 4px; }
.slot__clear:hover { color: var(--rem-text); }

/* === Control bar === */
.controlbar { display: flex; align-items: center; gap: var(--sp-3); padding: 6px var(--sp-3); border-bottom: 1px solid var(--border-subtle); font-size: var(--fs-xs); background: var(--bg-elevated); }
.controlbar__seg { display: inline-flex; border: 1px solid var(--border-default); border-radius: var(--r-sm); overflow: hidden; }
.controlbar__seg button { padding: 4px 10px; color: var(--text-secondary); }
.controlbar__seg button[data-active="true"] { background: var(--accent-bg); color: var(--text-primary); }
.controlbar__seg button:hover { background: var(--bg-overlay); }
.controlbar__spacer { flex: 1; }
.controlbar__btn { padding: 4px 10px; border: 1px solid var(--border-default); border-radius: var(--r-sm); color: var(--text-secondary); }
.controlbar__btn:hover { background: var(--bg-overlay); color: var(--text-primary); }
.controlbar__btn[data-active="true"] { background: var(--accent-bg); color: var(--text-primary); border-color: var(--border-emph); }
.controlbar__summary { color: var(--text-tertiary); font-family: var(--font-mono); }

/* === Directory list === */
.dirlist { display: grid; grid-template-rows: auto auto 1fr; height: 100%; min-height: 0; }
.dirlist__stats { display: flex; gap: var(--sp-5); padding: var(--sp-3); background: var(--bg-elevated); border-bottom: 1px solid var(--border-subtle); }
.dirlist__stat { font-size: var(--fs-xs); color: var(--text-secondary); }
.dirlist__stat-num { font: 600 var(--fs-xl)/1 var(--font-mono); display: block; }
.dirlist__stat--mod .dirlist__stat-num { color: var(--mod-text); }
.dirlist__stat--rem .dirlist__stat-num { color: var(--rem-text); }
.dirlist__stat--add .dirlist__stat-num { color: var(--add-text); }
.dirlist__stat--eq  .dirlist__stat-num { color: var(--text-secondary); }
.dirlist__stat--total .dirlist__stat-num { color: var(--text-tertiary); }
.dirlist__chips { display: flex; gap: var(--sp-2); padding: var(--sp-2) var(--sp-3); border-bottom: 1px solid var(--border-subtle); flex-wrap: wrap; }
.dirlist__chip { padding: 4px 10px; border-radius: 999px; font-size: var(--fs-xs); border: 1px solid var(--border-default); color: var(--text-secondary); }
.dirlist__chip[data-active="true"] { background: var(--bg-overlay); border-color: var(--border-emph); color: var(--text-primary); }
.dirlist__chip--mod { color: var(--mod-text); border-color: rgba(255, 210, 63, 0.3); }
.dirlist__chip--rem { color: var(--rem-text); border-color: rgba(255, 107, 107, 0.3); }
.dirlist__chip--add { color: var(--add-text); border-color: rgba(94, 237, 144, 0.3); }
.dirlist__chip--err { color: var(--err-text); border-color: rgba(255, 124, 224, 0.3); }
.dirlist__rows { overflow: auto; }
.dirlist__row {
  display: grid;
  grid-template-columns: 16px 1fr auto 16px;
  gap: var(--sp-3);
  align-items: center;
  padding: 8px var(--sp-3);
  border-bottom: 1px solid var(--border-subtle);
  cursor: pointer;
}
.dirlist__row:hover { background: var(--bg-overlay); }
.dirlist__row--eq { opacity: 0.55; }
.dirlist__dot { width: 8px; height: 8px; border-radius: 50%; }
.dirlist__dot--mod { background: var(--mod-text); }
.dirlist__dot--rem { background: var(--rem-text); }
.dirlist__dot--add { background: var(--add-text); }
.dirlist__dot--eq  { background: var(--text-tertiary); }
.dirlist__dot--err { background: var(--err-text); }
.dirlist__name { font: var(--fs-sm)/var(--lh-sm) var(--font-mono); }
.dirlist__chev { color: var(--text-tertiary); }

/* === Welcome / Empty === */
.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--sp-3);
  color: var(--text-secondary);
}
.welcome__title { font-size: var(--fs-2xl); font-weight: 600; color: var(--text-primary); }
.welcome__hint { font-size: var(--fs-sm); color: var(--text-tertiary); }
.welcome kbd {
  font: 500 var(--fs-xs) var(--font-mono);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 3px;
  padding: 1px 6px;
  margin: 0 2px;
}

/* === Banner / errors === */
.banner { padding: 8px var(--sp-3); font-size: var(--fs-sm); }
.banner--error { background: var(--err-bg); color: var(--err-text); }

/* === Toast === */
.toast {
  position: fixed; top: 50px; left: 50%; transform: translateX(-50%);
  padding: 8px 14px; background: var(--bg-overlay); color: var(--text-primary);
  font-size: var(--fs-xs); border: 1px solid var(--border-default); border-radius: var(--r-sm);
  z-index: 100;
}

/* === JSON pre === */
.raw-json {
  font: var(--fs-sm)/var(--lh-sm) var(--font-mono);
  color: var(--text-primary);
  padding: var(--sp-3);
  white-space: pre-wrap;
  word-break: break-all;
}
```

- [ ] **Step 3.3 — 跑现有测试，确保 CSS 变更没误伤其他测试**

Run: `npm --prefix frontend run test -- --run`
Expected: 现有测试可能失败（因为旧组件类名变化）。把这些失败先记录下来，**Task 11** 删除旧组件时一并清理。但 `labels.test`、`treeModel.test` 必须 PASS。

- [ ] **Step 3.4 — 提交**

```bash
git add frontend/src/styles/tokens.css frontend/src/styles/app.css
git commit -m "feat(styles): rewrite design tokens for high-contrast dark system

Drops the old MotherDuck-inspired palette and shadow system in favor of
the §6 black-base high-contrast scheme: full color tokens, typography,
spacing, radii, and class fixtures for slot/group/kv/dirlist/mirror."
```

---

## Task 4 — 原子组件 StatusBadge + GroupHead

**Files:**
- Create: `frontend/src/components/StatusBadge.tsx`
- Create: `frontend/src/components/StatusBadge.test.tsx`
- Create: `frontend/src/components/GroupHead.tsx`
- Create: `frontend/src/components/GroupHead.test.tsx`

- [ ] **Step 4.1 — 写 StatusBadge 失败测试**

```tsx
// frontend/src/components/StatusBadge.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { StatusBadge } from './StatusBadge';

describe('StatusBadge', () => {
  it('renders label with the right kind class', () => {
    render(<StatusBadge kind="modified">12 处不同</StatusBadge>);
    const badge = screen.getByText('12 处不同');
    expect(badge.className).toContain('badge--mod');
  });

  it('handles all kinds', () => {
    const { container } = render(
      <>
        <StatusBadge kind="modified">m</StatusBadge>
        <StatusBadge kind="added">a</StatusBadge>
        <StatusBadge kind="removed">r</StatusBadge>
        <StatusBadge kind="error">e</StatusBadge>
        <StatusBadge kind="unchanged">u</StatusBadge>
        <StatusBadge kind="reordered">o</StatusBadge>
      </>,
    );
    const html = container.innerHTML;
    expect(html).toContain('badge--mod');
    expect(html).toContain('badge--add');
    expect(html).toContain('badge--rem');
    expect(html).toContain('badge--err');
    expect(html).toContain('badge--neu');
  });
});
```

- [ ] **Step 4.2 — 跑测试确认失败**

Run: `npm --prefix frontend run test -- --run StatusBadge.test`
Expected: FAIL（找不到模块）

- [ ] **Step 4.3 — 实现 `StatusBadge.tsx`**

```tsx
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
```

- [ ] **Step 4.4 — 跑测试确认通过**

Run: `npm --prefix frontend run test -- --run StatusBadge.test`
Expected: PASS

- [ ] **Step 4.5 — 写 GroupHead 失败测试**

```tsx
// frontend/src/components/GroupHead.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { GroupHead } from './GroupHead';

describe('GroupHead', () => {
  it('renders label and count', () => {
    render(<GroupHead label="停靠线路" count={5} />);
    expect(screen.getByText('停靠线路')).toBeTruthy();
    expect(screen.getByText('(5 项)')).toBeTruthy();
  });

  it('shows ▼ when open and ▶ when closed and toggles via onToggle', () => {
    const onToggle = vi.fn();
    const { rerender } = render(<GroupHead label="x" open onToggle={onToggle} />);
    expect(screen.getByRole('button')).toHaveTextContent('▼');
    fireEvent.click(screen.getByRole('button'));
    expect(onToggle).toHaveBeenCalled();
    rerender(<GroupHead label="x" open={false} onToggle={onToggle} />);
    expect(screen.getByRole('button')).toHaveTextContent('▶');
  });

  it('applies nested class when level > 0', () => {
    const { container } = render(<GroupHead label="x" level={1} />);
    expect(container.firstChild).toHaveClass('group-head--nested');
  });
});
```

- [ ] **Step 4.6 — 实现 `GroupHead.tsx`**

```tsx
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
```

- [ ] **Step 4.7 — 跑两套测试**

Run: `npm --prefix frontend run test -- --run StatusBadge.test GroupHead.test`
Expected: PASS

- [ ] **Step 4.8 — 提交**

```bash
git add frontend/src/components/StatusBadge.tsx frontend/src/components/StatusBadge.test.tsx \
        frontend/src/components/GroupHead.tsx frontend/src/components/GroupHead.test.tsx
git commit -m "feat(atoms): add StatusBadge and GroupHead shared components"
```

---

## Task 5 — SoloTree（单边树）

**Files:**
- Create: `frontend/src/components/SoloTree.tsx`
- Create: `frontend/src/components/SoloTree.test.tsx`

### 5.1 接口

```ts
import type { JsonValue } from '../lib/types';
export function SoloTree(props: { value: JsonValue }): JSX.Element;
```

折叠状态用 `useState<Set<string>>` 维护一组**已折叠**的 path（默认折叠的 path 初始就在 set 里；点击切换从 set 里增删）。

- [ ] **Step 5.1 — 写失败测试**

```tsx
// frontend/src/components/SoloTree.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { SoloTree } from './SoloTree';

const meta = {
  StopName: '翻身地铁站',
  RoadName: '创业一路',
  HasHints: false,
  Lines: [
    {
      LineName: 'B932',
      Direction: '终点A',
      RouteStops: [{ Name: '上川', Sequence: 2, BuildingType: null, RoadName: '福城路' }],
    },
  ],
};

describe('SoloTree', () => {
  it('renders Chinese labels for top-level fields', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('中文站名')).toBeTruthy();
    expect(screen.getByText('翻身地铁站')).toBeTruthy();
    expect(screen.getByText('所在道路')).toBeTruthy();
    expect(screen.getByText('创业一路')).toBeTruthy();
  });

  it('formats booleans contextually', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('含温馨提示')).toBeTruthy();
    expect(screen.getByText('否')).toBeTruthy();
  });

  it('renders array group with count and is collapsed by default', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('停靠线路')).toBeTruthy();
    expect(screen.getByText('(1 项)')).toBeTruthy();
    // collapsed by default → array-item label NOT visible
    expect(screen.queryByText(/线路 1 · B932/)).toBeNull();
  });

  it('expands array group when clicked', () => {
    render(<SoloTree value={meta as any} />);
    const arrToggle = screen.getAllByRole('button', { name: '展开' })
      .find((b) => b.textContent === '▶')!;
    fireEvent.click(arrToggle);
    expect(screen.getByText(/线路 1 · B932/)).toBeTruthy();
  });

  it('does not show diff backgrounds in solo mode', () => {
    const { container } = render(<SoloTree value={meta as any} />);
    expect(container.querySelector('.kv--mod')).toBeNull();
    expect(container.querySelector('.kv--add')).toBeNull();
    expect(container.querySelector('.kv--rem')).toBeNull();
  });
});
```

- [ ] **Step 5.2 — 跑测试确认失败**

Run: `npm --prefix frontend run test -- --run SoloTree.test`
Expected: FAIL（模块不存在）

- [ ] **Step 5.3 — 实现 `SoloTree.tsx`**

```tsx
// frontend/src/components/SoloTree.tsx
import { useMemo, useState } from 'react';
import { GroupHead } from './GroupHead';
import { buildTree } from '../lib/treeModel';
import type { GroupNode, LeafNode, TreeNode } from '../lib/treeModel';
import type { JsonValue } from '../lib/types';

export function SoloTree({ value }: { value: JsonValue }) {
  const tree = useMemo(() => buildTree(value), [value]);
  const initialClosed = useMemo(() => collectDefaultClosed(tree), [tree]);
  const [closed, setClosed] = useState<Set<string>>(initialClosed);

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
```

- [ ] **Step 5.4 — 跑测试确认通过**

Run: `npm --prefix frontend run test -- --run SoloTree.test`
Expected: PASS（5 个 case）

- [ ] **Step 5.5 — 提交**

```bash
git add frontend/src/components/SoloTree.tsx frontend/src/components/SoloTree.test.tsx
git commit -m "feat(SoloTree): single-side metadata tree with Chinese labels and folding"
```

---

## Task 6 — MirrorTree（1v1 镜像）

**Files:**
- Create: `frontend/src/components/MirrorTree.tsx`
- Create: `frontend/src/components/MirrorTree.test.tsx`

### 6.1 接口

```ts
export function MirrorTree(props: {
  left: JsonValue | null;
  right: JsonValue | null;
  diffRoot: DiffNode | null;
  highlight: boolean;       // false 时不显示状态色
  onlyDiff: boolean;        // true 时隐藏全 unchanged 子树
  leftLabel: string;
  rightLabel: string;
}): JSX.Element;
```

实现要点：
- 用 `buildMirrorRows` 得到合并行；左右 pane 渲染相同的行集，但取各自的 value。
- 同步滚动：用一个外层包裹 + 内部两栏共用一个滚动容器（CSS grid），根本上不会出现"双滚动条"问题，因为只有一个 `.mirror-grid` 在滚。这是最简的同步方案。
- 占位行：`leaf` 行 leftValue/rightValue 为 null 时该侧渲染 `kv--placeholder` 的 `— — —`。
- `onlyDiff`：递归过滤掉 `hasDiffDeep === false` 的子树；折叠状态在 `onlyDiff=true` 时强制展开（自动展开嵌套数组到差异行）。

- [ ] **Step 6.1 — 写失败测试**

```tsx
// frontend/src/components/MirrorTree.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { MirrorTree } from './MirrorTree';
import type { DiffNode } from '../lib/types';

const diff: DiffNode = {
  path: '', status: 'modified', left_value: null, right_value: null, summary: '',
  children: [
    { path: 'FrameSize', status: 'modified', left_value: '"1050x1660"', right_value: '"1200x1800"', summary: '', children: [] },
    { path: 'Hints', status: 'removed', left_value: '"x"', right_value: null, summary: '', children: [] },
  ],
};

const left = { StopName: '翻身', FrameSize: '1050x1660', Hints: 'x' };
const right = { StopName: '翻身', FrameSize: '1200x1800' };

describe('MirrorTree', () => {
  it('renders both panes with values', () => {
    render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(screen.getAllByText('翻身').length).toBe(2);
    expect(screen.getByText('1050x1660')).toBeTruthy();
    expect(screen.getByText('1200x1800')).toBeTruthy();
  });

  it('applies kv--mod class to modified rows when highlight is on', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(container.querySelectorAll('.kv--mod').length).toBeGreaterThan(0);
  });

  it('omits status classes when highlight is off', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight={false}
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(container.querySelectorAll('.kv--mod').length).toBe(0);
  });

  it('renders placeholder on the missing side for removed leaf', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    // The right pane should have a placeholder for the "Hints" row.
    const placeholders = container.querySelectorAll('.kv--placeholder');
    expect(placeholders.length).toBeGreaterThan(0);
  });

  it('hides unchanged rows when onlyDiff is true', () => {
    render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff
        leftLabel="L"
        rightLabel="R"
      />,
    );
    // StopName is unchanged → not visible
    expect(screen.queryByText('翻身')).toBeNull();
    // FrameSize is modified → still visible
    expect(screen.getByText('1050x1660')).toBeTruthy();
  });
});
```

- [ ] **Step 6.2 — 跑测试确认失败**

Run: `npm --prefix frontend run test -- --run MirrorTree.test`
Expected: FAIL

- [ ] **Step 6.3 — 实现 `MirrorTree.tsx`**

```tsx
// frontend/src/components/MirrorTree.tsx
import { useMemo, useState } from 'react';
import { GroupHead } from './GroupHead';
import { buildMirrorRows, hasDiffDeep } from '../lib/treeModel';
import type { MirrorRow } from '../lib/treeModel';
import type { DiffNode, DiffStatus, JsonValue } from '../lib/types';

const STATUS_CLASS: Record<DiffStatus, string> = {
  unchanged: '',
  modified: 'kv--mod',
  added: 'kv--add',
  removed: 'kv--rem',
  reordered: 'kv--reord',
  error: 'kv--err',
};

export function MirrorTree({
  left,
  right,
  diffRoot,
  highlight,
  onlyDiff,
  leftLabel,
  rightLabel,
}: {
  left: JsonValue | null;
  right: JsonValue | null;
  diffRoot: DiffNode | null;
  highlight: boolean;
  onlyDiff: boolean;
  leftLabel: string;
  rightLabel: string;
}) {
  const rows = useMemo(() => buildMirrorRows(left, right, diffRoot), [left, right, diffRoot]);
  const initialClosed = useMemo(() => collectDefaultClosed(rows), [rows]);
  const [closed, setClosed] = useState<Set<string>>(initialClosed);

  const toggle = (path: string) =>
    setClosed((cur) => {
      const next = new Set(cur);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  // When onlyDiff is on, force-open any subtree that contains a diff.
  const effectiveClosed = useMemo(() => {
    if (!onlyDiff) return closed;
    const next = new Set(closed);
    forceOpenIfHasDiff(rows, next);
    return next;
  }, [onlyDiff, closed, rows]);

  return (
    <div className="mirror-grid">
      <PaneHeader label={leftLabel} side="left" />
      <PaneHeader label={rightLabel} side="right" />
      <Pane
        rows={rows}
        side="left"
        closed={effectiveClosed}
        toggle={toggle}
        highlight={highlight}
        onlyDiff={onlyDiff}
      />
      <Pane
        rows={rows}
        side="right"
        closed={effectiveClosed}
        toggle={toggle}
        highlight={highlight}
        onlyDiff={onlyDiff}
      />
    </div>
  );
}

function PaneHeader({ label, side }: { label: string; side: 'left' | 'right' }) {
  return (
    <div className={`mirror-pane mirror-pane--${side}`} style={{ borderBottom: '1px solid var(--border-subtle)', padding: '6px 12px', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
      {side === 'left' ? '左 · ' : '右 · '}{label}
    </div>
  );
}

function Pane({
  rows,
  side,
  closed,
  toggle,
  highlight,
  onlyDiff,
}: {
  rows: MirrorRow[];
  side: 'left' | 'right';
  closed: Set<string>;
  toggle: (p: string) => void;
  highlight: boolean;
  onlyDiff: boolean;
}) {
  return (
    <div className={`mirror-pane mirror-pane--${side}`}>
      {rows.map((row) => (
        <RowView
          key={row.path || 'root'}
          row={row}
          side={side}
          level={0}
          closed={closed}
          toggle={toggle}
          highlight={highlight}
          onlyDiff={onlyDiff}
        />
      ))}
    </div>
  );
}

function RowView({
  row,
  side,
  level,
  closed,
  toggle,
  highlight,
  onlyDiff,
}: {
  row: MirrorRow;
  side: 'left' | 'right';
  level: number;
  closed: Set<string>;
  toggle: (p: string) => void;
  highlight: boolean;
  onlyDiff: boolean;
}) {
  if (onlyDiff && !hasDiffDeep(row)) return null;

  if (row.kind === 'leaf') {
    return <MirrorLeaf row={row} side={side} highlight={highlight} />;
  }

  // group
  if (row.variant === 'object-root') {
    return (
      <>
        {row.children?.map((c) => (
          <RowView
            key={c.path}
            row={c}
            side={side}
            level={level}
            closed={closed}
            toggle={toggle}
            highlight={highlight}
            onlyDiff={onlyDiff}
          />
        ))}
      </>
    );
  }

  const isOpen = !closed.has(row.path);
  return (
    <>
      <GroupHead
        label={row.label}
        count={row.variant === 'array' ? row.count : undefined}
        level={level}
        open={isOpen}
        onToggle={() => toggle(row.path)}
      />
      {isOpen && (
        <div className="tree__nested">
          {row.children?.map((c) => (
            <RowView
              key={c.path}
              row={c}
              side={side}
              level={level + 1}
              closed={closed}
              toggle={toggle}
              highlight={highlight}
              onlyDiff={onlyDiff}
            />
          ))}
        </div>
      )}
    </>
  );
}

function MirrorLeaf({ row, side, highlight }: { row: MirrorRow; side: 'left' | 'right'; highlight: boolean }) {
  const value = side === 'left' ? row.leftValue : row.rightValue;
  const otherValue = side === 'left' ? row.rightValue : row.leftValue;

  // placeholder when this side absent but the other side exists
  if (value === null && otherValue !== null) {
    return (
      <div className="kv kv--placeholder">
        <span className="kv__key">— — —</span>
        <span className="kv__val">仅另一侧存在</span>
      </div>
    );
  }
  if (value === null && otherValue === null) {
    // both null — render as a normal em-dash row
    return (
      <div className="kv">
        <span className="kv__key">{row.label}</span>
        <span className="kv__val">—</span>
      </div>
    );
  }

  const cls = highlight ? STATUS_CLASS[row.status] : '';
  return (
    <div className={`kv ${cls}`.trim()}>
      <span className="kv__key">{row.label}</span>
      <span className="kv__val">{value}</span>
    </div>
  );
}

function collectDefaultClosed(rows: MirrorRow[], into = new Set<string>()): Set<string> {
  for (const r of rows) {
    if (r.kind === 'group' && !r.defaultOpen && r.path) into.add(r.path);
    if (r.children) collectDefaultClosed(r.children, into);
  }
  return into;
}

function forceOpenIfHasDiff(rows: MirrorRow[], closed: Set<string>): void {
  for (const r of rows) {
    if (r.kind === 'group' && hasDiffDeep(r)) closed.delete(r.path);
    if (r.children) forceOpenIfHasDiff(r.children, closed);
  }
}
```

- [ ] **Step 6.4 — 跑测试确认通过**

Run: `npm --prefix frontend run test -- --run MirrorTree.test`
Expected: PASS

- [ ] **Step 6.5 — 提交**

```bash
git add frontend/src/components/MirrorTree.tsx frontend/src/components/MirrorTree.test.tsx
git commit -m "feat(MirrorTree): 1v1 mirrored compare with placeholder rows and only-diff filter"
```

---

## Task 7 — Slot + SlotBar

**Files:**
- Create: `frontend/src/components/Slot.tsx`
- Create: `frontend/src/components/Slot.test.tsx`
- Create: `frontend/src/components/SlotBar.tsx`
- Create: `frontend/src/components/SlotBar.test.tsx`

### 7.1 接口

```ts
export function Slot(props: {
  side: 'left' | 'right';
  mode: 'single' | 'directory';
  value: string;                   // file path / dir path
  errorMessage?: string | null;
  onPick(): void;                  // open dialog
  onChange(path: string): void;    // drag-drop or clear
}): JSX.Element;

export function SlotBar(props: {
  mode: 'single' | 'directory';
  leftValue: string;
  rightValue: string;
  collapsed: boolean;
  leftError?: string | null;
  rightError?: string | null;
  onPickLeft(): void;
  onPickRight(): void;
  onLeftChange(path: string): void;
  onRightChange(path: string): void;
  onToggleCollapsed(): void;
}): JSX.Element;
```

拖放：监听 `dragover` / `dragleave` / `drop` 标记 `dragover` 视觉态。drop 时取 `event.dataTransfer.files[0].path`（Tauri 注入了 `path`）；浏览器场景仅 `name`，不可用——本应用是 Tauri 桌面端。

- [ ] **Step 7.1 — 写 Slot 失败测试**

```tsx
// frontend/src/components/Slot.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Slot } from './Slot';

describe('Slot', () => {
  it('renders empty state with mode hint', () => {
    render(
      <Slot side="left" mode="single" value="" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText(/拖入 PNG/)).toBeTruthy();
  });

  it('renders directory mode hint', () => {
    render(
      <Slot side="right" mode="directory" value="" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText(/拖入目录/)).toBeTruthy();
  });

  it('renders filename when filled', () => {
    render(
      <Slot side="left" mode="single" value="C:/x/翻身.png" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText('翻身.png')).toBeTruthy();
  });

  it('calls onPick when 浏览 button clicked', () => {
    const onPick = vi.fn();
    render(
      <Slot side="left" mode="single" value="" onPick={onPick} onChange={() => {}} />,
    );
    fireEvent.click(screen.getByText(/浏览/));
    expect(onPick).toHaveBeenCalled();
  });

  it('calls onChange("") when clear clicked', () => {
    const onChange = vi.fn();
    render(
      <Slot side="left" mode="single" value="C:/x/y.png" onPick={() => {}} onChange={onChange} />,
    );
    fireEvent.click(screen.getByLabelText('清除'));
    expect(onChange).toHaveBeenCalledWith('');
  });

  it('shows error styling and message', () => {
    const { container } = render(
      <Slot side="left" mode="single" value="C:/x.png" errorMessage="无元数据"
            onPick={() => {}} onChange={() => {}} />,
    );
    expect(container.querySelector('.slot--error')).toBeTruthy();
    expect(screen.getByText('无元数据')).toBeTruthy();
  });
});
```

- [ ] **Step 7.2 — 实现 `Slot.tsx`**

```tsx
// frontend/src/components/Slot.tsx
import { useState } from 'react';

function basename(p: string): string {
  if (!p) return '';
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function Slot({
  side,
  mode,
  value,
  errorMessage,
  onPick,
  onChange,
}: {
  side: 'left' | 'right';
  mode: 'single' | 'directory';
  value: string;
  errorMessage?: string | null;
  onPick(): void;
  onChange(path: string): void;
}) {
  const [drag, setDrag] = useState(false);
  const filled = value.length > 0;
  const error = !!errorMessage;

  const cls = [
    'slot',
    filled && 'slot--full',
    error && 'slot--error',
    drag && 'slot--dragover',
  ].filter(Boolean).join(' ');

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDrag(true);
  };
  const onDragLeave = () => setDrag(false);
  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDrag(false);
    const file = e.dataTransfer.files?.[0];
    // Tauri injects `path` on dropped files
    const p = (file as unknown as { path?: string })?.path;
    if (p) onChange(p);
  };

  if (!filled) {
    return (
      <div className={cls} data-side={side} onDragOver={onDragOver} onDragLeave={onDragLeave} onDrop={onDrop}>
        <span className="slot__icon">{mode === 'single' ? '📄' : '📁'}</span>
        <span>{mode === 'single' ? '拖入 PNG 或' : '拖入目录或'}</span>
        <button type="button" className="slot__pick" onClick={onPick}>浏览</button>
        {error && <span className="banner banner--error">{errorMessage}</span>}
      </div>
    );
  }

  return (
    <div className={cls} data-side={side} onDragOver={onDragOver} onDragLeave={onDragLeave} onDrop={onDrop}>
      <span className="slot__icon">{mode === 'single' ? '📄' : '📁'}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div className="slot__name">{basename(value)}</div>
        <div className="slot__sub">{value}{error ? ` · ${errorMessage}` : ''}</div>
      </div>
      <button type="button" className="slot__pick" onClick={onPick}>替换</button>
      <button type="button" className="slot__clear" aria-label="清除" onClick={() => onChange('')}>×</button>
    </div>
  );
}
```

- [ ] **Step 7.3 — 跑 Slot 测试**

Run: `npm --prefix frontend run test -- --run Slot.test`
Expected: PASS

- [ ] **Step 7.4 — 写 SlotBar 失败测试**

```tsx
// frontend/src/components/SlotBar.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SlotBar } from './SlotBar';

describe('SlotBar', () => {
  const baseProps = {
    mode: 'single' as const,
    leftValue: '',
    rightValue: '',
    onPickLeft: vi.fn(),
    onPickRight: vi.fn(),
    onLeftChange: vi.fn(),
    onRightChange: vi.fn(),
    onToggleCollapsed: vi.fn(),
  };

  it('renders both slots when expanded', () => {
    const { container } = render(<SlotBar {...baseProps} collapsed={false} />);
    expect(container.querySelectorAll('.slot').length).toBe(2);
  });

  it('renders collapsed summary when collapsed', () => {
    render(
      <SlotBar
        {...baseProps}
        collapsed
        leftValue="C:/a/翻身.png"
        rightValue="C:/b/翻身.png"
      />,
    );
    expect(screen.getAllByText(/翻身\.png/).length).toBe(2);
    expect(screen.getByLabelText('展开')).toBeTruthy();
  });

  it('calls onToggleCollapsed when expand button clicked', () => {
    const onToggleCollapsed = vi.fn();
    render(
      <SlotBar
        {...baseProps}
        collapsed
        leftValue="x" rightValue="y"
        onToggleCollapsed={onToggleCollapsed}
      />,
    );
    fireEvent.click(screen.getByLabelText('展开'));
    expect(onToggleCollapsed).toHaveBeenCalled();
  });
});
```

- [ ] **Step 7.5 — 实现 `SlotBar.tsx`**

```tsx
// frontend/src/components/SlotBar.tsx
import { Slot } from './Slot';

function basename(p: string): string {
  if (!p) return '';
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function SlotBar({
  mode,
  leftValue,
  rightValue,
  collapsed,
  leftError,
  rightError,
  onPickLeft,
  onPickRight,
  onLeftChange,
  onRightChange,
  onToggleCollapsed,
}: {
  mode: 'single' | 'directory';
  leftValue: string;
  rightValue: string;
  collapsed: boolean;
  leftError?: string | null;
  rightError?: string | null;
  onPickLeft(): void;
  onPickRight(): void;
  onLeftChange(path: string): void;
  onRightChange(path: string): void;
  onToggleCollapsed(): void;
}) {
  if (collapsed) {
    return (
      <div className="slotbar slotbar--collapsed">
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
          左 · {basename(leftValue)}
        </span>
        <span style={{ color: 'var(--text-tertiary)' }}>⇄</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
          右 · {basename(rightValue)}
        </span>
        <button
          type="button"
          aria-label="展开"
          className="slot__pick"
          onClick={onToggleCollapsed}
        >
          ▼
        </button>
      </div>
    );
  }

  return (
    <div className="slotbar">
      <Slot side="left"  mode={mode} value={leftValue}  errorMessage={leftError}  onPick={onPickLeft}  onChange={onLeftChange} />
      <Slot side="right" mode={mode} value={rightValue} errorMessage={rightError} onPick={onPickRight} onChange={onRightChange} />
    </div>
  );
}
```

- [ ] **Step 7.6 — 跑两套测试**

Run: `npm --prefix frontend run test -- --run Slot.test SlotBar.test`
Expected: PASS

- [ ] **Step 7.7 — 提交**

```bash
git add frontend/src/components/Slot.tsx frontend/src/components/Slot.test.tsx \
        frontend/src/components/SlotBar.tsx frontend/src/components/SlotBar.test.tsx
git commit -m "feat(slots): drop-zone Slot and collapsible SlotBar"
```

---

## Task 8 — useWorkbench 改造

**Files:**
- Modify: `frontend/src/features/workbench/useWorkbench.ts`
- Modify: `frontend/src/features/workbench/useWorkbench.test.tsx`

### 8.1 改造目标

新增：
- `view: 'welcome' | 'solo' | 'mirror' | 'directory-overview'`（移除 `pair-comparison`）
- `soloSide: 'left' | 'right' | null`、`soloResult: SideInspection | null`
- `slotBarCollapsed: boolean` 自动管理
- `onlyDiff: boolean` + `setOnlyDiff`
- `runAuto()`：根据 `mode` + 槽位填充情况自动决定调用 `compareSingle` / `inspectSingle` / `scanDirectory` / 渲染欢迎页
- `tryDropPath(side, path)`：拖入路径，自动模式判断（PNG → single；目录 → directory，并切模式 + 通知 toast）；暂时仅按扩展名判断（`.png` → single；其它 → directory）
- 键盘快捷键 hook `useKeyboard()` 把 `Ctrl+O` / `Ctrl+Shift+O` / `Ctrl+Enter` / `Esc` / `[` / `]` / `1`/`2`/`3` / `D` 绑到 window

### 8.2 决策：模式判断仅靠扩展名

理由：拖放进来的路径来自 Tauri，没有同步的 stat。`.png` 文件 → single；其它（含目录、未知后缀）→ directory。误判时用户可手动切回。

- [ ] **Step 8.1 — 写新增测试用例（`useWorkbench.test.tsx` 末尾追加）**

```tsx
// 追加到 frontend/src/features/workbench/useWorkbench.test.tsx 末尾，
// 在 `describe('useWorkbench', () => { ... })` 块内。

  it('view starts as welcome when both inputs empty', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    expect(result.current.view).toBe('welcome');
  });

  it('runAuto: single mode + only left filled → solo (left)', async () => {
    const sideInspection = {
      side: 'left' as const, file_path: '/a.png', file_name: 'a.png',
      raw_json: '{"k":1}', metadata: { k: 1 }, error: null,
    };
    const api = makeApi({ inspectSingle: vi.fn().mockResolvedValue(sideInspection) });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(api.inspectSingle).toHaveBeenCalledWith('/a.png', 'left');
    expect(result.current.view).toBe('solo');
    expect(result.current.soloSide).toBe('left');
    expect(result.current.soloResult).toBe(sideInspection);
  });

  it('runAuto: single mode + both filled → mirror', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('mirror');
    expect(result.current.pairResult).toBe(mockInspection);
  });

  it('runAuto: directory mode + both filled → directory-overview', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/L'); result.current.setRightInput('/R'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('directory-overview');
  });

  it('tryDropPath: dropping .png while in directory mode auto-switches to single', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.tryDropPath('left', '/some/file.png'); });
    expect(result.current.mode).toBe('single');
    expect(result.current.leftInput).toBe('/some/file.png');
    expect(result.current.toast).toMatch(/已切换到单文件模式/);
  });

  it('tryDropPath: dropping non-png while in single mode auto-switches to directory', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.tryDropPath('right', '/some/folder'); });
    expect(result.current.mode).toBe('directory');
    expect(result.current.rightInput).toBe('/some/folder');
    expect(result.current.toast).toMatch(/已切换到目录模式/);
  });

  it('slot bar collapses after first successful analysis with both filled', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    expect(result.current.slotBarCollapsed).toBe(false);
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.slotBarCollapsed).toBe(true);
  });

  it('navigateToPair: left_only item → solo left', async () => {
    const onlyLeft = {
      id: 'L', kind: 'left_only' as const, label: 'x.png',
      left_path: '/L/x.png', right_path: null, difference_count: 0,
      match_strategy: 'file_name' as const, message: null,
    };
    const sideInspection = {
      side: 'left' as const, file_path: '/L/x.png', file_name: 'x.png',
      raw_json: null, metadata: { Foo: 'bar' }, error: null,
    };
    const api = makeApi({ inspectSingle: vi.fn().mockResolvedValue(sideInspection) });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => { await result.current.navigateToPair(onlyLeft as any); });
    expect(api.inspectSingle).toHaveBeenCalledWith('/L/x.png', 'left');
    expect(result.current.view).toBe('solo');
    expect(result.current.soloSide).toBe('left');
  });
```

也更新文件顶部的 mock：

```tsx
// 修改 frontend/src/features/workbench/useWorkbench.test.tsx 顶部的 makeApi mock：
// 原来 inspectSingle: vi.fn().mockResolvedValue({})
// 改为下面的“每次返回一个标准 SideInspection”——避免覆盖默认时漏 case。
function makeApi(overrides: Partial<WorkbenchApi> = {}): WorkbenchApi {
  return {
    compareSingle: vi.fn().mockResolvedValue(mockInspection),
    scanDirectory: vi.fn().mockResolvedValue(mockSummary),
    inspectSingle: vi.fn().mockResolvedValue({
      side: 'left',
      file_path: '/x.png',
      file_name: 'x.png',
      raw_json: null,
      metadata: null,
      error: null,
    }),
    ...overrides,
  };
}
```

也修一处旧测试以匹配新的 view 命名（把 `'pair-comparison'` 改为 `'mirror'`）：

```tsx
// 在 frontend/src/features/workbench/useWorkbench.test.tsx：
// 把 `expect(result.current.view).toBe('pair-comparison')` 全部替换为 `'mirror'`。
// 把 `expect(result.current.view).toBe('directory-overview')` 保持不变。
```

- [ ] **Step 8.2 — 跑测试确认失败（接口不存在）**

Run: `npm --prefix frontend run test -- --run useWorkbench.test`
Expected: FAIL

- [ ] **Step 8.3 — 重写 `useWorkbench.ts`**

```tsx
// frontend/src/features/workbench/useWorkbench.ts
import { useEffect, useState } from 'react';
import { workbenchApi } from '../../lib/api';
import type {
  BatchListItem,
  BatchListItemKind,
  DirectorySummary,
  PairInspection,
  SideInspection,
  WorkbenchMode,
} from '../../lib/types';
import type { WorkbenchApi } from '../../lib/api';

export type AppView = 'welcome' | 'solo' | 'mirror' | 'directory-overview';
export type ViewMode = 'tree' | 'json' | 'image';
export type ActiveFilter = 'all' | BatchListItemKind;
export type Side = 'left' | 'right';

export interface DirectoryContext {
  index: number;
  totalDifferent: number;
}

type ModeInputs = Record<WorkbenchMode, { left: string; right: string }>;

function emptyInputs(): ModeInputs {
  return { single: { left: '', right: '' }, directory: { left: '', right: '' } };
}

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isPngPath(p: string): boolean {
  return /\.png$/i.test(p);
}

export function useWorkbench(api: WorkbenchApi = workbenchApi) {
  const [mode, setModeState] = useState<WorkbenchMode>('single');
  const [view, setView] = useState<AppView>('welcome');
  const [inputsByMode, setInputsByMode] = useState<ModeInputs>(emptyInputs);

  const [directorySummary, setDirectorySummary] = useState<DirectorySummary | null>(null);
  const [activeFilter, setActiveFilter] = useState<ActiveFilter>('different');
  const [pairResult, setPairResult] = useState<PairInspection | null>(null);
  const [soloResult, setSoloResult] = useState<SideInspection | null>(null);
  const [soloSide, setSoloSide] = useState<Side | null>(null);
  const [directoryContext, setDirectoryContext] = useState<DirectoryContext | null>(null);

  const [viewMode, setViewMode] = useState<ViewMode>('tree');
  const [diffHighlight, setDiffHighlight] = useState(true);
  const [onlyDiff, setOnlyDiff] = useState(false);

  const [slotBarCollapsed, setSlotBarCollapsed] = useState(false);

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const activeInputs = inputsByMode[mode];

  function setMode(nextMode: WorkbenchMode) {
    setModeState(nextMode);
    setView('welcome');
    setDirectorySummary(null);
    setPairResult(null);
    setSoloResult(null);
    setSoloSide(null);
    setDirectoryContext(null);
    setActiveFilter('different');
    setError(null);
    setSlotBarCollapsed(false);
  }

  function setLeftInput(value: string) {
    setInputsByMode((cur) => ({ ...cur, [mode]: { ...cur[mode], left: value } }));
    setSlotBarCollapsed(false);
  }
  function setRightInput(value: string) {
    setInputsByMode((cur) => ({ ...cur, [mode]: { ...cur[mode], right: value } }));
    setSlotBarCollapsed(false);
  }

  function tryDropPath(side: Side, path: string) {
    const wantsSingle = isPngPath(path);
    const targetMode: WorkbenchMode = wantsSingle ? 'single' : 'directory';
    if (targetMode !== mode) {
      setModeState(targetMode);
      setView('welcome');
      setPairResult(null);
      setSoloResult(null);
      setSoloSide(null);
      setDirectorySummary(null);
      setDirectoryContext(null);
      setError(null);
      // Set the dropped value into the new mode's slot, leaving the other slot empty.
      setInputsByMode(() => ({
        single: { left: '', right: '' },
        directory: { left: '', right: '' },
        [targetMode]: { left: side === 'left' ? path : '', right: side === 'right' ? path : '' },
      } as ModeInputs));
      flashToast(targetMode === 'single' ? '已切换到单文件模式' : '已切换到目录模式');
      return;
    }
    if (side === 'left') setLeftInput(path); else setRightInput(path);
  }

  function flashToast(msg: string) {
    setToast(msg);
    setTimeout(() => setToast(null), 2200);
  }

  function toggleDiffHighlight() { setDiffHighlight((v) => !v); }
  function toggleOnlyDiff() { setOnlyDiff((v) => !v); }
  function toggleSlotBarCollapsed() { setSlotBarCollapsed((v) => !v); }

  function goBackToDirectory() {
    setView('directory-overview');
    setPairResult(null);
    setSoloResult(null);
    setSoloSide(null);
    setDirectoryContext(null);
  }

  async function navigateToPair(item: BatchListItem) {
    setIsLoading(true);
    setError(null);

    const differentItems = (directorySummary?.items ?? []).filter((i) => i.kind === 'different');
    const diffIndex = differentItems.findIndex((i) => i.id === item.id);
    setDirectoryContext(
      diffIndex >= 0 ? { index: diffIndex + 1, totalDifferent: differentItems.length } : null,
    );

    try {
      if (item.kind === 'left_only' && item.left_path) {
        const result = await api.inspectSingle(item.left_path, 'left');
        setSoloResult(result);
        setSoloSide('left');
        setView('solo');
        setViewMode('tree');
        return;
      }
      if (item.kind === 'right_only' && item.right_path) {
        const result = await api.inspectSingle(item.right_path, 'right');
        setSoloResult(result);
        setSoloSide('right');
        setView('solo');
        setViewMode('tree');
        return;
      }
      if (item.left_path && item.right_path) {
        const result = await api.compareSingle(item.left_path, item.right_path);
        setPairResult(result);
        setView('mirror');
        setViewMode('tree');
        return;
      }
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function runAuto() {
    setIsLoading(true);
    setError(null);
    try {
      const { left, right } = activeInputs;
      if (mode === 'single') {
        if (left && right) {
          const result = await api.compareSingle(left, right);
          setPairResult(result);
          setSoloResult(null); setSoloSide(null);
          setDirectorySummary(null);
          setDirectoryContext(null);
          setView('mirror');
          setViewMode('tree');
          setSlotBarCollapsed(true);
        } else if (left || right) {
          const target = left || right;
          const side: Side = left ? 'left' : 'right';
          const result = await api.inspectSingle(target, side);
          setSoloResult(result);
          setSoloSide(side);
          setPairResult(null);
          setDirectorySummary(null);
          setDirectoryContext(null);
          setView('solo');
          setViewMode('tree');
          setSlotBarCollapsed(true);
        } else {
          setView('welcome');
          setSlotBarCollapsed(false);
        }
        return;
      }
      // directory mode
      if (left && right) {
        const summary = await api.scanDirectory(left, right);
        setDirectorySummary(summary);
        setPairResult(null);
        setSoloResult(null); setSoloSide(null);
        setDirectoryContext(null);
        setActiveFilter('different');
        setView('directory-overview');
        setSlotBarCollapsed(true);
      } else {
        setView('welcome');
        setSlotBarCollapsed(false);
      }
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  // Backwards-compatible alias used by older tests.
  const runCompare = runAuto;

  const filteredItems =
    activeFilter === 'all'
      ? (directorySummary?.items ?? [])
      : (directorySummary?.items ?? []).filter((i) => i.kind === activeFilter);

  // Keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;
      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === 'o') {
        e.preventDefault();
        document.dispatchEvent(new CustomEvent('wb:pickLeft'));
      } else if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'o') {
        e.preventDefault();
        document.dispatchEvent(new CustomEvent('wb:pickRight'));
      } else if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        void runAuto();
      } else if (e.key === 'Escape') {
        if (view === 'mirror' && directoryContext) {
          goBackToDirectory();
        } else {
          setLeftInput(''); setRightInput('');
        }
      } else if (e.key === '1') setViewMode('tree');
      else if (e.key === '2') setViewMode('json');
      else if (e.key === '3') setViewMode('image');
      else if (e.key.toLowerCase() === 'd' && view === 'mirror') toggleDiffHighlight();
      else if ((e.key === '[' || e.key === ']') && directoryContext) {
        e.preventDefault();
        const items = (directorySummary?.items ?? []).filter((i) => i.kind === 'different');
        const cur = directoryContext.index - 1;
        const next = e.key === ']' ? Math.min(cur + 1, items.length - 1) : Math.max(cur - 1, 0);
        if (items[next]) void navigateToPair(items[next]);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  return {
    mode,
    view,
    leftInput: activeInputs.left,
    rightInput: activeInputs.right,
    directorySummary,
    filteredItems,
    activeFilter,
    pairResult,
    soloResult,
    soloSide,
    directoryContext,
    viewMode,
    diffHighlight,
    onlyDiff,
    slotBarCollapsed,
    isLoading,
    error,
    toast,
    setMode,
    setLeftInput,
    setRightInput,
    setActiveFilter,
    setViewMode,
    toggleDiffHighlight,
    toggleOnlyDiff,
    toggleSlotBarCollapsed,
    goBackToDirectory,
    navigateToPair,
    runAuto,
    runCompare,
    tryDropPath,
  };
}
```

- [ ] **Step 8.4 — 跑测试**

Run: `npm --prefix frontend run test -- --run useWorkbench.test`
Expected: PASS

- [ ] **Step 8.5 — 提交**

```bash
git add frontend/src/features/workbench/useWorkbench.ts \
        frontend/src/features/workbench/useWorkbench.test.tsx
git commit -m "feat(workbench): solo view, drop-to-mode, runAuto, slot collapse, shortcuts"
```

---

## Task 9 — DirectoryList

**Files:**
- Create: `frontend/src/components/DirectoryList.tsx`
- Create: `frontend/src/components/DirectoryList.test.tsx`

### 9.1 接口

```ts
export function DirectoryList(props: {
  summary: DirectorySummary;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  onFilter(f: ActiveFilter): void;
  onSelect(item: BatchListItem): void;
}): JSX.Element;
```

- [ ] **Step 9.1 — 写失败测试**

```tsx
// frontend/src/components/DirectoryList.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { DirectoryList } from './DirectoryList';
import type { DirectorySummary, BatchListItem } from '../lib/types';

const summary: DirectorySummary = {
  counts: { identical: 1, different: 2, left_only: 1, right_only: 0, error: 0 },
  items: [
    { id: '1', kind: 'different', label: 'a.png', left_path: '/L/a.png', right_path: '/R/a.png', difference_count: 5, match_strategy: 'file_name', message: null },
    { id: '2', kind: 'different', label: 'b.png', left_path: '/L/b.png', right_path: '/R/b.png', difference_count: 1, match_strategy: 'file_name', message: null },
    { id: '3', kind: 'left_only', label: 'c.png', left_path: '/L/c.png', right_path: null, difference_count: 0, match_strategy: 'file_name', message: null },
    { id: '4', kind: 'identical', label: 'd.png', left_path: '/L/d.png', right_path: '/R/d.png', difference_count: 0, match_strategy: 'file_name', message: null },
  ],
};

describe('DirectoryList', () => {
  it('renders stat counts', () => {
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={() => {}} onSelect={() => {}} />,
    );
    expect(screen.getByText('2')).toBeTruthy();
    expect(screen.getByText('不一致')).toBeTruthy();
    expect(screen.getByText('仅左')).toBeTruthy();
  });

  it('renders rows for each filtered item', () => {
    const filtered = summary.items.filter((i) => i.kind === 'different');
    render(
      <DirectoryList summary={summary} filteredItems={filtered}
        activeFilter="different" onFilter={() => {}} onSelect={() => {}} />,
    );
    expect(screen.getByText('a.png')).toBeTruthy();
    expect(screen.getByText('b.png')).toBeTruthy();
    expect(screen.queryByText('d.png')).toBeNull();
    expect(screen.getByText('5 处不同')).toBeTruthy();
  });

  it('clicking a row calls onSelect with that item', () => {
    const onSelect = vi.fn();
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={() => {}} onSelect={onSelect} />,
    );
    fireEvent.click(screen.getByText('a.png').closest('.dirlist__row')!);
    expect(onSelect).toHaveBeenCalledWith(summary.items[0]);
  });

  it('clicking a chip calls onFilter', () => {
    const onFilter = vi.fn();
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={onFilter} onSelect={() => {}} />,
    );
    fireEvent.click(screen.getByText(/^不一致 2$/));
    expect(onFilter).toHaveBeenCalledWith('different');
  });
});
```

- [ ] **Step 9.2 — 实现 `DirectoryList.tsx`**

```tsx
// frontend/src/components/DirectoryList.tsx
import type { ActiveFilter } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary } from '../lib/types';

const KIND_TO_DOT: Record<BatchListItemKind, string> = {
  different: 'mod',
  identical: 'eq',
  left_only: 'rem',
  right_only: 'add',
  error: 'err',
};

const KIND_BADGE: Record<BatchListItemKind, (count: number) => string> = {
  different: (n) => `${n} 处不同`,
  identical: () => '一致',
  left_only: () => '仅左侧',
  right_only: () => '仅右侧',
  error: () => '错误',
};

const CHIPS: { id: ActiveFilter; label: (counts: DirectorySummary['counts']) => string; chipClass?: string }[] = [
  { id: 'all', label: (c) => `全部 ${c.identical + c.different + c.left_only + c.right_only + c.error}` },
  { id: 'different', label: (c) => `不一致 ${c.different}`, chipClass: 'dirlist__chip--mod' },
  { id: 'left_only', label: (c) => `仅左 ${c.left_only}`, chipClass: 'dirlist__chip--rem' },
  { id: 'right_only', label: (c) => `仅右 ${c.right_only}`, chipClass: 'dirlist__chip--add' },
  { id: 'identical', label: (c) => `一致 ${c.identical}` },
  { id: 'error', label: (c) => `错误 ${c.error}`, chipClass: 'dirlist__chip--err' },
];

export function DirectoryList({
  summary,
  filteredItems,
  activeFilter,
  onFilter,
  onSelect,
}: {
  summary: DirectorySummary;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  onFilter(f: ActiveFilter): void;
  onSelect(item: BatchListItem): void;
}) {
  const c = summary.counts;
  const total = c.identical + c.different + c.left_only + c.right_only + c.error;

  return (
    <div className="dirlist">
      <div className="dirlist__stats">
        <div className="dirlist__stat dirlist__stat--mod">
          <span className="dirlist__stat-num">{c.different}</span>不一致
        </div>
        <div className="dirlist__stat dirlist__stat--rem">
          <span className="dirlist__stat-num">{c.left_only}</span>仅左
        </div>
        <div className="dirlist__stat dirlist__stat--add">
          <span className="dirlist__stat-num">{c.right_only}</span>仅右
        </div>
        <div className="dirlist__stat dirlist__stat--eq">
          <span className="dirlist__stat-num">{c.identical}</span>一致
        </div>
        <div className="dirlist__stat dirlist__stat--total">
          <span className="dirlist__stat-num">{total}</span>总计
        </div>
      </div>

      <div className="dirlist__chips">
        {CHIPS.map((chip) => (
          <button
            key={chip.id}
            type="button"
            className={`dirlist__chip ${chip.chipClass ?? ''}`}
            data-active={activeFilter === chip.id}
            onClick={() => onFilter(chip.id)}
          >
            {chip.label(c)}
          </button>
        ))}
      </div>

      <div className="dirlist__rows">
        {filteredItems.map((item) => (
          <div
            key={item.id}
            className={`dirlist__row${item.kind === 'identical' ? ' dirlist__row--eq' : ''}`}
            onClick={() => onSelect(item)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(item); }
            }}
          >
            <span className={`dirlist__dot dirlist__dot--${KIND_TO_DOT[item.kind]}`} />
            <span className="dirlist__name">{item.label}</span>
            <span className={`badge badge--${badgeKindFor(item.kind)}`}>{KIND_BADGE[item.kind](item.difference_count)}</span>
            <span className="dirlist__chev">›</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function badgeKindFor(k: BatchListItemKind): 'mod' | 'add' | 'rem' | 'err' | 'neu' {
  switch (k) {
    case 'different': return 'mod';
    case 'left_only': return 'rem';
    case 'right_only': return 'add';
    case 'error': return 'err';
    default: return 'neu';
  }
}
```

- [ ] **Step 9.3 — 跑测试**

Run: `npm --prefix frontend run test -- --run DirectoryList.test`
Expected: PASS

- [ ] **Step 9.4 — 提交**

```bash
git add frontend/src/components/DirectoryList.tsx frontend/src/components/DirectoryList.test.tsx
git commit -m "feat(DirectoryList): rewrite directory overview as stats + chips + list"
```

---

## Task 10 — App.tsx 装配 + 控件条 + 视图切换

**Files:**
- Rewrite: `frontend/src/App.tsx`
- Update: `frontend/src/App.test.tsx`

App 新结构：

```
<div class="app-shell">
  <header class="topbar">…</header>
  <SlotBar … />
  <ControlBar … />        ← 视图模式 / 高亮 / 仅看不同 / 摘要
  <main>                   ← 根据 view 渲染：
    welcome | solo | mirror | directory-overview
  </main>
  {toast && <div class="toast">{toast}</div>}
</div>
```

- [ ] **Step 10.1 — 重写 `App.tsx`**

```tsx
// frontend/src/App.tsx
import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { SlotBar } from './components/SlotBar';
import { SoloTree } from './components/SoloTree';
import { MirrorTree } from './components/MirrorTree';
import { DirectoryList } from './components/DirectoryList';
import { useWorkbench } from './features/workbench/useWorkbench';

const win = getCurrentWindow();

async function pickPath(directory: boolean): Promise<string> {
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: 'PNG', extensions: ['png'] }],
  });
  return typeof selected === 'string' ? selected : '';
}

export default function App() {
  const wb = useWorkbench();
  const isDir = wb.mode === 'directory';

  const handlePickLeft = async () => {
    const p = await pickPath(isDir);
    if (p) wb.setLeftInput(p);
  };
  const handlePickRight = async () => {
    const p = await pickPath(isDir);
    if (p) wb.setRightInput(p);
  };

  // Listen to keyboard-driven pick events from useWorkbench
  useEffect(() => {
    const onL = () => void handlePickLeft();
    const onR = () => void handlePickRight();
    document.addEventListener('wb:pickLeft', onL);
    document.addEventListener('wb:pickRight', onR);
    return () => {
      document.removeEventListener('wb:pickLeft', onL);
      document.removeEventListener('wb:pickRight', onR);
    };
  });

  // Auto-run on input change
  useEffect(() => {
    if (wb.leftInput || wb.rightInput) void wb.runAuto();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wb.leftInput, wb.rightInput, wb.mode]);

  const showModeToggle = wb.view === 'welcome' || wb.view === 'directory-overview' || wb.directoryContext === null;
  const showSlotBar = wb.view !== 'mirror' || !wb.directoryContext;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="topbar-left" data-tauri-drag-region>
          <img className="brand-icon" src="/app-icon.png" alt="" draggable={false} />
          <span className="brand">PNG ⌁ Compare</span>

          {showModeToggle && (
            <>
              <div className="topbar-vsep" />
              <div className="mode-toggle" role="group" aria-label="模式">
                <button
                  type="button"
                  className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
                  onClick={() => wb.setMode('single')}
                >单文件</button>
                <button
                  type="button"
                  className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
                  onClick={() => wb.setMode('directory')}
                >目录</button>
              </div>
            </>
          )}

          {wb.directoryContext && wb.view !== 'directory-overview' && (
            <>
              <div className="topbar-vsep" />
              <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
                ← 返回目录
              </button>
            </>
          )}
        </div>

        <div className="topbar-center" data-tauri-drag-region>
          {wb.view === 'mirror' && wb.pairResult && (
            <span>{wb.pairResult.left.file_name}</span>
          )}
          {wb.view === 'solo' && wb.soloResult && (
            <span>{wb.soloResult.file_name}</span>
          )}
          {wb.directoryContext && (
            <span>{wb.directoryContext.index} / {wb.directoryContext.totalDifferent} 处不一致</span>
          )}
        </div>

        <div className="topbar-right" data-tauri-drag-region>
          <div className="win-controls">
            <button type="button" className="win-btn" onClick={() => void win.minimize()} aria-label="最小化">─</button>
            <button type="button" className="win-btn" onClick={() => void win.toggleMaximize()} aria-label="最大化">□</button>
            <button type="button" className="win-btn win-btn--close" onClick={() => void win.close()} aria-label="关闭">✕</button>
          </div>
        </div>
      </header>

      {showSlotBar && (
        <SlotBar
          mode={wb.mode}
          leftValue={wb.leftInput}
          rightValue={wb.rightInput}
          collapsed={wb.slotBarCollapsed}
          onPickLeft={() => void handlePickLeft()}
          onPickRight={() => void handlePickRight()}
          onLeftChange={(p) => wb.tryDropPath('left', p)}
          onRightChange={(p) => wb.tryDropPath('right', p)}
          onToggleCollapsed={wb.toggleSlotBarCollapsed}
        />
      )}

      <ControlBar wb={wb} />

      {wb.error && <div className="banner banner--error">{wb.error}</div>}

      <main style={{ overflow: 'hidden' }}>
        {wb.view === 'welcome' && <Welcome mode={wb.mode} />}

        {wb.view === 'solo' && wb.soloResult?.metadata && (
          <SoloTreeFrame name={wb.soloResult.file_name} side={wb.soloSide!}>
            <SoloTree value={wb.soloResult.metadata} />
          </SoloTreeFrame>
        )}
        {wb.view === 'solo' && wb.soloResult && !wb.soloResult.metadata && (
          <div className="banner banner--error">该文件不含嵌入式元数据。</div>
        )}

        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'tree' && (
          <MirrorTree
            left={wb.pairResult.left.metadata}
            right={wb.pairResult.right.metadata}
            diffRoot={wb.pairResult.diff_root}
            highlight={wb.diffHighlight}
            onlyDiff={wb.onlyDiff}
            leftLabel={wb.pairResult.left.file_name}
            rightLabel={wb.pairResult.right.file_name}
          />
        )}
        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'json' && (
          <RawJsonSplit left={wb.pairResult.left.raw_json} right={wb.pairResult.right.raw_json} />
        )}
        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'image' && (
          <ImageSplit
            leftPath={wb.pairResult.left.file_path}
            rightPath={wb.pairResult.right.file_path}
            leftName={wb.pairResult.left.file_name}
            rightName={wb.pairResult.right.file_name}
          />
        )}

        {wb.view === 'directory-overview' && wb.directorySummary && (
          <DirectoryList
            summary={wb.directorySummary}
            filteredItems={wb.filteredItems}
            activeFilter={wb.activeFilter}
            onFilter={wb.setActiveFilter}
            onSelect={(item) => void wb.navigateToPair(item)}
          />
        )}
      </main>

      {wb.toast && <div className="toast">{wb.toast}</div>}
    </div>
  );
}

function Welcome({ mode }: { mode: 'single' | 'directory' }) {
  return (
    <div className="welcome">
      <div className="welcome__title">PNG ⌁ Compare</div>
      <div className="welcome__hint">
        拖入 {mode === 'single' ? 'PNG 文件' : '文件夹'}（左右各一个），或按
        <kbd>Ctrl+O</kbd> / <kbd>Ctrl+Shift+O</kbd> 选择
      </div>
      <div className="welcome__hint">
        快捷键：<kbd>Ctrl+Enter</kbd> 重新分析 · <kbd>1</kbd>/<kbd>2</kbd>/<kbd>3</kbd> 切换视图 · <kbd>D</kbd> 切换差异高亮
      </div>
    </div>
  );
}

function ControlBar({ wb }: { wb: ReturnType<typeof useWorkbench> }) {
  if (wb.view === 'welcome' || wb.view === 'directory-overview') return null;
  const total = wb.pairResult ? wb.pairResult.diff_summary : null;
  return (
    <div className="controlbar">
      <div className="controlbar__seg" role="group" aria-label="视图模式">
        <button data-active={wb.viewMode === 'tree'} onClick={() => wb.setViewMode('tree')}>树</button>
        <button data-active={wb.viewMode === 'json'} onClick={() => wb.setViewMode('json')}>JSON</button>
        <button data-active={wb.viewMode === 'image'} onClick={() => wb.setViewMode('image')}>图片</button>
      </div>
      {wb.view === 'mirror' && (
        <>
          <button className="controlbar__btn" data-active={wb.diffHighlight} onClick={wb.toggleDiffHighlight}>高亮差异</button>
          <button className="controlbar__btn" data-active={wb.onlyDiff} onClick={wb.toggleOnlyDiff}>仅看不同</button>
          <span className="controlbar__spacer" />
          {total && (
            <span className="controlbar__summary">
              {total.modified} 处不同 · {total.added} 仅右 · {total.removed} 仅左 · {total.reordered} 顺序不同
            </span>
          )}
        </>
      )}
      {wb.view === 'solo' && (
        <span className="controlbar__summary">仅查看 {wb.soloSide === 'left' ? '左' : '右'} · {wb.soloResult?.file_name}</span>
      )}
    </div>
  );
}

function SoloTreeFrame({ side, name, children }: { side: 'left' | 'right'; name: string; children: React.ReactNode }) {
  return (
    <>
      <div className="solo-status">仅查看 {side === 'left' ? '左' : '右'} · {name}</div>
      {children}
    </>
  );
}

function RawJsonSplit({ left, right }: { left: string | null; right: string | null }) {
  return (
    <div className="mirror-grid">
      <pre className="raw-json">{format(left)}</pre>
      <pre className="raw-json">{format(right)}</pre>
    </div>
  );
}

function format(raw: string | null): string {
  if (!raw) return '— 无 JSON —';
  try { return JSON.stringify(JSON.parse(raw), null, 2); } catch { return raw; }
}

function ImageSplit({ leftPath, rightPath, leftName, rightName }: { leftPath: string; rightPath: string; leftName: string; rightName: string; }) {
  return (
    <div className="mirror-grid">
      <ImagePane path={leftPath} name={leftName} />
      <ImagePane path={rightPath} name={rightName} />
    </div>
  );
}

function ImagePane({ path, name }: { path: string; name: string }) {
  const url = `asset://localhost/${path.replace(/\\/g, '/').split('/').map(encodeURIComponent).join('/')}`;
  return (
    <div style={{ padding: 12, display: 'flex', flexDirection: 'column', gap: 8 }}>
      <img src={url} alt={name} style={{ maxWidth: '100%', maxHeight: 'calc(100vh - 200px)', objectFit: 'contain' }}
           onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
      <button type="button" className="controlbar__btn" onClick={() => void openPath(path)}>打开原文件 ↗</button>
    </div>
  );
}
```

- [ ] **Step 10.2 — 更新 `App.test.tsx`**

```tsx
// frontend/src/App.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import App from './App';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(), onCloseRequested: vi.fn(),
  })),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openPath: vi.fn() }));
vi.mock('./lib/api', () => ({
  workbenchApi: {
    compareSingle: vi.fn(),
    scanDirectory: vi.fn(),
    inspectSingle: vi.fn(),
  },
}));

describe('App', () => {
  it('renders brand name', () => {
    render(<App />);
    expect(screen.getByText(/PNG.*Compare/i)).toBeTruthy();
  });

  it('renders welcome screen on first load', () => {
    render(<App />);
    expect(screen.getByText(/拖入/)).toBeTruthy();
  });

  it('renders mode toggle buttons', () => {
    render(<App />);
    expect(screen.getByText('单文件')).toBeTruthy();
    expect(screen.getByText('目录')).toBeTruthy();
  });
});
```

- [ ] **Step 10.3 — 跑测试**

Run: `npm --prefix frontend run test -- --run App.test`
Expected: PASS（3 个 case）

- [ ] **Step 10.4 — 跑全部测试，看现在失败的是什么**

Run: `npm --prefix frontend run test -- --run`
Expected: 仅 `workbench-sync.test.tsx` 与一些以 PairComparison/DirectoryOverview 为测试目标的旧 case 还在失败。labels / treeModel / Status / GroupHead / SoloTree / MirrorTree / Slot / SlotBar / DirectoryList / useWorkbench / App / build-config 全部 PASS。

- [ ] **Step 10.5 — 提交**

```bash
git add frontend/src/App.tsx frontend/src/App.test.tsx
git commit -m "feat(app): wire new components, top bar, control bar, welcome and view switching"
```

---

## Task 11 — 删除旧组件 / 旧样式 / 旧测试

**Files:**
- Delete: `frontend/src/components/DiffTree.tsx`
- Delete: `frontend/src/components/MetadataTree.tsx`
- Delete: `frontend/src/components/PairComparison.tsx`
- Delete: `frontend/src/components/DirectoryOverview.tsx`
- Delete: `frontend/src/components/EmptyState.tsx`
- Delete: `frontend/src/components/StatusBanner.tsx`
- Delete: `frontend/src/components/DiffStrip.tsx`
- Delete: `frontend/src/components/FileCard.tsx`
- Delete: `frontend/src/components/ViewModeStrip.tsx`
- Delete: `frontend/src/components/workbench-sync.test.tsx`
- Delete: `frontend/src/lib/diffUtils.ts`

- [ ] **Step 11.1 — 验证这些文件没被新代码引用**

Run: `npm --prefix frontend run build`
Expected: 若任何旧文件被新文件引用（错引），构建会报错；目前应该全部通过。如果报错——修引用，再继续。

- [ ] **Step 11.2 — 删除旧文件**

```bash
rm frontend/src/components/DiffTree.tsx
rm frontend/src/components/MetadataTree.tsx
rm frontend/src/components/PairComparison.tsx
rm frontend/src/components/DirectoryOverview.tsx
rm frontend/src/components/EmptyState.tsx
rm frontend/src/components/StatusBanner.tsx
rm frontend/src/components/DiffStrip.tsx
rm frontend/src/components/FileCard.tsx
rm frontend/src/components/ViewModeStrip.tsx
rm frontend/src/components/workbench-sync.test.tsx
rm frontend/src/lib/diffUtils.ts
```

- [ ] **Step 11.3 — 跑全部测试，确认没再有引用残骸**

Run: `npm --prefix frontend run test -- --run`
Expected: 全部 PASS（约 50+ assertion）

- [ ] **Step 11.4 — 跑构建**

Run: `npm --prefix frontend run build`
Expected: 类型检查 + Vite 构建均成功

- [ ] **Step 11.5 — 提交**

```bash
git add -A frontend/src
git commit -m "chore: remove obsolete components, styles, and tests after redesign"
```

---

## Task 12 — 端到端 smoke + 类型/构建验证

- [ ] **Step 12.1 — 启动 Tauri 开发态，手测以下场景**

Run（从仓库根）: `cargo tauri dev`
（如果没有 Rust dev 环境，跳过此 step；CI/构建步骤覆盖即可）

手测 checklist：
- [ ] 启动后是欢迎页（黑底，提示拖入）
- [ ] 拖一张 PNG 到左槽 → 看到中文标签的元数据树（SoloTree），顶栏显示文件名
- [ ] 再拖一张到右槽 → 切到 MirrorTree，差异行黄底，左右值都清晰可见
- [ ] 切到 `JSON` 视图、`图片` 视图，都能渲染
- [ ] 点 `仅看不同`：unchanged 行消失；嵌套 `途经站点` 中如有差异，自动展开
- [ ] 切到目录模式（点顶部"目录"），拖入两个目录 → DirectoryList，统计条 + 芯片正确，默认筛选 `不一致`
- [ ] 点击 `不一致` 行 → MirrorTree；点击 `仅左/仅右` 行 → SoloTree；`← 返回目录`、`[`、`]` 都工作
- [ ] 拖 PNG 时身处目录模式 → toast `已切换到单文件模式`
- [ ] `Esc` 在 mirror 子页（来自目录）→ 返回目录
- [ ] `Ctrl+Enter` 重跑分析

- [ ] **Step 12.2 — 跑全套自动化校验**

Run: `npm --prefix frontend run test -- --run && npm --prefix frontend run build && cargo check --manifest-path Cargo.toml`
Expected: 全部成功

- [ ] **Step 12.3 — 最后总提交（如果之前每步都 commit 过则跳过）**

```bash
git status
# 若有任何尾随小修，把它们整理为一个 commit。
```

---

## Self-Review

**Spec coverage check：**
- §0 设计目标 — 文件结构 + Task 1-11 全覆盖
- §1 字段映射 — Task 1
- §2 信息架构与流程 — Task 7（槽位/SlotBar）+ Task 8（视图分支/快捷键/拖放模式切换）+ Task 10（顶栏/控件条/欢迎页）
- §3 树形渲染 — Task 5（SoloTree）；分组小标题、缩进、折叠、占位行、嵌套数组默认折叠、数组元素摘要标签全部体现
- §4 1v1 镜像 — Task 6；行级状态视觉、占位行、`仅看不同`、强制展开嵌套差异 全部覆盖；`复制差异为 Markdown` 的具体实现暂未做（spec 第 §4 控件条提到，本计划不实现）→ 已在下方"Out of scope of v1 plan"中记录
- §5 目录概览 — Task 9；统计条 + 筛选芯片 + 列表 + 默认 `不一致`。`自动分组（>50）` 和 `排序` 控件、`扫描进度条` 同样未做 → 记入 Out of scope。`错误项详情面板` 也未做 → 暂用 banner 提示。
- §6 视觉系统 — Task 3；token、字体栈、字号、间距、圆角、徽章、分组小标题、键值行、焦点环、滚动条、动效、显式不做事项 全部落地
- §7 实施轮廓 — Task 1-11 严格按这个顺序

**Out of scope of v1 plan（明确未做、留 v2）：**
- 复制差异为 Markdown
- 目录列表自动分组（>50）
- 目录排序下拉
- 扫描进度条
- 错误详情面板
- `右键菜单 / 复制路径 / 在资源管理器中显示`
- 分组标题悬停的"复制 JSON 子树"和"只看本组差异"

**Placeholder scan：**
- 没有 TBD/TODO/"implement later"
- 没有"add appropriate error handling"等空话
- 每步都有具体代码或具体命令

**Type consistency：**
- `MirrorRow` / `TreeNode` 在 Task 2 定义，Task 5/6 引用一致
- `useWorkbench` 返回值在 Task 8 完整列出，Task 10 引用的字段（`view`、`soloSide`、`soloResult`、`onlyDiff`、`toggleOnlyDiff`、`toggleSlotBarCollapsed`、`tryDropPath`、`runAuto`、`toast`）全部存在
- `StatusBadge` 接受的 `kind: DiffStatus` 与 lib/types 中已有的类型一致
- `arrayItemLabel` / `fieldLabel` / `formatValue` / `isKnownField` 接口在 Task 1 定义、Task 2 使用
