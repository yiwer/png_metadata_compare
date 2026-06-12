# UI 舒适化重设计 + 性能专项 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按 [2026-06-12 设计稿](../specs/2026-06-12-ui-comfort-redesign-design.md) 把应用改造为「主从布局 + 单树双值列 + 右侧差异栏 + 石墨皮肤」，并完成性能专项（选目录卡顿、渲染减负、扫描取消）。

**Architecture:** 渐进式改造（甲路线）：每个任务在现有组件上做定向手术，任务结束时应用可编译、测试全绿、可运行。先做底层（色板、行模型、纯逻辑），再做独立新组件（UnifiedTree / DiffRail / Sidebar / WelcomePane，先建好并单测，不挂载），最后一次性把 `App.tsx` 切到新骨架，再做后端取消与对话框修复，最后退役旧组件。

**Tech Stack:** React 18 + TypeScript + Vite + Vitest（frontend/）；Rust + Tauri 2（src/）。无新增第三方前端库；Rust 侧视诊断结果新增 `rfd` 直接依赖。

**约定：**
- 前端测试：`cd frontend; npx vitest run <file>`（一次性运行）；全量 `cd frontend; npx vitest run`
- 前端构建（含 tsc 类型检查）：`cd frontend; npm run build`
- Rust 测试：`cargo test`（仓库根）
- 手动跑应用：`cargo tauri dev`
- 提交信息后缀统一带 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`

---

## Task 1: 石墨色板（tokens.css 重写，旧变量名保留为别名）

**Files:**
- Modify: `frontend/src/styles/tokens.css`（全文件替换）

旧组件（Slot/MirrorTree/DirectoryList…）在 Task 7 前仍在线，所以旧变量名一律保留、只换值，新名字与旧名字共存；Task 10 清理别名。

- [ ] **Step 1: 全文替换 tokens.css**

```css
/* frontend/src/styles/tokens.css — graphite palette (2026-06-12 redesign §3) */
:root {
  /* Surface */
  --bg-page:     #1b1e23;   /* 中央底 */
  --bg-panel:    #1e2126;   /* 侧栏 / 差异栏底 */
  --bg-elevated: #22262d;   /* 顶栏 / 卡片 / 输入框 */
  --bg-hover:    #262a31;

  /* Legacy aliases (Task 10 移除) */
  --bg-topbar:      #22262d;
  --bg-slotbar:     #1e2126;
  --bg-controlbar:  #1b1e23;
  --bg-overlay:     #262a31;
  --bg-tint-warm:   rgba(214, 185, 123, 0.04);
  --bg-tint-cool:   rgba(130, 170, 255, 0.04);
  --bar-edge-top:    none;
  --bar-edge-bottom: none;

  /* Text */
  --text-primary:   #e8eaed;
  --text-secondary: #9aa0a6;
  --text-tertiary:  #6b7178;
  --text-disabled:  #4a4f57;

  /* Borders */
  --border-subtle:  #2a2e35;
  --border-default: #33373e;
  --border-emph:    #3a3f47;
  --border-focus:   #82aaff;

  /* Diff status */
  --mod-bg:    rgba(214, 185, 123, 0.13);
  --mod-old:   #c8935e;
  --mod-new:   #e8c97a;
  --mod-text:  #e8c97a;   /* legacy alias */
  --mod-arrow: #e8c97a;   /* legacy alias */

  --add-bg:    rgba(152, 195, 121, 0.10);
  --add-text:  #98c379;
  --add-emph:  #98c379;   /* legacy alias */

  --rem-bg:    rgba(224, 108, 117, 0.10);
  --rem-text:  #e06c75;
  --rem-emph:  #e06c75;   /* legacy alias */

  --reord-bord: #82aaff;

  --err-bg:    rgba(214, 120, 181, 0.10);
  --err-text:  #d678b5;

  /* Group title */
  --group-head:        #d6b97b;
  --group-rule:        rgba(214, 185, 123, 0.15);
  --group-bg:          transparent;
  --group-head-nested: rgba(214, 185, 123, 0.75);
  --group-rule-nested: rgba(214, 185, 123, 0.40);

  /* Accent */
  --accent:    #82aaff;
  --accent-bg: #2d3f5e;

  /* Fonts */
  --font-ui:   "Inter", "PingFang SC", "Microsoft YaHei", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", "SF Mono", "Cascadia Code", ui-monospace, Consolas, monospace;

  /* Sizes — 树行用 --fs-md (13px)，§3 舒适度调整 */
  --fs-xs:  11px;  --lh-xs: 1.4;
  --fs-sm:  12px;  --lh-sm: 1.5;
  --fs-md:  13px;  --lh-md: 1.6;
  --fs-lg:  14px;  --lh-lg: 1.4;
  --fs-xl:  18px;  --lh-xl: 1.3;
  --fs-2xl: 24px;  --lh-2xl: 1.2;

  /* Spacing */
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

- [ ] **Step 2: 构建 + 全量测试确认无破坏**

Run: `cd frontend; npm run build; npx vitest run`
Expected: build 通过；测试全绿（颜色变化不影响断言）

- [ ] **Step 3: 手动目检（可选但推荐）**

Run: `cargo tauri dev` → 拖入两个 PNG，确认整体已变为石墨基调（旧布局，新颜色），无纯黑底、无霓虹色。

- [ ] **Step 4: Commit**

```powershell
git add frontend/src/styles/tokens.css
git commit -m "feat(theme): graphite palette, legacy token names aliased"
```

---

## Task 2: 行模型携带原始 JSON 子树（leftRaw / rightRaw）

**Files:**
- Modify: `frontend/src/lib/treeModel.ts`
- Test: `frontend/src/lib/treeModel.test.ts`（已存在，追加用例）

「复制 JSON 子树 / 复制值」需要行上有原始数据；merge 时 li/ri 就在手边，顺手带上。

- [ ] **Step 1: 写失败测试（追加到 treeModel.test.ts 末尾的 describe 外层或新 describe）**

```ts
describe('buildMirrorRows raw payloads', () => {
  it('carries leftRaw/rightRaw on leaves and groups', () => {
    const left = { StopName: '翻身地铁站', Lines: [{ LineName: 'B932', Direction: '东' }] };
    const right = { StopName: '翻身地铁站', Lines: [{ LineName: 'B932', Direction: '西' }] };
    const [root] = buildMirrorRows(left, right, null);
    const stopName = root.children!.find((r) => r.path === 'StopName')!;
    expect(stopName.leftRaw).toBe('翻身地铁站');
    expect(stopName.rightRaw).toBe('翻身地铁站');
    const lines = root.children!.find((r) => r.path === 'Lines')!;
    expect(lines.leftRaw).toEqual(left.Lines);
    expect(lines.rightRaw).toEqual(right.Lines);
    const item = lines.children![0];
    expect(item.leftRaw).toEqual(left.Lines[0]);
    expect(item.rightRaw).toEqual(right.Lines[0]);
  });

  it('leaves missing side raw as undefined', () => {
    const [root] = buildMirrorRows({ A: 1 }, {}, null);
    const a = root.children!.find((r) => r.path === 'A')!;
    expect(a.leftRaw).toBe(1);
    expect(a.rightRaw).toBeUndefined();
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend; npx vitest run src/lib/treeModel.test.ts`
Expected: FAIL（`leftRaw` 不存在 / undefined 不匹配）

- [ ] **Step 3: 实现**

`MirrorRow` 接口追加两个字段：

```ts
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
```

四处 return 补字段（值直接用入参，undefined 自然缺省）：

- `mergeObject` 的 return：`leftValue: null,` 后追加 `leftRaw: left ?? undefined, rightRaw: right ?? undefined,`
- `mergeArray` 的 return：同样追加 `leftRaw: left, rightRaw: right,`
- `mergeLeaf` 的 return：追加 `leftRaw: left, rightRaw: right,`

注意 `mergeObject` 的 `left`/`right` 形参类型是 `Record<string, JsonValue> | null`，`?? undefined` 把 null（表示该侧无对象）归一为 undefined，与 leaf 缺失语义一致。

- [ ] **Step 4: 运行确认通过 + 回归**

Run: `cd frontend; npx vitest run src/lib/treeModel.test.ts; npx vitest run`
Expected: 全部 PASS

- [ ] **Step 5: Commit**

```powershell
git add frontend/src/lib/treeModel.ts frontend/src/lib/treeModel.test.ts
git commit -m "feat(treeModel): carry leftRaw/rightRaw on mirror rows for copy actions"
```

---

## Task 3: UnifiedTree 组件（单树双值列，含 solo 模式）

**Files:**
- Create: `frontend/src/components/UnifiedTree.tsx`
- Test: `frontend/src/components/UnifiedTree.test.tsx`
- Modify: `frontend/src/styles/app.css`（追加 `.utree` 样式块）

行模型由父组件算好传入（§5 单处 useMemo）。本任务只建组件 + 单测，不挂载到 App。

- [ ] **Step 1: 写失败测试**

```tsx
// frontend/src/components/UnifiedTree.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { UnifiedTree } from './UnifiedTree';
import { buildMirrorRows } from '../lib/treeModel';
import type { DiffNode } from '../lib/types';

const left = { StopName: '翻身地铁站', RoadName: '创业一路' };
const right = { StopName: '翻身地铁站', RoadName: '创业二路' };
const diff: DiffNode = {
  path: '', status: 'modified', left_value: null, right_value: null, summary: '', children: [
    { path: 'RoadName', status: 'modified', left_value: '创业一路', right_value: '创业二路', summary: '', children: [] },
  ],
};

function rowsFor(l: unknown, r: unknown, d: DiffNode | null) {
  return buildMirrorRows(l as never, r as never, d);
}

describe('UnifiedTree', () => {
  it('renders one label cell and two value cells per leaf', () => {
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
      leftLabel="a.png" rightLabel="b.png" focusRequest={null} />);
    expect(screen.getAllByText('创业一路')).toHaveLength(1);
    expect(screen.getAllByText('创业二路')).toHaveLength(1);
    expect(screen.getByText('a.png')).toBeInTheDocument();
    expect(screen.getByText('b.png')).toBeInTheDocument();
  });

  it('marks modified rows with status class only when highlight on', () => {
    const { container, rerender } = render(
      <UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
        leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(container.querySelector('.utree__row--modified')).not.toBeNull();
    rerender(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight={false} onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(container.querySelector('.utree__row--modified')).toBeNull();
  });

  it('onlyDiff hides unchanged leaves', () => {
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(screen.queryByText('翻身地铁站')).toBeNull();
    expect(screen.getByText('创业二路')).toBeInTheDocument();
  });

  it('solo mode renders a single value column without diff classes', () => {
    const { container } = render(
      <UnifiedTree rows={rowsFor(left, null, null)} solo="left" highlight onlyDiff={false}
        leftLabel="a.png" rightLabel="" focusRequest={null} />);
    expect(screen.getByText('翻身地铁站')).toBeInTheDocument();
    expect(container.querySelectorAll('.utree__val')).toHaveLength(2); // 两个叶子各一个值列
    expect(container.querySelector('.utree__row--modified')).toBeNull();
  });

  it('group toggle collapses children', () => {
    const nested = { Lines: [{ LineName: 'B932', Direction: '东' }] };
    render(<UnifiedTree rows={rowsFor(nested, nested, null)} solo={null} highlight onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    // Lines 数组默认折叠（defaultOpen=false）→ 内容不可见
    expect(screen.queryByText('B932')).toBeNull();
    fireEvent.click(screen.getAllByRole('button', { name: '展开' })[0]);
    expect(screen.queryAllByText('B932').length).toBeGreaterThan(0);
  });

  it('copy button writes value to clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    const btns = screen.getAllByRole('button', { name: '复制左值' });
    fireEvent.click(btns[0]);
    expect(writeText).toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend; npx vitest run src/components/UnifiedTree.test.tsx`
Expected: FAIL（模块不存在）

- [ ] **Step 3: 实现 UnifiedTree.tsx**

```tsx
// frontend/src/components/UnifiedTree.tsx
import { useEffect, useRef, useState } from 'react';
import { GroupHead } from './GroupHead';
import { hasDiffDeep } from '../lib/treeModel';
import type { MirrorRow } from '../lib/treeModel';
import type { DiffStatus } from '../lib/types';
import type { Side } from '../lib/types';

export interface FocusRequest {
  path: string;
  seq: number;   // 单调递增，保证同一路径可重复触发
}

const ROW_STATUS: Partial<Record<DiffStatus, string>> = {
  modified: 'utree__row--modified',
  added: 'utree__row--added',
  removed: 'utree__row--removed',
  reordered: 'utree__row--reordered',
  error: 'utree__row--error',
};

export function UnifiedTree({
  rows, solo, highlight, onlyDiff, leftLabel, rightLabel, focusRequest,
}: {
  rows: MirrorRow[];
  solo: Side | null;
  highlight: boolean;
  onlyDiff: boolean;
  leftLabel: string;
  rightLabel: string;
  focusRequest: FocusRequest | null;
}) {
  const [closed, setClosed] = useState<Set<string>>(() => collectDefaultClosed(rows));
  const bodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => { setClosed(collectDefaultClosed(rows)); }, [rows]);

  // 跳转：展开祖先 → 滚动 → 闪烁
  useEffect(() => {
    if (!focusRequest) return;
    setClosed((cur) => openAncestors(rows, focusRequest.path, cur));
    // 等一帧让展开后的行进入 DOM
    const t = requestAnimationFrame(() => {
      const el = bodyRef.current?.querySelector<HTMLElement>(
        `[data-path="${CSS.escape(focusRequest.path)}"]`,
      );
      if (!el) return;
      el.scrollIntoView({ block: 'center' });
      el.classList.remove('utree__row--flash');
      void el.offsetWidth; // 重启 CSS 动画
      el.classList.add('utree__row--flash');
    });
    return () => cancelAnimationFrame(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focusRequest]);

  const toggle = (path: string) =>
    setClosed((cur) => {
      const next = new Set(cur);
      if (next.has(path)) next.delete(path); else next.add(path);
      return next;
    });

  const effectiveClosed = onlyDiff ? withDiffGroupsOpen(rows, closed) : closed;

  return (
    <div className="utree">
      <div className="utree__cols utree__head" data-solo={solo ?? undefined}>
        <span className="utree__col-label">字段</span>
        {(solo === null || solo === 'left') && <span className="utree__col-side" title={leftLabel}>左 · {leftLabel}</span>}
        {(solo === null || solo === 'right') && <span className="utree__col-side" title={rightLabel}>右 · {rightLabel}</span>}
      </div>
      <div className="utree__body" ref={bodyRef}>
        {rows.map((row) => (
          <RowView key={row.path || 'root'} row={row} level={0} closed={effectiveClosed}
            toggle={toggle} highlight={highlight} onlyDiff={onlyDiff} solo={solo} />
        ))}
      </div>
    </div>
  );
}

function RowView({
  row, level, closed, toggle, highlight, onlyDiff, solo,
}: {
  row: MirrorRow; level: number; closed: Set<string>;
  toggle: (p: string) => void; highlight: boolean; onlyDiff: boolean; solo: Side | null;
}) {
  if (onlyDiff && !hasDiffDeep(row)) return null;

  if (row.kind === 'leaf') {
    if (onlyDiff && row.status === 'unchanged') return null;
    return <Leaf row={row} level={level} highlight={highlight} solo={solo} />;
  }

  if (row.variant === 'object-root') {
    return (
      <>
        {row.children?.map((c) => (
          <RowView key={c.path} row={c} level={level} closed={closed} toggle={toggle}
            highlight={highlight} onlyDiff={onlyDiff} solo={solo} />
        ))}
      </>
    );
  }

  // 仅一侧存在的数组项：solo 不染色；非 solo 高亮时整组染色
  const headStatus = solo ? 'unchanged' : row.status;
  const isOpen = !closed.has(row.path);
  const raw = row.leftRaw ?? row.rightRaw;
  return (
    <>
      <div data-path={row.path}>
        <GroupHead
          label={row.label}
          count={row.variant === 'array' ? row.count : undefined}
          level={level}
          open={isOpen}
          onToggle={() => toggle(row.path)}
          status={headStatus}
          highlight={highlight}
          trailing={
            raw !== undefined ? (
              <button type="button" className="utree__copy" aria-label="复制 JSON 子树"
                onClick={(e) => { e.stopPropagation(); void navigator.clipboard?.writeText(JSON.stringify(raw, null, 2)); }}
              >⧉</button>
            ) : undefined
          }
        />
      </div>
      {isOpen && (
        <div className="utree__nested">
          {row.children?.map((c) => (
            <RowView key={c.path} row={c} level={level + 1} closed={closed} toggle={toggle}
              highlight={highlight} onlyDiff={onlyDiff} solo={solo} />
          ))}
        </div>
      )}
    </>
  );
}

function Leaf({ row, level, highlight, solo }: { row: MirrorRow; level: number; highlight: boolean; solo: Side | null }) {
  const statusCls = !solo && highlight ? ROW_STATUS[row.status] ?? '' : '';
  const showLeft = solo === null || solo === 'left';
  const showRight = solo === null || solo === 'right';
  return (
    <div className={`utree__cols utree__row ${statusCls}`.trim()} data-path={row.path} data-solo={solo ?? undefined}
      style={{ paddingLeft: 10 + level * 16 }}>
      <span className="utree__key">
        {row.label}
        {row.isUnknown && <span className="utree__unknown">未识别</span>}
      </span>
      {showLeft && (
        <span className={`utree__val${statusCls && row.status === 'modified' ? ' utree__val--old' : ''}`}>
          {row.leftValue}
          <button type="button" className="utree__copy" aria-label="复制左值"
            onClick={() => void navigator.clipboard?.writeText(row.leftValue ?? '')}>⧉</button>
        </span>
      )}
      {showRight && (
        <span className={`utree__val${statusCls && row.status === 'modified' ? ' utree__val--new' : ''}`}>
          {row.rightValue}
          <button type="button" className="utree__copy" aria-label="复制右值"
            onClick={() => void navigator.clipboard?.writeText(row.rightValue ?? '')}>⧉</button>
        </span>
      )}
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

function withDiffGroupsOpen(rows: MirrorRow[], closed: Set<string>): Set<string> {
  const next = new Set(closed);
  const walk = (rs: MirrorRow[]) => {
    for (const r of rs) {
      if (r.kind === 'group' && hasDiffDeep(r)) next.delete(r.path);
      if (r.children) walk(r.children);
    }
  };
  walk(rows);
  return next;
}

/** 打开 target 的所有祖先分组（段前缀判定：后随 '.' 或 '['）。 */
function openAncestors(rows: MirrorRow[], target: string, closed: Set<string>): Set<string> {
  const next = new Set(closed);
  const walk = (rs: MirrorRow[]) => {
    for (const r of rs) {
      if (r.kind === 'group' && r.path && isAncestorPath(r.path, target)) next.delete(r.path);
      if (r.children) walk(r.children);
    }
  };
  walk(rows);
  return next;
}

function isAncestorPath(parent: string, child: string): boolean {
  if (!child.startsWith(parent)) return false;
  const rest = child.slice(parent.length);
  return rest === '' || rest.startsWith('.') || rest.startsWith('[');
}
```

- [ ] **Step 4: 追加样式到 app.css 末尾**

```css
/* ============ UnifiedTree（单树双值列） ============ */
.utree { display: flex; flex-direction: column; height: 100%; min-width: 0; }
.utree__cols { display: grid; grid-template-columns: minmax(140px, 200px) minmax(0,1fr) minmax(0,1fr); column-gap: var(--sp-4); align-items: baseline; }
.utree__cols[data-solo] { grid-template-columns: minmax(140px, 200px) minmax(0,1fr); }
.utree__head { padding: 6px 10px; border-bottom: 1px solid var(--border-default); color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-mono); position: sticky; top: 0; background: var(--bg-page); z-index: 1; }
.utree__col-side { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.utree__body { overflow-y: auto; flex: 1; padding-bottom: var(--sp-5); }
.utree__nested { border-left: 1px solid var(--border-subtle); margin-left: 10px; content-visibility: auto; contain-intrinsic-size: auto 600px; }
.utree__row { padding-top: 4px; padding-bottom: 4px; padding-right: 10px; border-radius: var(--r-sm); font: var(--fs-md)/var(--lh-md) var(--font-mono); }
.utree__row:hover { background: var(--bg-hover); }
.utree__key { color: var(--text-secondary); }
.utree__val { color: var(--text-primary); word-break: break-all; position: relative; }
.utree__row--modified { background: var(--mod-bg); }
.utree__row--modified .utree__val--old { color: var(--mod-old); }
.utree__row--modified .utree__val--new { color: var(--mod-new); }
.utree__row--added { background: var(--add-bg); }
.utree__row--added .utree__val { color: var(--add-text); }
.utree__row--removed { background: var(--rem-bg); }
.utree__row--removed .utree__val { color: var(--rem-text); }
.utree__row--reordered { border-left: 3px solid var(--reord-bord); }
.utree__row--error { background: var(--err-bg); }
.utree__row--error .utree__val { color: var(--err-text); }
.utree__unknown { margin-left: 6px; font-size: 10px; color: var(--text-tertiary); border: 1px solid var(--border-default); border-radius: var(--r-sm); padding: 0 4px; }
.utree__copy { visibility: hidden; margin-left: 6px; border: none; background: none; color: var(--text-tertiary); cursor: pointer; font-size: var(--fs-xs); }
.utree__row:hover .utree__copy, .group-head:hover .utree__copy { visibility: visible; }
.utree__copy:hover { color: var(--accent); }
@keyframes utree-flash { 0% { outline: 2px solid var(--accent); } 100% { outline: 2px solid transparent; } }
.utree__row--flash { animation: utree-flash 0.9s ease-out 1; }
```

- [ ] **Step 5: 运行确认通过**

Run: `cd frontend; npx vitest run src/components/UnifiedTree.test.tsx`
Expected: 全部 PASS

- [ ] **Step 6: Commit**

```powershell
git add frontend/src/components/UnifiedTree.tsx frontend/src/components/UnifiedTree.test.tsx frontend/src/styles/app.css
git commit -m "feat(UnifiedTree): single tree with left/right value columns, solo mode, jump-and-flash"
```

---

## Task 4: 差异清单纯逻辑（diffList.ts）+ DiffRail 组件

**Files:**
- Create: `frontend/src/lib/diffList.ts`
- Create: `frontend/src/components/DiffRail.tsx`
- Test: `frontend/src/lib/diffList.test.ts`、`frontend/src/components/DiffRail.test.tsx`
- Modify: `frontend/src/styles/app.css`（追加 `.rail` 样式块）

- [ ] **Step 1: 写 diffList 失败测试**

```ts
// frontend/src/lib/diffList.test.ts
import { describe, it, expect } from 'vitest';
import { buildDiffEntries, buildDiffText } from './diffList';
import { buildMirrorRows } from './treeModel';
import type { DiffNode } from './types';

const diff = (children: DiffNode[]): DiffNode =>
  ({ path: '', status: 'modified', left_value: null, right_value: null, summary: '', children });
const node = (path: string, status: DiffNode['status']): DiffNode =>
  ({ path, status, left_value: null, right_value: null, summary: '', children: [] });

describe('buildDiffEntries', () => {
  it('collects changed leaves with top group label', () => {
    const left = { StopName: 'A', Lines: [{ LineName: 'B932', Direction: '东', NextStop: '尚都花园' }] };
    const right = { StopName: 'A', Lines: [{ LineName: 'B932', Direction: '西' }] };
    const rows = buildMirrorRows(left, right, diff([
      node('Lines[B932|东]', 'modified'),    // 容器变化不计入条目
      node('Lines[B932|东].Direction', 'modified'),
      node('Lines[B932|东].NextStop', 'removed'),
    ]));
    const entries = buildDiffEntries(rows);
    expect(entries).toHaveLength(2);
    expect(entries[0].topGroup).toContain('线路');
    expect(entries.map((e) => e.status)).toEqual(['modified', 'removed']);
  });

  it('counts an array item that exists on one side as a single unit', () => {
    const left = { Lines: [] as unknown[] };
    const right = { Lines: [{ LineName: 'M197', Direction: '北' }] };
    const rows = buildMirrorRows(left, right, diff([node('Lines[M197|北]', 'added')]));
    const entries = buildDiffEntries(rows);
    expect(entries).toHaveLength(1);
    expect(entries[0].status).toBe('added');
  });
});

describe('buildDiffText', () => {
  it('formats modified / one-side lines', () => {
    const text = buildDiffText([
      { path: 'a', topGroup: '线路 1 · B932', label: '开往方向', status: 'modified', leftValue: '福城万达广场', rightValue: '福城天虹' },
      { path: 'b', topGroup: '线路 1 · B932', label: '下一站', status: 'removed', leftValue: '尚都花园', rightValue: '—' },
      { path: 'c', topGroup: '', label: '中文站名', status: 'added', leftValue: '—', rightValue: '新站' },
    ]);
    expect(text.split('\n')).toEqual([
      '线路 1 · B932 › 开往方向: 福城万达广场 → 福城天虹',
      '线路 1 · B932 › 下一站: 尚都花园（仅左）',
      '中文站名: 新站（仅右）',
    ]);
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend; npx vitest run src/lib/diffList.test.ts`
Expected: FAIL（模块不存在）

- [ ] **Step 3: 实现 diffList.ts**

收集规则与现 `App.tsx` 的 `tallyDiff` 一致：变化叶子逐条计；`array-item` 整项 added/removed 记为单条不下钻。`topGroup` 取根的直接子分组标签；更深的祖先标签并入 `label`（` › ` 连接）。

```ts
// frontend/src/lib/diffList.ts
import type { MirrorRow } from './treeModel';
import type { DiffStatus } from './types';

export interface DiffEntry {
  path: string;
  topGroup: string;          // 顶层分组标签（聚类用），顶层字段为 ''
  label: string;             // 展示名（含中间层级，' › ' 连接）
  status: DiffStatus;
  leftValue: string | null;
  rightValue: string | null;
}

export function buildDiffEntries(rows: MirrorRow[]): DiffEntry[] {
  const out: DiffEntry[] = [];
  const walk = (rs: MirrorRow[], ancestors: string[]) => {
    for (const r of rs) {
      if (r.kind === 'leaf') {
        if (r.status === 'unchanged' || r.status === 'reordered') continue;
        out.push(entryFor(r, ancestors, r.label));
        continue;
      }
      const isItemAddRem = r.variant === 'array-item' && (r.status === 'added' || r.status === 'removed');
      if (isItemAddRem) {
        out.push(entryFor(r, ancestors, r.label));
        continue;
      }
      if (r.children) {
        const nextAncestors = r.variant === 'object-root' || !r.label ? ancestors : [...ancestors, r.label];
        walk(r.children, nextAncestors);
      }
    }
  };
  walk(rows, []);
  return out;
}

function entryFor(r: MirrorRow, ancestors: string[], leafLabel: string): DiffEntry {
  const [top, ...rest] = ancestors;
  return {
    path: r.path,
    topGroup: top ?? '',
    label: [...rest, leafLabel].join(' › '),
    status: r.status,
    leftValue: r.leftValue,
    rightValue: r.rightValue,
  };
}

export function buildDiffText(entries: DiffEntry[]): string {
  return entries
    .map((e) => {
      const name = e.topGroup ? `${e.topGroup} › ${e.label}` : e.label;
      if (e.status === 'modified') return `${name}: ${e.leftValue} → ${e.rightValue}`;
      if (e.status === 'removed') return `${name}: ${e.leftValue}（仅左）`;
      if (e.status === 'added') return `${name}: ${e.rightValue}（仅右）`;
      return `${name}: ${e.leftValue ?? e.rightValue ?? ''}（${e.status}）`;
    })
    .join('\n');
}
```

- [ ] **Step 4: 运行 diffList 测试确认通过**

Run: `cd frontend; npx vitest run src/lib/diffList.test.ts`
Expected: PASS

- [ ] **Step 5: 写 DiffRail 失败测试**

```tsx
// frontend/src/components/DiffRail.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { DiffRail } from './DiffRail';
import type { DiffEntry } from '../lib/diffList';

const entries: DiffEntry[] = [
  { path: 'Lines[B932|东].Direction', topGroup: '线路 1 · B932', label: '开往方向', status: 'modified', leftValue: 'A', rightValue: 'B' },
  { path: 'Lines[B932|东].NextStop', topGroup: '线路 1 · B932', label: '下一站', status: 'removed', leftValue: 'C', rightValue: '—' },
  { path: 'StopName', topGroup: '', label: '中文站名', status: 'added', leftValue: '—', rightValue: 'D' },
];

describe('DiffRail', () => {
  it('clusters entries by top group and shows count', () => {
    render(<DiffRail entries={entries} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    expect(screen.getByText('差异 3')).toBeInTheDocument();
    expect(screen.getByText('线路 1 · B932')).toBeInTheDocument();
    expect(screen.getByText(/开往方向/)).toBeInTheDocument();
  });

  it('clicking an entry calls onJump with its path', () => {
    const onJump = vi.fn();
    render(<DiffRail entries={entries} onJump={onJump} collapsed={false} onToggle={() => {}} />);
    fireEvent.click(screen.getByText(/开往方向/));
    expect(onJump).toHaveBeenCalledWith('Lines[B932|东].Direction');
  });

  it('copy button puts the diff text on the clipboard', () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<DiffRail entries={entries} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: '复制差异清单' }));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('开往方向: A → B'));
  });

  it('renders nothing but reopen handle when collapsed', () => {
    render(<DiffRail entries={entries} onJump={() => {}} collapsed onToggle={() => {}} />);
    expect(screen.queryByText('线路 1 · B932')).toBeNull();
  });

  it('shows 无差异 when empty', () => {
    render(<DiffRail entries={[]} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    expect(screen.getByText('无差异')).toBeInTheDocument();
  });
});
```

- [ ] **Step 6: 运行确认失败，然后实现 DiffRail.tsx**

Run: `cd frontend; npx vitest run src/components/DiffRail.test.tsx` → FAIL

```tsx
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
          <button type="button" className="rail__copy"
            onClick={() => void navigator.clipboard?.writeText(buildDiffText(entries))}>
            复制差异清单
          </button>
        </div>
      )}
    </aside>
  );
}
```

- [ ] **Step 7: 追加样式到 app.css 末尾**

```css
/* ============ DiffRail（右侧差异栏） ============ */
.rail { width: 220px; flex: none; display: flex; flex-direction: column; background: var(--bg-panel); border-left: 1px solid var(--border-default); min-height: 0; }
.rail--collapsed { width: 28px; align-items: center; padding-top: var(--sp-3); cursor: pointer; border: none; border-left: 1px solid var(--border-default); color: var(--text-secondary); font: var(--fs-xs)/1 var(--font-mono); }
.rail__collapsed-count { background: var(--mod-bg); color: var(--mod-new); border-radius: var(--r-sm); padding: 2px 5px; }
.rail__head { display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; border-bottom: 1px solid var(--border-subtle); color: var(--text-primary); font: 500 var(--fs-sm)/1 var(--font-ui); }
.rail__toggle { border: none; background: none; color: var(--text-tertiary); cursor: pointer; }
.rail__toggle:hover { color: var(--accent); }
.rail__body { flex: 1; overflow-y: auto; padding: var(--sp-2); }
.rail__empty { color: var(--text-tertiary); font: var(--fs-sm)/1 var(--font-ui); padding: var(--sp-3); }
.rail__group { margin-bottom: var(--sp-3); }
.rail__group-name { color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-ui); padding: 2px 6px; }
.rail__entry { display: block; width: 100%; text-align: left; border: none; background: none; cursor: pointer; border-radius: var(--r-sm); padding: 3px 8px; font: var(--fs-sm)/var(--lh-sm) var(--font-mono); color: var(--text-secondary); overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.rail__entry:hover { background: var(--bg-hover); }
.rail__entry--mod { background: var(--mod-bg); color: var(--mod-new); }
.rail__entry--add { color: var(--add-text); }
.rail__entry--rem { color: var(--rem-text); }
.rail__entry--err { color: var(--err-text); }
.rail__foot { border-top: 1px solid var(--border-subtle); padding: var(--sp-2); }
.rail__copy { width: 100%; border: 1px solid var(--border-default); background: none; color: var(--text-secondary); border-radius: var(--r-sm); padding: 5px 0; cursor: pointer; font: var(--fs-sm)/1 var(--font-ui); }
.rail__copy:hover { border-color: var(--accent); color: var(--accent); }
```

- [ ] **Step 8: 运行确认通过 + 回归**

Run: `cd frontend; npx vitest run src/components/DiffRail.test.tsx; npx vitest run`
Expected: 全部 PASS

- [ ] **Step 9: Commit**

```powershell
git add frontend/src/lib/diffList.ts frontend/src/lib/diffList.test.ts frontend/src/components/DiffRail.tsx frontend/src/components/DiffRail.test.tsx frontend/src/styles/app.css
git commit -m "feat(DiffRail): grouped diff entries rail with jump and copy-as-text"
```

---

## Task 5: useWorkbench 选中流 + Sidebar 组件

**Files:**
- Modify: `frontend/src/features/workbench/useWorkbench.ts`（增量，不破坏现有 API）
- Create: `frontend/src/components/Sidebar.tsx`
- Test: `frontend/src/features/workbench/useWorkbench.test.tsx`（追加）、`frontend/src/components/Sidebar.test.tsx`
- Modify: `frontend/src/styles/app.css`（追加 `.sidebar` 样式块）

本任务只做**增量状态**与独立组件；App 接线在 Task 7。`view` 枚举与 `inDirectorySubview` 暂不动。

- [ ] **Step 1: 写 useWorkbench 失败测试（追加）**

```tsx
describe('sidebar selection & search (redesign)', () => {
  it('searchQuery narrows filteredItems by label substring, case-insensitive', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    act(() => { result.current.setActiveFilter('all'); result.current.setSearchQuery('A.PNG'); });
    expect(result.current.filteredItems.map((i) => i.label)).toEqual(['a.png']);
  });

  it('sortKey diff-desc orders different items by count desc, then kinds, then name', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    act(() => { result.current.setActiveFilter('all'); });
    expect(result.current.filteredItems.map((i) => i.label)).toEqual(['b.png', 'a.png', 'c.png']);
    act(() => { result.current.setSortKey('name-asc'); });
    expect(result.current.filteredItems.map((i) => i.label)).toEqual(['a.png', 'b.png', 'c.png']);
  });

  it('scan auto-selects the first item of the default filter and loads it', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    // 默认筛选 different，diff-desc 排序下第一项是 b.png（2 处差异）
    expect(result.current.selectedItemId).toBe('2');
    expect(api.compareSingle).toHaveBeenCalledWith('/l/b.png', '/r/b.png');
  });

  it('selectNext / selectPrev walk the filtered list and clamp at ends', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => { await result.current.selectNext(); });
    expect(result.current.selectedItemId).toBe('1');
    await act(async () => { await result.current.selectNext(); });   // 末尾，原地
    expect(result.current.selectedItemId).toBe('1');
    await act(async () => { await result.current.selectPrev(); });
    expect(result.current.selectedItemId).toBe('2');
  });

  it('zero different falls back to all filter after scan', async () => {
    const api = makeApi({
      scanDirectory: vi.fn().mockResolvedValue({
        counts: { identical: 1, different: 0, left_only: 0, right_only: 0, error: 0 },
        items: [{ id: '9', kind: 'identical', label: 'z.png', left_path: '/l/z.png', right_path: '/r/z.png', difference_count: 0, match_strategy: 'file_name', message: null }],
      }),
    });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    expect(result.current.activeFilter).toBe('all');
  });

  it('selecting an error item exposes errorItem without calling compare', async () => {
    const api = makeApi({
      scanDirectory: vi.fn().mockResolvedValue({
        counts: { identical: 0, different: 0, left_only: 0, right_only: 0, error: 1 },
        items: [{ id: 'e1', kind: 'error', label: 'C:/bad.png', left_path: null, right_path: null, difference_count: 0, match_strategy: null, message: 'E_NO_METADATA' }],
      }),
    });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    expect(result.current.errorItem?.id).toBe('e1');
    expect(api.compareSingle).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend; npx vitest run src/features/workbench/useWorkbench.test.tsx`
Expected: FAIL（`setSearchQuery` 等不存在）

- [ ] **Step 3: 实现 useWorkbench 增量**

类型与纯排序函数（放在文件顶部、`useWorkbench` 之外，便于复用与测试）：

```ts
export type SortKey = 'diff-desc' | 'name-asc';

const KIND_RANK: Record<BatchListItemKind, number> = {
  different: 0, left_only: 1, right_only: 2, error: 3, identical: 4,
};

export function sortItems(items: BatchListItem[], key: SortKey): BatchListItem[] {
  const byName = (a: BatchListItem, b: BatchListItem) => a.label.localeCompare(b.label, 'zh');
  const sorted = [...items];
  if (key === 'name-asc') return sorted.sort(byName);
  return sorted.sort((a, b) =>
    KIND_RANK[a.kind] - KIND_RANK[b.kind]
    || b.difference_count - a.difference_count
    || byName(a, b));
}
```

新增状态与动作（加入 `useWorkbench` 函数体）：

```ts
const [searchQuery, setSearchQuery] = useState('');
const [sortKey, setSortKey] = useState<SortKey>('diff-desc');
const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
const [errorItem, setErrorItem] = useState<BatchListItem | null>(null);
const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
const [railCollapsed, setRailCollapsed] = useState(false);
```

`filteredItems` 由现有的单行 filter 替换为：

```ts
const query = searchQuery.trim().toLowerCase();
const filteredItems = sortItems(
  (directorySummary?.items ?? [])
    .filter((i) => activeFilter === 'all' || i.kind === activeFilter)
    .filter((i) => !query || i.label.toLowerCase().includes(query)),
  sortKey,
);
```

`selectItem`（`navigateToPair` 改名并扩展；保留 `navigateToPair` 别名导出以免改动现有测试）：

```ts
async function selectItem(item: BatchListItem) {
  setSelectedItemId(item.id);
  setErrorItem(null);
  setIsLoading(true);
  setError(null);

  const differentItems = (directorySummary?.items ?? []).filter((i) => i.kind === 'different');
  const diffIndex = differentItems.findIndex((i) => i.id === item.id);
  setDirectoryContext(diffIndex >= 0 ? { index: diffIndex + 1, totalDifferent: differentItems.length } : null);

  try {
    setInDirectorySubview(true);
    if (item.kind === 'error') {
      setErrorItem(item);
      setPairResult(null); setSoloResult(null); setSoloSide(null);
      setView('directory-overview'); // Task 7 改为 'error'
    } else if (item.kind === 'left_only' && item.left_path) {
      const result = await api.inspectSingle(item.left_path, 'left');
      setSoloResult(result); setSoloSide('left'); setView('solo'); setViewMode('tree');
    } else if (item.kind === 'right_only' && item.right_path) {
      const result = await api.inspectSingle(item.right_path, 'right');
      setSoloResult(result); setSoloSide('right'); setView('solo'); setViewMode('tree');
    } else if (item.left_path && item.right_path) {
      const result = await api.compareSingle(item.left_path, item.right_path);
      setPairResult(result); setView('mirror'); setViewMode('tree');
    } else {
      setError('无法打开此项目：路径缺失');
    }
  } catch (err) {
    setError(formatError(err));
  } finally {
    setIsLoading(false);
  }
}
const navigateToPair = selectItem;

async function selectNext() { await selectByOffset(1); }
async function selectPrev() { await selectByOffset(-1); }
async function selectByOffset(delta: number) {
  if (filteredItems.length === 0) return;
  const cur = filteredItems.findIndex((i) => i.id === selectedItemId);
  const next = cur < 0 ? 0 : Math.min(filteredItems.length - 1, Math.max(0, cur + delta));
  if (next === cur) return;
  await selectItem(filteredItems[next]);
}

// Task 8 之前后端还没有 cancel_scan 命令，这里先落桩：
// api.cancelScan 是可选成员，缺省时本函数是 no-op。
async function cancelScan() {
  try { await api.cancelScan?.(); } catch { /* 后端未实现/未启动时静默 */ }
}
```

同时给 `frontend/src/lib/api.ts` 的 `WorkbenchApi` 接口追加**可选**成员（本任务不实现）：

```ts
cancelScan?(): Promise<void>;
```

`runAuto` 目录成功分支（`setView('directory-overview'); setSlotBarCollapsed(true);` 之后）追加默认筛选回退 + 自动选中：

```ts
const defaultFilter: ActiveFilter = summary.counts.different > 0 ? 'different' : 'all';
setActiveFilter(defaultFilter);
setSearchQuery('');
setSelectedItemId(null);
const visible = sortItems(
  summary.items.filter((i) => defaultFilter === 'all' || i.kind === defaultFilter),
  sortKey,
);
if (visible[0]) void selectItem(visible[0]);
```

（原 `setActiveFilter('different')` 一行删除，避免重复设置。）`setMode` 重置块追加：`setSearchQuery(''); setSelectedItemId(null); setErrorItem(null);`。

return 对象追加：`searchQuery, setSearchQuery, sortKey, setSortKey, selectedItemId, selectItem, selectNext, selectPrev, errorItem, sidebarCollapsed, railCollapsed, toggleSidebarCollapsed, toggleRailCollapsed, setRailCollapsed, cancelScan`（两个 toggle 函数按 `toggleSlotBarCollapsed` 同款写法）。

- [ ] **Step 4: 运行 hook 测试确认通过**

Run: `cd frontend; npx vitest run src/features/workbench/useWorkbench.test.tsx`
Expected: 新旧用例全部 PASS（自动选中会让既有「navigateToPair」用例多一次 compareSingle 调用——若有用例断言调用次数为 1，将断言改为 `toHaveBeenCalledWith(...)` 形式）

- [ ] **Step 5: 写 Sidebar 失败测试**

```tsx
// frontend/src/components/Sidebar.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Sidebar } from './Sidebar';
import type { BatchListItem, DirectorySummary } from '../lib/types';

const items: BatchListItem[] = [
  { id: '1', kind: 'different', label: 'a.png', left_path: '/l/a.png', right_path: '/r/a.png', difference_count: 3, match_strategy: 'file_name', message: null },
  { id: '2', kind: 'left_only', label: 'b.png', left_path: '/l/b.png', right_path: null, difference_count: 0, match_strategy: null, message: null },
];
const summary: DirectorySummary = {
  counts: { identical: 0, different: 1, left_only: 1, right_only: 0, error: 0 },
  items,
};

function renderBar(over: Partial<Parameters<typeof Sidebar>[0]> = {}) {
  const props = {
    leftDir: 'C:/tmp/bim_v1', rightDir: 'C:/tmp/bim_v2',
    summary, filteredItems: items, activeFilter: 'all' as const,
    searchQuery: '', sortKey: 'diff-desc' as const,
    selectedItemId: '1', isLoading: false, scanProgress: null,
    onFilter: vi.fn(), onSearch: vi.fn(), onSort: vi.fn(), onSelect: vi.fn(),
    onPickLeft: vi.fn(), onPickRight: vi.fn(), onCancelScan: vi.fn(),
    ...over,
  };
  render(<Sidebar {...props} />);
  return props;
}

describe('Sidebar', () => {
  it('renders rows and marks the selected one', () => {
    renderBar();
    expect(screen.getByText('a.png').closest('.sidebar__row')).toHaveAttribute('data-selected', 'true');
    expect(screen.getByText('b.png')).toBeInTheDocument();
  });

  it('typing in search calls onSearch', () => {
    const p = renderBar();
    fireEvent.change(screen.getByPlaceholderText('搜索文件名…'), { target: { value: 'foo' } });
    expect(p.onSearch).toHaveBeenCalledWith('foo');
  });

  it('clicking a chip calls onFilter', () => {
    const p = renderBar();
    fireEvent.click(screen.getByRole('button', { name: /仅左 1/ }));
    expect(p.onFilter).toHaveBeenCalledWith('left_only');
  });

  it('clicking a row calls onSelect with the item', () => {
    const p = renderBar();
    fireEvent.click(screen.getByText('b.png'));
    expect(p.onSelect).toHaveBeenCalledWith(items[1]);
  });

  it('shows progress and cancel while loading', () => {
    const p = renderBar({ isLoading: true, scanProgress: { stage: 'comparing', done: 87, total: 600 } });
    expect(screen.getByText(/87 \/ 600/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '取消' }));
    expect(p.onCancelScan).toHaveBeenCalled();
  });

  it('empty filtered list shows clear-search hint when query active', () => {
    const p = renderBar({ filteredItems: [], searchQuery: 'zzz' });
    fireEvent.click(screen.getByRole('button', { name: '清空搜索' }));
    expect(p.onSearch).toHaveBeenCalledWith('');
  });
});
```

- [ ] **Step 6: 运行确认失败，然后实现 Sidebar.tsx**

Run: `cd frontend; npx vitest run src/components/Sidebar.test.tsx` → FAIL

```tsx
// frontend/src/components/Sidebar.tsx
import { memo, useEffect, useRef, useState } from 'react';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import type { ActiveFilter, SortKey } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary, ScanProgress } from '../lib/types';

const KIND_DOT: Record<BatchListItemKind, string> = {
  different: 'mod', identical: 'eq', left_only: 'rem', right_only: 'add', error: 'err',
};
const KIND_TAG: Record<BatchListItemKind, (n: number) => string> = {
  different: (n) => String(n), identical: () => '一致', left_only: () => '仅左', right_only: () => '仅右', error: () => '错误',
};
const CHIPS: { id: ActiveFilter; label: (c: DirectorySummary['counts']) => string }[] = [
  { id: 'different', label: (c) => `不一致 ${c.different}` },
  { id: 'left_only', label: (c) => `仅左 ${c.left_only}` },
  { id: 'right_only', label: (c) => `仅右 ${c.right_only}` },
  { id: 'identical', label: (c) => `一致 ${c.identical}` },
  { id: 'error', label: (c) => `错误 ${c.error}` },
  { id: 'all', label: (c) => `全部 ${c.identical + c.different + c.left_only + c.right_only + c.error}` },
];

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function Sidebar({
  leftDir, rightDir, summary, filteredItems, activeFilter, searchQuery, sortKey,
  selectedItemId, isLoading, scanProgress,
  onFilter, onSearch, onSort, onSelect, onPickLeft, onPickRight, onCancelScan,
}: {
  leftDir: string;
  rightDir: string;
  summary: DirectorySummary | null;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  searchQuery: string;
  sortKey: SortKey;
  selectedItemId: string | null;
  isLoading: boolean;
  scanProgress: ScanProgress | null;
  onFilter(f: ActiveFilter): void;
  onSearch(q: string): void;
  onSort(k: SortKey): void;
  onSelect(item: BatchListItem): void;
  onPickLeft(): void;
  onPickRight(): void;
  onCancelScan(): void;
}) {
  const searchRef = useRef<HTMLInputElement>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; item: BatchListItem } | null>(null);

  // Ctrl+F 经全局事件聚焦搜索框
  useEffect(() => {
    const onFocus = () => searchRef.current?.focus();
    document.addEventListener('wb:focusSearch', onFocus);
    return () => document.removeEventListener('wb:focusSearch', onFocus);
  }, []);

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    window.addEventListener('click', close);
    return () => window.removeEventListener('click', close);
  }, [menu]);

  const counts = summary?.counts ?? null;
  const totalPairs = counts
    ? counts.identical + counts.different + counts.left_only + counts.right_only + counts.error
    : 0;
  const diffPos = (() => {
    if (!summary || !selectedItemId) return null;
    const diffs = summary.items.filter((i) => i.kind === 'different');
    const idx = diffs.findIndex((i) => i.id === selectedItemId);
    return idx >= 0 ? `${idx + 1} / ${diffs.length}` : null;
  })();

  return (
    <aside className="sidebar">
      <div className="sidebar__slots">
        <DirChip side="左" path={leftDir} onPick={onPickLeft} />
        <DirChip side="右" path={rightDir} onPick={onPickRight} />
      </div>

      <div className="sidebar__search">
        <input ref={searchRef} type="text" placeholder="搜索文件名…" value={searchQuery}
          onChange={(e) => onSearch(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Escape') { onSearch(''); (e.target as HTMLInputElement).blur(); } }} />
      </div>

      {counts && (
        <div className="sidebar__chips">
          {CHIPS.map((chip) => (
            <button key={chip.id} type="button" className="sidebar__chip"
              data-active={activeFilter === chip.id} onClick={() => onFilter(chip.id)}>
              {chip.label(counts)}
            </button>
          ))}
          <button type="button" className="sidebar__sort" title="切换排序"
            onClick={() => onSort(sortKey === 'diff-desc' ? 'name-asc' : 'diff-desc')}>
            {sortKey === 'diff-desc' ? '↓差异' : 'A-Z'}
          </button>
        </div>
      )}

      {isLoading && (
        <div className="sidebar__progress" role="status" aria-live="polite">
          <span>
            {scanProgress?.stage === 'comparing' && scanProgress.total > 0
              ? `已比对 ${scanProgress.done} / ${scanProgress.total}`
              : '正在扫描目录…'}
          </span>
          <button type="button" className="sidebar__cancel" onClick={onCancelScan}>取消</button>
          <div className="sidebar__progress-track">
            <div className="sidebar__progress-fill" style={
              scanProgress?.stage === 'comparing' && scanProgress.total > 0
                ? { width: `${Math.round((scanProgress.done / scanProgress.total) * 100)}%` }
                : undefined
            } />
          </div>
        </div>
      )}

      <div className="sidebar__rows">
        {filteredItems.length === 0 && !isLoading && (
          <div className="sidebar__empty">
            {searchQuery
              ? <>无匹配 <button type="button" onClick={() => onSearch('')}>清空搜索</button></>
              : summary && counts!.different === 0 && activeFilter === 'all' && totalPairs > 0 && summary.items.every((i) => i.kind === 'identical')
                ? '两侧完全一致'
                : '无条目'}
          </div>
        )}
        {filteredItems.map((item) => (
          <Row key={item.id} item={item} selected={item.id === selectedItemId}
            onSelect={onSelect}
            onMenu={(e) => { e.preventDefault(); setMenu({ x: e.clientX, y: e.clientY, item }); }} />
        ))}
      </div>

      <div className="sidebar__foot">
        {summary ? `${totalPairs} 对${diffPos ? ` · ${diffPos} 不一致` : ''}` : '未扫描'}
      </div>

      {menu && (
        <div className="sidebar__menu" style={{ left: menu.x, top: menu.y }}>
          <button type="button" onClick={() => {
            void navigator.clipboard?.writeText([menu.item.left_path, menu.item.right_path].filter(Boolean).join('\n'));
          }}>复制路径</button>
          <button type="button" onClick={() => {
            const p = menu.item.left_path ?? menu.item.right_path ?? menu.item.label;
            void revealItemInDir(p).catch(() => openPath(p));
          }}>在资源管理器中显示</button>
        </div>
      )}
    </aside>
  );
}

function DirChip({ side, path, onPick }: { side: string; path: string; onPick(): void }) {
  return (
    <div className="sidebar__slot">
      <span className="sidebar__slot-side">{side}</span>
      <button type="button" className="sidebar__slot-path" title={path || '未选择'} onClick={onPick}>
        {path ? `…\\${basename(path)}` : '选择目录'}
      </button>
    </div>
  );
}

const Row = memo(function Row({
  item, selected, onSelect, onMenu,
}: {
  item: BatchListItem; selected: boolean;
  onSelect(item: BatchListItem): void;
  onMenu(e: React.MouseEvent): void;
}) {
  return (
    <div className="sidebar__row" data-selected={selected || undefined} role="button" tabIndex={0}
      title={[item.left_path, item.right_path].filter(Boolean).join('\n')}
      onClick={() => onSelect(item)} onContextMenu={onMenu}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(item); } }}>
      <span className={`sidebar__dot sidebar__dot--${KIND_DOT[item.kind]}`} />
      <span className="sidebar__name">{item.label}</span>
      <span className={`sidebar__tag sidebar__tag--${KIND_DOT[item.kind]}`}>{KIND_TAG[item.kind](item.difference_count)}</span>
    </div>
  );
});
```

- [ ] **Step 7: 追加样式到 app.css 末尾**

```css
/* ============ Sidebar（主从布局左栏） ============ */
.sidebar { width: 260px; flex: none; display: flex; flex-direction: column; background: var(--bg-panel); border-right: 1px solid var(--border-default); min-height: 0; position: relative; }
.sidebar__slots { padding: var(--sp-2); border-bottom: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 4px; }
.sidebar__slot { display: flex; align-items: center; gap: 6px; }
.sidebar__slot-side { color: var(--text-tertiary); font: var(--fs-xs)/1 var(--font-ui); flex: none; }
.sidebar__slot-path { flex: 1; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; text-align: left; background: var(--bg-elevated); border: 1px solid var(--border-subtle); border-radius: var(--r-sm); color: var(--text-secondary); font: var(--fs-sm)/1.6 var(--font-mono); padding: 2px 8px; cursor: pointer; }
.sidebar__slot-path:hover { border-color: var(--accent); }
.sidebar__search { padding: var(--sp-2); }
.sidebar__search input { width: 100%; box-sizing: border-box; background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: var(--r-sm); color: var(--text-primary); font: var(--fs-sm)/1.6 var(--font-mono); padding: 4px 8px; }
.sidebar__search input:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
.sidebar__chips { display: flex; flex-wrap: wrap; gap: 4px; padding: 0 var(--sp-2) var(--sp-2); }
.sidebar__chip { border: 1px solid var(--border-default); background: none; color: var(--text-secondary); border-radius: 9px; padding: 1px 8px; font: var(--fs-xs)/var(--lh-xs) var(--font-ui); cursor: pointer; }
.sidebar__chip[data-active="true"] { background: var(--mod-bg); color: var(--mod-new); border-color: transparent; }
.sidebar__sort { margin-left: auto; border: none; background: none; color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-ui); cursor: pointer; }
.sidebar__sort:hover { color: var(--accent); }
.sidebar__progress { padding: var(--sp-2); color: var(--text-secondary); font: var(--fs-xs)/var(--lh-xs) var(--font-ui); display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
.sidebar__cancel { border: 1px solid var(--border-default); background: none; color: var(--text-secondary); border-radius: var(--r-sm); padding: 1px 8px; cursor: pointer; }
.sidebar__cancel:hover { border-color: var(--rem-text); color: var(--rem-text); }
.sidebar__progress-track { width: 100%; height: 3px; background: var(--bg-elevated); border-radius: 2px; overflow: hidden; }
.sidebar__progress-fill { height: 100%; background: var(--accent); transition: width 120ms linear; }
.sidebar__rows { flex: 1; overflow-y: auto; padding: 0 var(--sp-1); }
.sidebar__row { display: flex; align-items: center; gap: 6px; padding: 4px 8px; border-radius: var(--r-sm); cursor: pointer; }
.sidebar__row:hover { background: var(--bg-hover); }
.sidebar__row[data-selected] { background: var(--accent-bg); }
.sidebar__dot { width: 6px; height: 6px; border-radius: 50%; flex: none; }
.sidebar__dot--mod { background: var(--mod-new); }
.sidebar__dot--add { background: var(--add-text); }
.sidebar__dot--rem { background: var(--rem-text); }
.sidebar__dot--err { background: var(--err-text); }
.sidebar__dot--eq { background: var(--text-tertiary); }
.sidebar__name { flex: 1; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; color: var(--text-secondary); font: var(--fs-sm)/var(--lh-sm) var(--font-mono); }
.sidebar__row[data-selected] .sidebar__name { color: var(--text-primary); }
.sidebar__tag { flex: none; font: var(--fs-xs)/var(--lh-xs) var(--font-mono); }
.sidebar__tag--mod { color: var(--mod-new); }
.sidebar__tag--add { color: var(--add-text); }
.sidebar__tag--rem { color: var(--rem-text); }
.sidebar__tag--err { color: var(--err-text); }
.sidebar__tag--eq { color: var(--text-tertiary); }
.sidebar__empty { color: var(--text-tertiary); font: var(--fs-sm)/var(--lh-sm) var(--font-ui); padding: var(--sp-3); display: flex; gap: 6px; align-items: center; }
.sidebar__empty button { border: none; background: none; color: var(--accent); cursor: pointer; }
.sidebar__foot { border-top: 1px solid var(--border-subtle); padding: 5px 10px; color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-mono); }
.sidebar__menu { position: fixed; z-index: 20; background: var(--bg-elevated); border: 1px solid var(--border-emph); border-radius: var(--r-md); padding: 4px; display: flex; flex-direction: column; min-width: 160px; }
.sidebar__menu button { text-align: left; border: none; background: none; color: var(--text-secondary); font: var(--fs-sm)/1.8 var(--font-ui); padding: 2px 10px; border-radius: var(--r-sm); cursor: pointer; }
.sidebar__menu button:hover { background: var(--bg-hover); color: var(--text-primary); }
```

- [ ] **Step 8: 运行确认通过 + 回归 + 构建**

Run: `cd frontend; npx vitest run; npm run build`
Expected: 全部 PASS、build 通过

- [ ] **Step 9: Commit**

```powershell
git add frontend/src/features/workbench/useWorkbench.ts frontend/src/features/workbench/useWorkbench.test.tsx frontend/src/components/Sidebar.tsx frontend/src/components/Sidebar.test.tsx frontend/src/styles/app.css
git commit -m "feat(sidebar): selection flow, filename search, sort, progress+cancel UI (not yet mounted)"
```

---

## Task 6: recentDirs（MRU）+ WelcomePane

**Files:**
- Create: `frontend/src/lib/recentDirs.ts`
- Create: `frontend/src/components/WelcomePane.tsx`
- Test: `frontend/src/lib/recentDirs.test.ts`、`frontend/src/components/WelcomePane.test.tsx`
- Modify: `frontend/src/features/workbench/useWorkbench.ts`（成功后写 MRU）+ 其测试
- Modify: `frontend/src/styles/app.css`（追加 `.welcome2` 样式块）

- [ ] **Step 1: 写 recentDirs 失败测试**

```ts
// frontend/src/lib/recentDirs.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { loadRecent, touchRecent, removeRecent } from './recentDirs';

describe('recentDirs', () => {
  beforeEach(() => localStorage.clear());

  it('touch adds to front and dedupes by both paths', () => {
    touchRecent('dir', 'L1', 'R1');
    touchRecent('dir', 'L2', 'R2');
    touchRecent('dir', 'L1', 'R1');
    const list = loadRecent('dir');
    expect(list.map((p) => p.left)).toEqual(['L1', 'L2']);
  });

  it('caps at 8 entries', () => {
    for (let i = 0; i < 10; i++) touchRecent('dir', `L${i}`, `R${i}`);
    expect(loadRecent('dir')).toHaveLength(8);
    expect(loadRecent('dir')[0].left).toBe('L9');
  });

  it('file and dir lists are independent', () => {
    touchRecent('dir', 'D', 'D2');
    touchRecent('file', 'F', 'F2');
    expect(loadRecent('dir')).toHaveLength(1);
    expect(loadRecent('file')).toHaveLength(1);
  });

  it('remove drops the matching pair', () => {
    touchRecent('dir', 'L1', 'R1');
    touchRecent('dir', 'L2', 'R2');
    removeRecent('dir', 'L1', 'R1');
    expect(loadRecent('dir').map((p) => p.left)).toEqual(['L2']);
  });

  it('survives corrupted storage', () => {
    localStorage.setItem('recent.dirPairs', '{not json');
    expect(loadRecent('dir')).toEqual([]);
  });
});
```

- [ ] **Step 2: 运行确认失败，然后实现 recentDirs.ts**

Run: `cd frontend; npx vitest run src/lib/recentDirs.test.ts` → FAIL

```ts
// frontend/src/lib/recentDirs.ts
export interface RecentPair {
  left: string;
  right: string;
  lastUsed: number;   // epoch ms
}

export type RecentKind = 'dir' | 'file';

const KEYS: Record<RecentKind, string> = {
  dir: 'recent.dirPairs',
  file: 'recent.filePairs',
};
const CAP = 8;

export function loadRecent(kind: RecentKind): RecentPair[] {
  try {
    const raw = localStorage.getItem(KEYS[kind]);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (p): p is RecentPair =>
        typeof p?.left === 'string' && typeof p?.right === 'string' && typeof p?.lastUsed === 'number',
    );
  } catch {
    return [];
  }
}

export function touchRecent(kind: RecentKind, left: string, right: string): void {
  const rest = loadRecent(kind).filter((p) => !(p.left === left && p.right === right));
  const next = [{ left, right, lastUsed: Date.now() }, ...rest].slice(0, CAP);
  try { localStorage.setItem(KEYS[kind], JSON.stringify(next)); } catch { /* storage full: 忽略 */ }
}

export function removeRecent(kind: RecentKind, left: string, right: string): void {
  const next = loadRecent(kind).filter((p) => !(p.left === left && p.right === right));
  try { localStorage.setItem(KEYS[kind], JSON.stringify(next)); } catch { /* ignore */ }
}
```

Run: `cd frontend; npx vitest run src/lib/recentDirs.test.ts` → PASS

- [ ] **Step 3: useWorkbench 成功后写 MRU（测试先行）**

测试（追加到 useWorkbench.test.tsx，`beforeEach(() => localStorage.clear())` 放入该 describe）：

```tsx
import { loadRecent } from '../../lib/recentDirs';

describe('MRU writes', () => {
  beforeEach(() => localStorage.clear());

  it('writes dir pair after successful scan', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    expect(loadRecent('dir')).toMatchObject([{ left: '/left', right: '/right' }]);
  });

  it('writes file pair after successful single compare', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runCompare(); });
    expect(loadRecent('file')).toMatchObject([{ left: '/a.png', right: '/b.png' }]);
  });
});
```

实现：`useWorkbench.ts` 顶部 `import { touchRecent } from '../../lib/recentDirs';`；`runAuto` 单文件 `left && right` 成功分支（`setSlotBarCollapsed(true);` 后）加 `touchRecent('file', left, right);`；目录成功分支（自动选中代码前）加 `touchRecent('dir', left, right);`。

Run: `cd frontend; npx vitest run src/features/workbench/useWorkbench.test.tsx` → PASS

- [ ] **Step 4: 写 WelcomePane 失败测试**

```tsx
// frontend/src/components/WelcomePane.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WelcomePane } from './WelcomePane';
import { touchRecent } from '../lib/recentDirs';

describe('WelcomePane', () => {
  beforeEach(() => localStorage.clear());

  it('lists recent pairs for the current mode and applies on click', () => {
    touchRecent('dir', 'C:/tmp/bim_v1', 'C:/tmp/bim_v2');
    const onApplyPair = vi.fn();
    render(<WelcomePane mode="directory" onApplyPair={onApplyPair} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    fireEvent.click(screen.getByText(/bim_v1/));
    expect(onApplyPair).toHaveBeenCalledWith('C:/tmp/bim_v1', 'C:/tmp/bim_v2');
  });

  it('removes an entry via its × button without applying', () => {
    touchRecent('dir', 'C:/a', 'C:/b');
    const onApplyPair = vi.fn();
    render(<WelcomePane mode="directory" onApplyPair={onApplyPair} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: '删除该记录' }));
    expect(onApplyPair).not.toHaveBeenCalled();
    expect(screen.queryByText(/C:\/a/)).toBeNull();
  });

  it('shows mode-appropriate hint', () => {
    render(<WelcomePane mode="single" onApplyPair={() => {}} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    expect(screen.getByText(/PNG 文件/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 5: 运行确认失败，然后实现 WelcomePane.tsx**

Run: `cd frontend; npx vitest run src/components/WelcomePane.test.tsx` → FAIL

```tsx
// frontend/src/components/WelcomePane.tsx
import { useState } from 'react';
import { loadRecent, removeRecent } from '../lib/recentDirs';
import type { RecentKind, RecentPair } from '../lib/recentDirs';
import type { Side, WorkbenchMode } from '../lib/types';

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

function relTime(ts: number): string {
  const mins = Math.round((Date.now() - ts) / 60000);
  if (mins < 1) return '刚刚';
  if (mins < 60) return `${mins} 分钟前`;
  const hours = Math.round(mins / 60);
  if (hours < 24) return `${hours} 小时前`;
  return `${Math.round(hours / 24)} 天前`;
}

export function WelcomePane({
  mode, onApplyPair, onDrop, onPickLeft, onPickRight,
}: {
  mode: WorkbenchMode;
  onApplyPair(left: string, right: string): void;
  onDrop(side: Side, path: string): void;
  onPickLeft(): void;
  onPickRight(): void;
}) {
  const kind: RecentKind = mode === 'directory' ? 'dir' : 'file';
  const [recent, setRecent] = useState<RecentPair[]>(() => loadRecent(kind));
  const noun = mode === 'directory' ? '文件夹' : 'PNG 文件';

  const dropHandler = (side: Side) => (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    const p = (file as unknown as { path?: string })?.path;
    if (p) onDrop(side, p);
  };

  return (
    <div className="welcome2">
      <div className="welcome2__title">PNG Compare</div>
      <div className="welcome2__slots">
        {(['left', 'right'] as const).map((side) => (
          <div key={side} className="welcome2__slot"
            onDragOver={(e) => e.preventDefault()} onDrop={dropHandler(side)}>
            <span>拖入{side === 'left' ? '左侧' : '右侧'}{noun}，或</span>
            <button type="button" onClick={side === 'left' ? onPickLeft : onPickRight}>浏览</button>
          </div>
        ))}
      </div>

      {recent.length > 0 && (
        <div className="welcome2__recent">
          <div className="welcome2__recent-title">最近使用</div>
          {recent.map((p) => (
            <div key={`${p.left}|${p.right}`} className="welcome2__recent-row">
              <button type="button" className="welcome2__recent-main"
                title={`${p.left}\n${p.right}`}
                onClick={() => onApplyPair(p.left, p.right)}>
                <span className="welcome2__recent-name">{basename(p.left)} ⇄ {basename(p.right)}</span>
                <span className="welcome2__recent-time">{relTime(p.lastUsed)}</span>
              </button>
              <button type="button" className="welcome2__recent-del" aria-label="删除该记录"
                onClick={() => { removeRecent(kind, p.left, p.right); setRecent(loadRecent(kind)); }}>×</button>
            </div>
          ))}
        </div>
      )}

      <div className="welcome2__hint">
        <kbd>Ctrl+O</kbd> 选左 · <kbd>Ctrl+Shift+O</kbd> 选右 · <kbd>↑</kbd>/<kbd>↓</kbd> 列表穿梭 · <kbd>n</kbd>/<kbd>p</kbd> 跳差异 · <kbd>1/2/3</kbd> 视图 · <kbd>F</kbd> 仅看不同 · <kbd>D</kbd> 高亮
      </div>
    </div>
  );
}
```

样式（app.css 末尾）：

```css
/* ============ WelcomePane ============ */
.welcome2 { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--sp-5); height: 100%; padding: var(--sp-6); }
.welcome2__title { font: 500 var(--fs-2xl)/var(--lh-2xl) var(--font-ui); color: var(--text-primary); }
.welcome2__slots { display: flex; gap: var(--sp-4); width: min(720px, 90%); }
.welcome2__slot { flex: 1; border: 1px dashed var(--border-emph); border-radius: var(--r-md); padding: var(--sp-5); display: flex; gap: 8px; align-items: center; justify-content: center; color: var(--text-secondary); font: var(--fs-md)/var(--lh-md) var(--font-ui); }
.welcome2__slot button { border: 1px solid var(--border-default); background: var(--bg-elevated); color: var(--text-primary); border-radius: var(--r-sm); padding: 3px 12px; cursor: pointer; }
.welcome2__slot button:hover { border-color: var(--accent); }
.welcome2__recent { width: min(560px, 90%); display: flex; flex-direction: column; gap: 2px; }
.welcome2__recent-title { color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-ui); margin-bottom: 4px; }
.welcome2__recent-row { display: flex; align-items: center; gap: 4px; }
.welcome2__recent-main { flex: 1; min-width: 0; display: flex; justify-content: space-between; gap: 12px; border: none; background: none; color: var(--text-secondary); font: var(--fs-sm)/1.9 var(--font-mono); padding: 2px 10px; border-radius: var(--r-sm); cursor: pointer; }
.welcome2__recent-main:hover { background: var(--bg-hover); color: var(--text-primary); }
.welcome2__recent-name { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.welcome2__recent-time { flex: none; color: var(--text-tertiary); }
.welcome2__recent-del { border: none; background: none; color: var(--text-tertiary); cursor: pointer; }
.welcome2__recent-del:hover { color: var(--rem-text); }
.welcome2__hint { color: var(--text-tertiary); font: var(--fs-sm)/2 var(--font-ui); }
.welcome2__hint kbd { background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: var(--r-sm); padding: 0 5px; font-family: var(--font-mono); }
```

- [ ] **Step 6: Sidebar 目录芯片 → MRU 浮层 + 粘贴路径（spec §2 MRU / §4 绕路通道）**

测试先行，`Sidebar.test.tsx` 追加（文件顶部补 `import { touchRecent } from '../lib/recentDirs';`，describe 内补 `beforeEach(() => localStorage.clear());`；`renderBar` 工厂 props 追加 `onApplyPair: vi.fn(), onPastePath: vi.fn(),`）：

```tsx
it('dir chip opens a menu with recent pairs and applies one', () => {
  touchRecent('dir', 'C:/x1', 'C:/x2');
  const p = renderBar();
  fireEvent.click(screen.getByTitle('C:/tmp/bim_v1'));
  fireEvent.click(screen.getByText(/x1 ⇄ x2/));
  expect(p.onApplyPair).toHaveBeenCalledWith('C:/x1', 'C:/x2');
});

it('pasting a path + Enter calls onPastePath for that side', () => {
  const p = renderBar();
  fireEvent.click(screen.getByTitle('C:/tmp/bim_v1'));
  const input = screen.getByPlaceholderText('粘贴路径后回车');
  fireEvent.change(input, { target: { value: 'D:/new' } });
  fireEvent.keyDown(input, { key: 'Enter' });
  expect(p.onPastePath).toHaveBeenCalledWith('left', 'D:/new');
});
```

Run: `cd frontend; npx vitest run src/components/Sidebar.test.tsx` → FAIL 后实现：

`Sidebar.tsx` 改动——props 追加：

```ts
onApplyPair(left: string, right: string): void;
onPastePath(side: 'left' | 'right', path: string): void;
```

顶部补 `import { loadRecent } from '../lib/recentDirs';`；组件内加 `const [pickMenu, setPickMenu] = useState<'left' | 'right' | null>(null);`；`.sidebar__slots` 块替换为：

```tsx
<div className="sidebar__slots">
  <DirChip label="左" path={leftDir} onOpen={() => setPickMenu(pickMenu === 'left' ? null : 'left')} />
  <DirChip label="右" path={rightDir} onOpen={() => setPickMenu(pickMenu === 'right' ? null : 'right')} />
  {pickMenu && (
    <div className="sidebar__pickmenu">
      <button type="button" onClick={() => { (pickMenu === 'left' ? onPickLeft : onPickRight)(); setPickMenu(null); }}>浏览…</button>
      <input type="text" placeholder="粘贴路径后回车"
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            const v = (e.target as HTMLInputElement).value.trim();
            if (v) { onPastePath(pickMenu, v); setPickMenu(null); }
          } else if (e.key === 'Escape') setPickMenu(null);
        }} />
      {loadRecent('dir').map((p) => (
        <button key={`${p.left}|${p.right}`} type="button" title={`${p.left}\n${p.right}`}
          onClick={() => { onApplyPair(p.left, p.right); setPickMenu(null); }}>
          {basename(p.left)} ⇄ {basename(p.right)}
        </button>
      ))}
    </div>
  )}
</div>
```

`DirChip` 相应简化（`onPick` 改名 `onOpen`，去掉 `side` 形参）：

```tsx
function DirChip({ label, path, onOpen }: { label: string; path: string; onOpen(): void }) {
  return (
    <div className="sidebar__slot">
      <span className="sidebar__slot-side">{label}</span>
      <button type="button" className="sidebar__slot-path" title={path || '未选择'} onClick={onOpen}>
        {path ? `…\\${basename(path)}` : '选择目录'}
      </button>
    </div>
  );
}
```

Task 5 中 `renderBar` 既有用例若因新必填 props 报 TS 错，统一由工厂补默认值解决。样式追加到 app.css：

```css
.sidebar__pickmenu { position: absolute; top: 60px; left: 8px; right: 8px; z-index: 15; background: var(--bg-elevated); border: 1px solid var(--border-emph); border-radius: var(--r-md); padding: 6px; display: flex; flex-direction: column; gap: 4px; }
.sidebar__pickmenu button { text-align: left; border: none; background: none; color: var(--text-secondary); font: var(--fs-sm)/1.9 var(--font-mono); padding: 2px 8px; border-radius: var(--r-sm); cursor: pointer; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.sidebar__pickmenu button:hover { background: var(--bg-hover); color: var(--text-primary); }
.sidebar__pickmenu input { background: var(--bg-page); border: 1px solid var(--border-default); border-radius: var(--r-sm); color: var(--text-primary); font: var(--fs-sm)/1.6 var(--font-mono); padding: 3px 8px; }
```

Run: `cd frontend; npx vitest run src/components/Sidebar.test.tsx` → PASS

- [ ] **Step 7: 运行确认通过 + 回归**

Run: `cd frontend; npx vitest run`
Expected: 全部 PASS

- [ ] **Step 8: Commit**

```powershell
git add frontend/src/lib/recentDirs.ts frontend/src/lib/recentDirs.test.ts frontend/src/components/WelcomePane.tsx frontend/src/components/WelcomePane.test.tsx frontend/src/components/Sidebar.tsx frontend/src/components/Sidebar.test.tsx frontend/src/features/workbench/useWorkbench.ts frontend/src/features/workbench/useWorkbench.test.tsx frontend/src/styles/app.css
git commit -m "feat(welcome+sidebar): MRU recent pairs, dir-chip quick menu with paste-path"
```

---

## Task 7: App 三栏骨架接线 + 键盘地图 + 视图收编

这是集成任务：把 Sidebar / UnifiedTree / DiffRail / WelcomePane 挂进 App，删除整页切换语义。

**Files:**
- Modify: `frontend/src/App.tsx`（大改）
- Modify: `frontend/src/features/workbench/useWorkbench.ts`（view 枚举、键盘地图、删 inDirectorySubview/goBackToDirectory/slotBarCollapsed）
- Modify: `frontend/src/features/workbench/useWorkbench.test.tsx`（适配）
- Modify: `frontend/src/styles/app.css`（三栏骨架 + 详情头部 + 错误卡样式）

- [ ] **Step 1: useWorkbench 收编（测试先行改写）**

变更点：
1. `AppView` 改为 `'welcome' | 'solo' | 'mirror' | 'error'`；`selectItem` 错误分支 `setView('error')`；目录扫描成功后 `setView` 不再切 `'directory-overview'`（保持/落到 `'welcome'`，由自动选中把它带进 `'mirror'/'solo'/'error'`）。
2. 删除：`inDirectorySubview`、`goBackToDirectory`、`slotBarCollapsed`、`toggleSlotBarCollapsed`、`directoryContext`（footer 的 n/N 由 Sidebar 自行从 summary+selectedItemId 推导，已在 Task 5 实现）。
3. 键盘地图改为设计稿 §2（整体替换现 `onKey`）：

```ts
useEffect(() => {
  const onKey = (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    const k = e.key.toLowerCase();
    if (e.ctrlKey && !e.shiftKey && k === 'o') {
      e.preventDefault(); document.dispatchEvent(new CustomEvent('wb:pickLeft'));
    } else if (e.ctrlKey && e.shiftKey && k === 'o') {
      e.preventDefault(); document.dispatchEvent(new CustomEvent('wb:pickRight'));
    } else if (e.ctrlKey && k === 'f') {
      e.preventDefault(); document.dispatchEvent(new CustomEvent('wb:focusSearch'));
    } else if (e.ctrlKey && !e.shiftKey && k === 'b') {
      e.preventDefault(); setSidebarCollapsed((v) => !v);
    } else if (e.ctrlKey && e.shiftKey && k === 'b') {
      e.preventDefault(); setRailCollapsed((v) => !v);
    } else if (e.ctrlKey && e.key === 'Enter') {
      e.preventDefault(); void runAuto();
    } else if (e.key === 'ArrowDown' && directorySummary) {
      e.preventDefault(); void selectNext();
    } else if (e.key === 'ArrowUp' && directorySummary) {
      e.preventDefault(); void selectPrev();
    } else if (k === 'n' || k === 'p') {
      document.dispatchEvent(new CustomEvent('wb:diffJump', { detail: k === 'n' ? 1 : -1 }));
    } else if (e.key === '1') setViewMode('tree');
    else if (e.key === '2') setViewMode('json');
    else if (e.key === '3') setViewMode('image');
    else if (k === 'f' && view === 'mirror') toggleOnlyDiff();
    else if (k === 'd' && view === 'mirror') toggleDiffHighlight();
  };
  window.addEventListener('keydown', onKey);
  return () => window.removeEventListener('keydown', onKey);
});
```

4. 既有测试适配：删除断言 `inDirectorySubview`/`goBackToDirectory`/`slotBarCollapsed`/`directoryContext` 的用例（或改写为新行为断言）；`'directory-overview'` 断言改为「扫描成功后 `directorySummary` 非空 + 自动选中生效」。涉及用例逐个改，不留 skip。

- [ ] **Step 2: 运行 hook 测试**

Run: `cd frontend; npx vitest run src/features/workbench/useWorkbench.test.tsx`
Expected: PASS

- [ ] **Step 3: 重写 App.tsx**

```tsx
// frontend/src/App.tsx
import { useEffect, useMemo, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { Sidebar } from './components/Sidebar';
import { UnifiedTree } from './components/UnifiedTree';
import type { FocusRequest } from './components/UnifiedTree';
import { DiffRail } from './components/DiffRail';
import { WelcomePane } from './components/WelcomePane';
import { useWorkbench } from './features/workbench/useWorkbench';
import { buildMirrorRows } from './lib/treeModel';
import { buildDiffEntries } from './lib/diffList';

const win = getCurrentWindow();
const RAIL_AUTO_COLLAPSE_WIDTH = 1000;

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

  useEffect(() => {
    if (wb.leftInput || wb.rightInput) void wb.runAuto();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wb.leftInput, wb.rightInput, wb.mode]);

  // 窗口过窄自动收起差异栏；手动操作后本会话内尊重手动状态
  const [railManual, setRailManual] = useState(false);
  useEffect(() => {
    const onResize = () => {
      if (railManual) return;
      wb.setRailCollapsed(window.innerWidth < RAIL_AUTO_COLLAPSE_WIDTH);
    };
    onResize();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [railManual]);

  // ===== 行模型单处计算，树与差异栏共享（§5） =====
  const pairRows = useMemo(
    () => (wb.pairResult
      ? buildMirrorRows(wb.pairResult.left.metadata, wb.pairResult.right.metadata, wb.pairResult.diff_root)
      : null),
    [wb.pairResult],
  );
  const soloRows = useMemo(
    () => (wb.soloResult?.metadata !== undefined && wb.soloResult?.metadata !== null
      ? buildMirrorRows(
          wb.soloSide === 'right' ? null : wb.soloResult.metadata,
          wb.soloSide === 'right' ? wb.soloResult.metadata : null,
          null,
        )
      : null),
    [wb.soloResult, wb.soloSide],
  );
  const diffEntries = useMemo(() => (pairRows ? buildDiffEntries(pairRows) : []), [pairRows]);

  // ===== 差异跳转（n/p 与差异栏点击共用） =====
  const [focusRequest, setFocusRequest] = useState<FocusRequest | null>(null);
  const jumpTo = (path: string) =>
    setFocusRequest((cur) => ({ path, seq: (cur?.seq ?? 0) + 1 }));
  useEffect(() => {
    const onJump = (e: Event) => {
      const dir = (e as CustomEvent<number>).detail;
      if (diffEntries.length === 0) return;
      setFocusRequest((cur) => {
        const curIdx = cur ? diffEntries.findIndex((d) => d.path === cur.path) : -1;
        const next = (curIdx + dir + diffEntries.length) % diffEntries.length;
        return { path: diffEntries[next].path, seq: (cur?.seq ?? 0) + 1 };
      });
    };
    document.addEventListener('wb:diffJump', onJump);
    return () => document.removeEventListener('wb:diffJump', onJump);
  }, [diffEntries]);

  const showSidebar = isDir && (wb.directorySummary !== null || wb.isLoading) && !wb.sidebarCollapsed;
  const showRail = wb.view === 'mirror' && wb.pairResult !== null;
  const showWelcome = wb.view === 'welcome' && !(isDir && wb.isLoading);

  return (
    <div className="app-shell">
      <header className="topbar" data-tauri-drag-region>
        <div className="topbar-left">
          <img className="brand-icon" src="/app-icon.png" alt="" draggable={false} />
          <span className="brand">PNG Compare</span>
          <div className="topbar-vsep" />
          <div className="mode-toggle" role="group" aria-label="模式">
            <button type="button" className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
              onClick={() => wb.setMode('single')}>单文件</button>
            <button type="button" className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
              onClick={() => wb.setMode('directory')}>目录</button>
          </div>
          {isDir && wb.directorySummary && (
            <button type="button" className="topbar-collapse" title="收起/展开侧栏 (Ctrl+B)"
              onClick={wb.toggleSidebarCollapsed}>{wb.sidebarCollapsed ? '⇤' : '⇥'}</button>
          )}
        </div>
        <div className="topbar-right">
          <div className="win-controls">
            <button type="button" className="win-btn" onClick={() => void win.minimize()} aria-label="最小化">─</button>
            <button type="button" className="win-btn" onClick={() => void win.toggleMaximize()} aria-label="最大化">□</button>
            <button type="button" className="win-btn win-btn--close" onClick={() => void win.close()} aria-label="关闭">✕</button>
          </div>
        </div>
      </header>

      {wb.error && <div className="banner banner--error">{wb.error}</div>}

      <div className="shell-body">
        {showSidebar && (
          <Sidebar
            leftDir={wb.leftInput} rightDir={wb.rightInput}
            summary={wb.directorySummary} filteredItems={wb.filteredItems}
            activeFilter={wb.activeFilter} searchQuery={wb.searchQuery} sortKey={wb.sortKey}
            selectedItemId={wb.selectedItemId} isLoading={wb.isLoading} scanProgress={wb.scanProgress}
            onFilter={wb.setActiveFilter} onSearch={wb.setSearchQuery} onSort={wb.setSortKey}
            onSelect={(item) => void wb.selectItem(item)}
            onPickLeft={() => void handlePickLeft()} onPickRight={() => void handlePickRight()}
            onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }}
            onPastePath={(side, p) => wb.tryDropPath(side, p)}
            onCancelScan={() => void wb.cancelScan()}
          />
        )}

        <main className="center">
          {showWelcome && (
            <WelcomePane mode={wb.mode}
              onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }}
              onDrop={(side, p) => wb.tryDropPath(side, p)}
              onPickLeft={() => void handlePickLeft()} onPickRight={() => void handlePickRight()} />
          )}

          {(wb.view === 'mirror' || wb.view === 'solo') && (
            <DetailHeader wb={wb} diffCount={diffEntries.length} />
          )}

          {wb.view === 'solo' && wb.soloResult && (
            soloRows
              ? (wb.viewMode === 'image'
                  ? <SingleImage path={wb.soloResult.file_path} name={wb.soloResult.file_name} />
                  : wb.viewMode === 'json'
                    ? <RawJsonSplit left={wb.soloSide === 'left' ? wb.soloResult.raw_json : null}
                        right={wb.soloSide === 'right' ? wb.soloResult.raw_json : null} solo={wb.soloSide} />
                    : <UnifiedTree rows={soloRows} solo={wb.soloSide} highlight={false} onlyDiff={false}
                        leftLabel={wb.soloSide === 'left' ? wb.soloResult.file_name : ''}
                        rightLabel={wb.soloSide === 'right' ? wb.soloResult.file_name : ''}
                        focusRequest={null} />)
              : <div className="banner banner--error">该文件不含嵌入式元数据。</div>
          )}

          {wb.view === 'mirror' && wb.pairResult && pairRows && (
            <>
              {wb.viewMode === 'tree' && (
                <UnifiedTree rows={pairRows} solo={null}
                  highlight={wb.diffHighlight} onlyDiff={wb.onlyDiff}
                  leftLabel={wb.pairResult.left.file_name} rightLabel={wb.pairResult.right.file_name}
                  focusRequest={focusRequest} />
              )}
              {wb.viewMode === 'json' && (
                <RawJsonSplit left={wb.pairResult.left.raw_json} right={wb.pairResult.right.raw_json} solo={null} />
              )}
              {wb.viewMode === 'image' && (
                <ImageSplit
                  leftPath={wb.pairResult.left.file_path} rightPath={wb.pairResult.right.file_path}
                  leftName={wb.pairResult.left.file_name} rightName={wb.pairResult.right.file_name} />
              )}
            </>
          )}

          {wb.view === 'error' && wb.errorItem && <ErrorCard item={wb.errorItem} />}
        </main>

        {showRail && (
          <DiffRail entries={diffEntries} onJump={jumpTo}
            collapsed={wb.railCollapsed}
            onToggle={() => { setRailManual(true); wb.toggleRailCollapsed(); }} />
        )}
      </div>

      {wb.toast && <div className="toast">{wb.toast}</div>}
    </div>
  );
}

function DetailHeader({ wb, diffCount }: { wb: ReturnType<typeof useWorkbench>; diffCount: number }) {
  const isSingle = wb.mode === 'single';
  const name = wb.view === 'mirror'
    ? wb.pairResult?.left.file_name
    : wb.soloResult?.file_name;
  return (
    <div className="detail-head">
      <div className="detail-head__seg" role="group" aria-label="视图模式">
        <button data-active={wb.viewMode === 'tree'} onClick={() => wb.setViewMode('tree')}>树</button>
        <button data-active={wb.viewMode === 'json'} onClick={() => wb.setViewMode('json')}>JSON</button>
        <button data-active={wb.viewMode === 'image'} onClick={() => wb.setViewMode('image')}>图片</button>
      </div>
      {wb.view === 'mirror' && (
        <>
          <button className="detail-head__btn" data-active={wb.onlyDiff} onClick={wb.toggleOnlyDiff}>仅看不同</button>
          <button className="detail-head__btn" data-active={wb.diffHighlight} onClick={wb.toggleDiffHighlight}>高亮</button>
        </>
      )}
      <span className="detail-head__name" title={name ?? ''}>
        {wb.view === 'solo' && `仅查看${wb.soloSide === 'left' ? '左' : '右'} · `}{name}
      </span>
      <span className="detail-head__spacer" />
      {isSingle && wb.view === 'mirror' && wb.pairResult && (
        <span className="detail-head__chips">
          <button type="button" title={wb.pairResult.left.file_path}
            onClick={() => document.dispatchEvent(new CustomEvent('wb:pickLeft'))}>左 ⌁ {wb.pairResult.left.file_name}</button>
          <button type="button" title={wb.pairResult.right.file_path}
            onClick={() => document.dispatchEvent(new CustomEvent('wb:pickRight'))}>右 ⌁ {wb.pairResult.right.file_name}</button>
        </span>
      )}
      {wb.view === 'mirror' && (
        <span className="detail-head__hint">{diffCount > 0 ? `n/p 跳差异 · ${diffCount} 处` : '无差异'}</span>
      )}
    </div>
  );
}

function ErrorCard({ item }: { item: import('./lib/types').BatchListItem }) {
  const target = item.left_path ?? item.right_path ?? item.label;
  return (
    <div className="error-card">
      <div className="error-card__title">无法解析此文件</div>
      <div className="error-card__path">{item.label}</div>
      {item.message && <div className="error-card__msg">{item.message}</div>}
      <button type="button" className="error-card__open" onClick={() => void openPath(target)}>打开文件 ↗</button>
    </div>
  );
}

function RawJsonSplit({ left, right, solo }: { left: string | null; right: string | null; solo: 'left' | 'right' | null }) {
  const leftText = useMemo(() => format(left), [left]);
  const rightText = useMemo(() => format(right), [right]);
  if (solo === 'left') return <pre className="raw-json raw-json--solo">{leftText}</pre>;
  if (solo === 'right') return <pre className="raw-json raw-json--solo">{rightText}</pre>;
  return (
    <div className="mirror-grid">
      <pre className="raw-json">{leftText}</pre>
      <pre className="raw-json">{rightText}</pre>
    </div>
  );
}

function format(raw: string | null): string {
  if (!raw) return '— 无 JSON —';
  try { return JSON.stringify(JSON.parse(raw), null, 2); } catch { return raw; }
}

function SingleImage({ path, name }: { path: string; name: string }) {
  return (
    <div className="image-split">
      <div className="image-split__panes">
        <ImagePane path={path} name={name} transform="none" />
      </div>
    </div>
  );
}
```

`ImageSplit` 函数：从现 App.tsx **原样保留**（带同步缩放/平移逻辑，含 `useRef` 引入），追加在文件尾部，代码与现版本一致（见现 `App.tsx:285-335`）；两处 `<ImagePane …/>` 调用加 `key={leftPath}` / `key={rightPath}`（换文件时重置加载失败状态）。`ImagePane` 改为可见的加载失败占位（spec §5，替换静默 `data-broken`）：

```tsx
function ImagePane({ path, name, transform }: { path: string; name: string; transform: string }) {
  const url = convertFileSrc(path);
  const [broken, setBroken] = useState(false);
  return (
    <div className="image-pane">
      <div className="image-pane__viewport">
        {broken ? (
          <div className="image-pane__broken">无法加载图片</div>
        ) : (
          <img className="image-pane__img" src={url} alt={name} draggable={false}
            style={{ transform, transformOrigin: 'center center' }}
            onError={() => setBroken(true)} />
        )}
      </div>
      <div className="image-pane__bar">
        <span className="image-pane__name">{name}</span>
        <button type="button" className="detail-head__btn" onClick={() => void openPath(path)}>打开原文件 ↗</button>
      </div>
    </div>
  );
}
```

`SingleImage` 内的 `<ImagePane>` 同样加 `key={path}`。配套样式（并入 Step 4 的 css 块）：

```css
.image-pane__broken { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-tertiary); font: var(--fs-sm)/1 var(--font-ui); }
```

- [ ] **Step 4: 追加骨架样式（app.css 末尾），并把 `body`/`.app-shell` 既有底色改为 `var(--bg-page)`（如未已是）**

```css
/* ============ 三栏骨架 / 详情头部 / 错误卡 ============ */
.shell-body { display: flex; flex: 1; min-height: 0; }
.center { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; background: var(--bg-page); }
.topbar-collapse { border: none; background: none; color: var(--text-tertiary); cursor: pointer; font-size: var(--fs-md); }
.topbar-collapse:hover { color: var(--accent); }
.detail-head { display: flex; align-items: center; gap: var(--sp-2); padding: 6px 10px; border-bottom: 1px solid var(--border-subtle); }
.detail-head__seg { display: flex; border: 1px solid var(--border-default); border-radius: var(--r-sm); overflow: hidden; }
.detail-head__seg button { border: none; background: none; color: var(--text-secondary); font: var(--fs-sm)/1.6 var(--font-ui); padding: 2px 10px; cursor: pointer; }
.detail-head__seg button[data-active="true"] { background: var(--accent-bg); color: var(--text-primary); }
.detail-head__btn { border: 1px solid var(--border-default); background: none; color: var(--text-secondary); border-radius: var(--r-sm); font: var(--fs-sm)/1.6 var(--font-ui); padding: 2px 10px; cursor: pointer; }
.detail-head__btn[data-active="true"] { background: var(--accent-bg); color: var(--text-primary); border-color: transparent; }
.detail-head__name { color: var(--text-secondary); font: var(--fs-sm)/1.6 var(--font-mono); overflow: hidden; white-space: nowrap; text-overflow: ellipsis; max-width: 40%; }
.detail-head__spacer { flex: 1; }
.detail-head__chips { display: flex; gap: 4px; }
.detail-head__chips button { border: 1px solid var(--border-subtle); background: var(--bg-elevated); color: var(--text-secondary); border-radius: var(--r-sm); font: var(--fs-xs)/1.6 var(--font-mono); padding: 1px 8px; cursor: pointer; max-width: 180px; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
.detail-head__hint { color: var(--text-tertiary); font: var(--fs-xs)/var(--lh-xs) var(--font-ui); }
.raw-json--solo { width: 100%; }
.error-card { margin: var(--sp-6) auto; max-width: 560px; background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: var(--r-md); padding: var(--sp-5); display: flex; flex-direction: column; gap: var(--sp-2); }
.error-card__title { color: var(--err-text); font: 500 var(--fs-lg)/var(--lh-lg) var(--font-ui); }
.error-card__path { color: var(--text-secondary); font: var(--fs-sm)/var(--lh-sm) var(--font-mono); word-break: break-all; }
.error-card__msg { color: var(--text-tertiary); font: var(--fs-sm)/var(--lh-sm) var(--font-mono); }
.error-card__open { align-self: flex-start; border: 1px solid var(--border-default); background: none; color: var(--text-secondary); border-radius: var(--r-sm); padding: 4px 12px; cursor: pointer; }
.error-card__open:hover { border-color: var(--accent); color: var(--accent); }
```

- [ ] **Step 5: 全量测试 + 构建 + 手动验证**

Run: `cd frontend; npx vitest run; npm run build`
Expected: 全绿、build 通过。
Run: `cargo tauri dev` 手动走：拖两个目录 → 自动扫描 → 自动选中第一个不一致 → ↑↓ 穿梭 → 搜索过滤 → n/p 树内跳差异 + 闪烁 → 差异栏点击跳转 → 复制差异清单 → Ctrl+B / Ctrl+Shift+B → 单文件模式两个 PNG → 图片/JSON 视图。

- [ ] **Step 6: Commit**

```powershell
git add frontend/src
git commit -m "feat(shell): three-column master-detail layout, new keyboard map, shared row model"
```

---

## Task 8: 后端扫描取消 + 前端接线

**Files:**
- Modify: `src/inspection.rs`（可取消版扫描 + 测试）
- Modify: `src/desktop_api.rs`（代号机制 + `cancel_scan` 命令）
- Modify: `src/main.rs`（注册命令）
- Modify: `frontend/src/lib/api.ts`、`frontend/src/features/workbench/useWorkbench.ts` + 测试

- [ ] **Step 1: Rust 失败测试（inspection.rs tests 模块内追加）**

```rust
#[test]
fn cancellable_scan_stops_early_and_reports_cancelled() {
    let fixture = BatchFixture::new("cancel");
    for i in 0..6 {
        let name = format!("f{i}.png");
        fixture.write_left_png(&name, "shared", r#"{"Title":"L"}"#);
        fixture.write_right_png(&name, "shared", r#"{"Title":"R"}"#);
    }

    let result = scan_directory_summary_cancellable(
        fixture.left_dir(),
        fixture.right_dir(),
        |_p| {},
        || true, // 立即取消
    );

    assert!(matches!(result, Err(ScanCancelled)));
}

#[test]
fn cancellable_scan_with_no_cancel_matches_plain_scan() {
    let fixture = BatchFixture::new("cancel_noop");
    fixture.write_left_png("a.png", "shared", r#"{"Title":"A"}"#);
    fixture.write_right_png("a.png", "shared", r#"{"Title":"A"}"#);

    let result =
        scan_directory_summary_cancellable(fixture.left_dir(), fixture.right_dir(), |_p| {}, || false)
            .expect("not cancelled");
    assert_eq!(result, scan_directory_summary(fixture.left_dir(), fixture.right_dir()));
}
```

（test 模块 use 行追加 `scan_directory_summary_cancellable, ScanCancelled`。）

Run: `cargo test cancellable` → Expected: FAIL（符号不存在）

- [ ] **Step 2: 实现可取消扫描**

`src/inspection.rs`：

```rust
/// 扫描被取消（新的扫描开始或用户点了取消）。
#[derive(Debug, PartialEq, Eq)]
pub struct ScanCancelled;
```

新函数：函数体 = **剪切**现 `scan_directory_summary_with_progress` 的整个函数体粘入（约 inspection.rs:100-229，不要从零重写扫描逻辑），返回语句从 `DirectorySummary { counts, items }` 改为 `Ok(DirectorySummary { counts, items })`，再做下方「调用侧改动」两处修改：

```rust
pub fn scan_directory_summary_cancellable<F, C>(
    left_dir: &Path,
    right_dir: &Path,
    progress: F,
    should_cancel: C,
) -> Result<DirectorySummary, ScanCancelled>
where
    F: Fn(ScanProgress) + Sync,
    C: Fn() -> bool + Sync,
{
    // 此处为搬移过来的原函数体
}
```

旧名字保留为薄包装：

```rust
pub fn scan_directory_summary_with_progress<F>(
    left_dir: &Path,
    right_dir: &Path,
    progress: F,
) -> DirectorySummary
where
    F: Fn(ScanProgress) + Sync,
{
    scan_directory_summary_cancellable(left_dir, right_dir, progress, || false)
        .expect("scan without cancel hook cannot be cancelled")
}
```

`compare_pairs_parallel` 改造：增加 `should_cancel` 参数；worker 循环每轮先检查；结果改为 `Option` 容忍空槽：

```rust
fn compare_pairs_parallel<F, C>(
    pairs: &[MatchedPair],
    on_done: &F,
    should_cancel: &C,
) -> Vec<Option<DiffSummary>>
where
    F: Fn(usize) + Sync,
    C: Fn() -> bool + Sync,
{
    let total = pairs.len();
    if total == 0 {
        return Vec::new();
    }

    let worker_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(4)
        .min(total);
    let next_index = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<DiffSummary>>> = (0..total).map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| {
                loop {
                    if should_cancel() {
                        break;
                    }
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    if index >= total {
                        break;
                    }
                    let pair = &pairs[index];
                    let summary =
                        compare_pair_summary(&pair.left.absolute_path, &pair.right.absolute_path);
                    *results[index].lock().expect("result slot lock poisoned") = Some(summary);
                    let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    on_done(done);
                }
            });
        }
    });

    results
        .into_iter()
        .map(|slot| slot.into_inner().expect("result slot lock poisoned"))
        .collect()
}
```

调用侧（搬移后的函数体内）相应改为：

```rust
let summaries = compare_pairs_parallel(&pairs, &on_done_closure, &should_cancel);
if should_cancel() {
    return Err(ScanCancelled);
}
for (pair, diff_summary) in pairs.into_iter().zip(summaries) {
    let diff_summary = diff_summary.expect("non-cancelled scan compares every pair");
    // ……原循环体不变
}
// 函数末尾：Ok(DirectorySummary { counts, items })
```

- [ ] **Step 3: 运行 Rust 测试**

Run: `cargo test`
Expected: 新增 2 个用例 PASS，既有用例（含 `scan_directory_summary_with_progress_reports_each_compared_pair`）全部 PASS

- [ ] **Step 4: desktop_api 代号机制 + 命令注册**

`src/desktop_api.rs`（`use` 区追加 `use std::sync::atomic::{AtomicU64, Ordering};` 与 `scan_directory_summary_cancellable`，移除 `scan_directory_summary_with_progress` 的引入）：

```rust
/// 扫描代号：scan_directory 开始与 cancel_scan 都会推进它；
/// worker 发现代号过期即停。新的扫描天然取消旧扫描。
static SCAN_GENERATION: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub async fn scan_directory(
    left_dir: String,
    right_dir: String,
    on_progress: Channel<ScanProgress>,
) -> Result<DirectorySummary, String> {
    let my_gen = SCAN_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    run_blocking(move || {
        scan_directory_summary_cancellable(
            Path::new(&left_dir),
            Path::new(&right_dir),
            |p| {
                if should_emit_progress(p) {
                    let _ = on_progress.send(p);
                }
            },
            || SCAN_GENERATION.load(Ordering::SeqCst) != my_gen,
        )
    })
    .await?
    .map_err(|_| "cancelled".to_string())
}

#[tauri::command]
pub fn cancel_scan() {
    SCAN_GENERATION.fetch_add(1, Ordering::SeqCst);
}
```

注意 `run_blocking` 现在返回 `Result<Result<DirectorySummary, ScanCancelled>, String>`，所以是 `.await?` 后再 `map_err`。`src/main.rs` 的 `generate_handler!` 列表追加 `desktop_api::cancel_scan`。

desktop_api tests 模块追加：

```rust
#[test]
fn cancel_scan_bumps_generation_and_running_scan_reports_cancelled() {
    use super::SCAN_GENERATION;
    use std::sync::atomic::Ordering;
    let before = SCAN_GENERATION.load(Ordering::SeqCst);
    super::cancel_scan();
    assert_eq!(SCAN_GENERATION.load(Ordering::SeqCst), before + 1);
}
```

Run: `cargo test` → Expected: PASS

- [ ] **Step 5: 前端接线（测试先行）**

useWorkbench.test.tsx 追加：

```tsx
it('cancelScan invokes api and a cancelled scan resets silently to welcome', async () => {
  let rejectScan!: (e: Error) => void;
  const api = makeApi({
    cancelScan: vi.fn().mockResolvedValue(undefined),
    scanDirectory: vi.fn().mockImplementation(() => new Promise((_res, rej) => { rejectScan = rej; })),
  });
  const { result } = renderHook(() => useWorkbench(api));
  act(() => { result.current.setMode('directory'); });
  act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
  let pending!: Promise<void>;
  act(() => { pending = result.current.runCompare(); });
  await act(async () => {
    await result.current.cancelScan();
    rejectScan(new Error('cancelled'));
    await pending;
  });
  expect(api.cancelScan).toHaveBeenCalled();
  expect(result.current.error).toBeNull();
  expect(result.current.view).toBe('welcome');
  expect(result.current.isLoading).toBe(false);
});
```

Run: `cd frontend; npx vitest run src/features/workbench/useWorkbench.test.tsx` → FAIL

实现：

`frontend/src/lib/api.ts` — Task 5 加的可选成员 `cancelScan?` 改为**必选** `cancelScan(): Promise<void>;`，`workbenchApi` 实现追加：

```ts
async cancelScan(): Promise<void> {
  await invoke('cancel_scan');
},
```

（`makeApi` 工厂补 `cancelScan: vi.fn().mockResolvedValue(undefined)` 默认实现。`useWorkbench` 的 `cancelScan` 桩在 Task 5 已落地，调用处 `api.cancelScan?.()` 对必选成员依然合法，无需改动。）

`runAuto` 的 catch 改为：

```ts
} catch (err) {
  if (scanSeqRef.current === runId) {
    const msg = formatError(err);
    if (msg.includes('cancelled')) {
      setView('welcome');
      setDirectorySummary(null);
      setSelectedItemId(null);
    } else {
      setError(msg);
    }
  }
}
```

return 对象追加 `cancelScan`。

- [ ] **Step 6: 全量回归**

Run: `cd frontend; npx vitest run; npm run build`，然后 `cargo test`
Expected: 全部 PASS

- [ ] **Step 7: 手动验证取消**

Run: `cargo tauri dev` → 选 600 PNG 目录开扫 → 点「取消」→ 进度立停（< 300ms 体感）、回欢迎态、无错误横幅；再点 MRU 重扫成功。

- [ ] **Step 8: Commit**

```powershell
git add src/inspection.rs src/desktop_api.rs src/main.rs frontend/src/lib/api.ts frontend/src/features/workbench
git commit -m "feat(scan): generation-based cancellation, cancel_scan command, silent frontend reset"
```

---

## Task 9: 「浏览」卡顿诊断 → 修复

**Files:**
- Modify: `frontend/src/App.tsx`（打点 → 视结果切换实现）
- Possibly create: `src/desktop_api.rs` 的 `pick_folder` 命令、`Cargo.toml` 加 `rfd`
- Modify: `frontend/src/lib/api.ts`（可选 `pickFolder`）

- [ ] **Step 1: 打点诊断**

`App.tsx` 的 `pickPath` 临时加计时：

```ts
async function pickPath(directory: boolean): Promise<string> {
  const t0 = performance.now();
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: 'PNG', extensions: ['png'] }],
  });
  console.log(`[pick] dialog round-trip ${Math.round(performance.now() - t0)}ms`);
  return typeof selected === 'string' ? selected : '';
}
```

Run: `cargo tauri dev` → 在含 600 PNG 的真实目录上点「浏览」数次，记录：
- A：点击 → 对话框出现的延迟（目测 + console）
- B：对话框打开期间拖动主窗口是否冻结

- [ ] **Step 2: 按诊断分支处理**

**分支 A（对话框出现慢 / 主窗口冻结，疑似主线程阻塞）→ 实现 Rust 侧 `pick_folder`：**

`Cargo.toml` `[dependencies]` 追加：

```toml
rfd = "0.15"
```

`src/desktop_api.rs` 追加：

```rust
/// 在阻塞线程池上跑文件夹选择器，避免占住主线程导致 webview 卡顿。
/// rfd 在调用线程上自行初始化 COM（STA）。
#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    run_blocking(|| {
        rfd::FileDialog::new()
            .pick_folder()
            .map(|p| p.display().to_string())
    })
    .await
}
```

`src/main.rs` 注册 `desktop_api::pick_folder`。

`frontend/src/lib/api.ts` 接口追加可选方法 `pickFolder?(): Promise<string | null>;`，实现追加：

```ts
async pickFolder(): Promise<string | null> {
  return invoke<string | null>('pick_folder');
},
```

`App.tsx` 的 `pickPath` 目录分支改走后端命令：

```ts
import { workbenchApi } from './lib/api';

async function pickPath(directory: boolean): Promise<string> {
  if (directory) {
    const selected = await workbenchApi.pickFolder?.();
    return selected ?? '';
  }
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'PNG', extensions: ['png'] }],
  });
  return typeof selected === 'string' ? selected : '';
}
```

再次实测对比计时，确认达标后**移除 console 打点**。

**分支 B（对话框秒开、不冻结，慢的是 OS 枚举缩略图）→ 不改代码：**
移除打点，在 commit message 里记录实测数据，结论写「OS 行为，由 MRU/拖放/粘贴绕路吸收（已于 Task 6 落地）」。

- [ ] **Step 3: 回归**

Run: `cd frontend; npm run build; npx vitest run`，`cargo test`
Expected: 全部 PASS

- [ ] **Step 4: Commit（按分支二选一的信息）**

```powershell
git add -A
git commit -m "perf(pick): folder dialog off the main thread via rfd (measured: <填入实测前后毫秒数>)"
# 或
git commit -m "chore(pick): instrumented dialog latency — OS-side enumeration, mitigated by MRU (measured: <数据>)"
```

---

## Task 10: 退役旧组件 + 样式清理

**Files:**
- Delete: `frontend/src/components/MirrorTree.tsx` + `MirrorTree.test.tsx`
- Delete: `frontend/src/components/SoloTree.tsx` + `SoloTree.test.tsx`
- Delete: `frontend/src/components/DirectoryList.tsx` + `DirectoryList.test.tsx`
- Delete: `frontend/src/components/Slot.tsx` + `Slot.test.tsx`、`SlotBar.tsx` + `SlotBar.test.tsx`
- Maybe delete: `frontend/src/components/StatusBadge.tsx` + 测试（先查引用）
- Modify: `frontend/src/styles/app.css`（删除死样式）、`frontend/src/styles/tokens.css`（删 legacy 别名）

- [ ] **Step 1: 确认无引用后删除组件**

Run: `cd frontend; npx tsc --noEmit -p tsconfig.json` 先确认基线干净。逐个 grep：

```powershell
cd frontend
findstr /s /m "MirrorTree SoloTree DirectoryList SlotBar StatusBadge" src\*.tsx src\*.ts
```

确认仅自身与测试文件引用后，删除上列文件（`StatusBadge` 若仍被引用则保留并在 commit message 注明）。

- [ ] **Step 2: 删除死样式与 legacy 变量**

- `app.css`：删除 `.slot`、`.slotbar`、`.dirlist`、`.mirror-tree`、`.mirror-pane`、`.solo-status`、`.diff-report`、`.diff-tree`、`.controlbar`、`.scan-progress`、`.welcome`（旧版）相关块（保留 `.mirror-grid`、`.raw-json`、`.image-split`、`.image-pane`、`.kv`?——`.kv` 仅旧树使用，一并删除；删除前 grep 确认）。
- `tokens.css`：删除 Task 1 注释标记的 legacy aliases 区块；随后 grep `--bg-topbar|--bg-slotbar|--bg-controlbar|--bg-overlay|--mod-text|--mod-arrow|--add-emph|--rem-emph|--bar-edge|--bg-tint` 确认无残留引用（`--bg-overlay`/`--mod-text` 若仍被新样式引用，改写为新名字再删）。

- [ ] **Step 3: 全量回归**

Run: `cd frontend; npx vitest run; npm run build`，`cargo test`
Expected: 全部 PASS；build 无 unused 报错

- [ ] **Step 4: 手动冒烟**

Run: `cargo tauri dev` → 欢迎态 / 目录巡检 / 单文件 / solo / 错误卡 / 图片 / JSON 全过一遍。

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "chore(cleanup): retire MirrorTree/SoloTree/DirectoryList/Slot(Bar), drop dead styles and legacy tokens"
```

---

## Task 11: 验收指标实测（设计稿 §4 ④）

**Files:** 无代码变更（除非实测不达标，则按对应任务回修）

- [ ] **Step 1: 逐项实测并记录**

用真实 600 PNG 目录，`cargo tauri dev`（性能项建议再用 `cargo tauri build` 的 release 包复测一遍）：

| 指标 | 目标 | 测法 |
|---|---|---|
| 点「浏览」→ 对话框可交互 | < 500ms，主窗不冻结 | Task 9 打点数据 / 秒表目测 |
| 列表搜索每击键渲染 | < 16ms | DevTools Performance 录制连续输入，看主线程帧 |
| 打开一对典型差异文件 → 树出现 | < 100ms | DevTools Performance：点击行 → 首次 paint |
| 切换仅看不同 / 高亮 | < 50ms | 同上 |
| 取消扫描生效 | < 300ms | 点取消 → 进度停更的体感/录屏 |
| 扫描全程 UI 可交互 | 达成 | 扫描中拖动窗口、滚动列表 |

- [ ] **Step 2: 不达标项回修**

回到对应任务的实现处修（如列表击键超标 → Task 5 预留的手写窗口化：固定行高 28px、只渲染可视区 ± 10 行）。修完重测。

- [ ] **Step 3: 记录结果并收尾提交**

把实测数据追加到设计稿 §4 表格后（新增「实测」列），commit：

```powershell
git add docs/superpowers/specs/2026-06-12-ui-comfort-redesign-design.md
git commit -m "docs(spec): record acceptance measurements for perf targets"
```

---

## 完成后

全部任务完成且验收达标后，使用 superpowers:finishing-a-development-branch 技能决定合并/PR/清理。
