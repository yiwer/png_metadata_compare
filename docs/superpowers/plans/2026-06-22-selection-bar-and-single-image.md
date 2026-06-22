# 常驻选择条 + 单图修复 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 给左右来源加一条常驻"选择条"（已选样式 + ✕清除 + 点击重选，单文件/目录一致），并修复"只选一侧时点『图片』看不到图"的塌陷 bug。

**Architecture:** 前端 React + TypeScript（Tauri v2 webview）。新增 `SelectionBar` 组件统一接管左右来源选择，去重原先分散在 `WelcomePane`/`Sidebar`/`DetailHeader` 的选择 UI；抽出 `useImageTransform` 共享缩放/平移逻辑，把单图与双图视图收进 `components/ImageViews.tsx`，并修单图布局。

**Tech Stack:** React 18、TypeScript、Vitest + @testing-library/react、CSS（`frontend/src/styles/app.css`）。

## Global Constraints

- 不新增第三方依赖。
- UI 文案为中文，沿用现有字符串（如 `浏览…`、`粘贴路径后回车`）。
- 保持深色石墨风格，样式优先复用 `app.css` 既有 CSS 变量（`--bg-elevated`、`--border-default`、`--border-subtle`、`--bg-page`、`--text-secondary`、`--sp-2/3`、`--fs-xs/sm`）。
- 组件文件放 `frontend/src/components/`，hook/纯逻辑放 `frontend/src/lib/`，每个测试与被测文件同目录同名 `.test.tsx`。
- 测试命令：在 `frontend/` 下运行 `npm run test`（vitest）；类型检查 `npm run build`（含 `tsc --noEmit`）。
- 选择条**不要**加 `data-tauri-drag-region`，避免与窗口拖动区冲突。
- 与本计划无关的、工作区里已存在的窗口拖动修复改动（`capabilities/`、`App.tsx`/`app.css` 的 drag-region 部分）保持原样，勿一并提交。

---

### Task 1: `useImageTransform` 共享 hook + 重构 `ImageSplit`

把 `ImageSplit` 内联的缩放/平移逻辑抽成可测 hook，行为不变。

**Files:**
- Create: `frontend/src/lib/useImageTransform.ts`
- Create: `frontend/src/lib/useImageTransform.test.ts`
- Modify: `frontend/src/App.tsx`（`ImageSplit` 改用 hook；本任务暂不动 `SingleImage`）

**Interfaces:**
- Produces: `useImageTransform(): ImageTransform`，其中
  ```ts
  interface ImageTransform {
    zoom: number;
    transform: string;          // `translate(${x}px, ${y}px) scale(${zoom})`
    dragging: boolean;
    onWheel(e: React.WheelEvent): void;
    onMouseDown(e: React.MouseEvent): void;
    onMouseMove(e: React.MouseEvent): void;
    endDrag(): void;
    zoomIn(): void;
    zoomOut(): void;
    reset(): void;
  }
  ```

- [ ] **Step 1: 写失败测试**

`frontend/src/lib/useImageTransform.test.ts`:
```ts
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { useImageTransform } from './useImageTransform';

describe('useImageTransform', () => {
  it('starts at zoom 1 with an identity translate/scale', () => {
    const { result } = renderHook(() => useImageTransform());
    expect(result.current.zoom).toBe(1);
    expect(result.current.transform).toBe('translate(0px, 0px) scale(1)');
  });

  it('zoomIn multiplies by 1.25, reset returns to 1', () => {
    const { result } = renderHook(() => useImageTransform());
    act(() => { result.current.zoomIn(); });
    expect(result.current.zoom).toBeCloseTo(1.25);
    act(() => { result.current.reset(); });
    expect(result.current.zoom).toBe(1);
  });

  it('drag updates the translate offset', () => {
    const { result } = renderHook(() => useImageTransform());
    act(() => { result.current.onMouseDown({ button: 0, clientX: 10, clientY: 10 } as unknown as React.MouseEvent); });
    act(() => { result.current.onMouseMove({ clientX: 30, clientY: 25 } as unknown as React.MouseEvent); });
    expect(result.current.transform).toBe('translate(20px, 15px) scale(1)');
    act(() => { result.current.endDrag(); });
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend && npx vitest run src/lib/useImageTransform.test.ts`
Expected: FAIL（`Cannot find module './useImageTransform'`）

- [ ] **Step 3: 实现 hook**

`frontend/src/lib/useImageTransform.ts`:
```ts
import { useRef, useState } from 'react';
import type * as React from 'react';

export interface ImageTransform {
  zoom: number;
  transform: string;
  dragging: boolean;
  onWheel(e: React.WheelEvent): void;
  onMouseDown(e: React.MouseEvent): void;
  onMouseMove(e: React.MouseEvent): void;
  endDrag(): void;
  zoomIn(): void;
  zoomOut(): void;
  reset(): void;
}

export function useImageTransform(): ImageTransform {
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const dragRef = useRef<{ startX: number; startY: number; baseX: number; baseY: number } | null>(null);

  const onWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
    setZoom((z) => Math.min(20, Math.max(0.1, z * factor)));
  };
  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY, baseX: offset.x, baseY: offset.y };
  };
  const onMouseMove = (e: React.MouseEvent) => {
    if (!dragRef.current) return;
    setOffset({
      x: dragRef.current.baseX + (e.clientX - dragRef.current.startX),
      y: dragRef.current.baseY + (e.clientY - dragRef.current.startY),
    });
  };
  const endDrag = () => { dragRef.current = null; };
  const zoomIn = () => setZoom((z) => Math.min(20, z * 1.25));
  const zoomOut = () => setZoom((z) => Math.max(0.1, z / 1.25));
  const reset = () => { setZoom(1); setOffset({ x: 0, y: 0 }); };

  const transform = `translate(${offset.x}px, ${offset.y}px) scale(${zoom})`;
  const dragging = dragRef.current !== null;
  return { zoom, transform, dragging, onWheel, onMouseDown, onMouseMove, endDrag, zoomIn, zoomOut, reset };
}
```

- [ ] **Step 4: 运行确认通过**

Run: `cd frontend && npx vitest run src/lib/useImageTransform.test.ts`
Expected: PASS（3 passed）

- [ ] **Step 5: 重构 `ImageSplit` 使用 hook（行为不变）**

在 `frontend/src/App.tsx` 中，把 `ImageSplit`（当前约 327-377 行）整体替换为：
```tsx
function ImageSplit({ leftPath, rightPath, leftName, rightName }: { leftPath: string; rightPath: string; leftName: string; rightName: string; }) {
  const t = useImageTransform();
  return (
    <div className="image-split">
      <div className="image-split__toolbar">
        <button type="button" className="controlbar__btn" onClick={t.zoomOut}>−</button>
        <span className="controlbar__summary" style={{ minWidth: 56, textAlign: 'center' }}>{Math.round(t.zoom * 100)}%</span>
        <button type="button" className="controlbar__btn" onClick={t.zoomIn}>＋</button>
        <button type="button" className="controlbar__btn" onClick={t.reset}>重置</button>
        <span className="controlbar__spacer" />
        <span className="controlbar__summary">滚轮缩放 · 拖拽平移（左右同步）</span>
      </div>
      <div
        className={`image-split__panes${t.dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={t.onWheel}
        onMouseDown={t.onMouseDown}
        onMouseMove={t.onMouseMove}
        onMouseUp={t.endDrag}
        onMouseLeave={t.endDrag}
      >
        <ImagePane key={leftPath} path={leftPath} name={leftName} transform={t.transform} />
        <ImagePane key={rightPath} path={rightPath} name={rightName} transform={t.transform} />
      </div>
    </div>
  );
}
```
并在 `App.tsx` 顶部 import 区加入：
```tsx
import { useImageTransform } from './lib/useImageTransform';
```
同时删除 `App.tsx` 中 `useState`/`useRef` 若因此变为未使用——注意 `App` 组件别处仍用到它们，**保留**现有 import（`ImageSplit` 不再用 `useState/useRef`，但文件其他部分仍需要）。

- [ ] **Step 6: 类型检查 + 全量测试**

Run: `cd frontend && npm run build && npx vitest run`
Expected: tsc 退出 0；所有测试 PASS

- [ ] **Step 7: Commit**

```bash
git add frontend/src/lib/useImageTransform.ts frontend/src/lib/useImageTransform.test.ts frontend/src/App.tsx
git commit -m "refactor(image): extract useImageTransform hook; ImageSplit consumes it"
```

---

### Task 2: 修复单图 — `components/ImageViews.tsx`

把 `ImagePane` / `ImageSplit` 移入独立文件，新增 `SoloImage`（带工具条 + 单列），修单图塌陷。

**Files:**
- Create: `frontend/src/components/ImageViews.tsx`
- Create: `frontend/src/components/ImageViews.test.tsx`
- Modify: `frontend/src/App.tsx`（删除本地 `ImagePane`/`ImageSplit`/`SingleImage`，改 import；`solo` 图片视图用 `SoloImage`）
- Modify: `frontend/src/styles/app.css`（新增 `.image-split__panes--solo`）

**Interfaces:**
- Consumes: `useImageTransform`（Task 1）
- Produces:
  - `ImagePane({ path, name, transform }: { path: string; name: string; transform: string })`
  - `ImageSplit({ leftPath, rightPath, leftName, rightName })`
  - `SoloImage({ path, name }: { path: string; name: string })`

- [ ] **Step 1: 写失败测试**

`frontend/src/components/ImageViews.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SoloImage, ImageSplit } from './ImageViews';

vi.mock('@tauri-apps/api/core', () => ({ convertFileSrc: (p: string) => `asset://${p}` }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openPath: vi.fn() }));

describe('image views', () => {
  it('SoloImage renders one pane, a zoom toolbar, and the single-column class', () => {
    const { container } = render(<SoloImage path="/x.png" name="x.png" />);
    expect(container.querySelectorAll('.image-pane')).toHaveLength(1);
    expect(container.querySelector('.image-split__panes--solo')).not.toBeNull();
    expect(screen.getByRole('button', { name: '重置' })).toBeInTheDocument();
  });

  it('SoloImage points the <img> at the converted asset url', () => {
    const { container } = render(<SoloImage path="/pics/a.png" name="a.png" />);
    expect(container.querySelector('img.image-pane__img')).toHaveAttribute('src', 'asset:///pics/a.png');
  });

  it('ImageSplit renders two panes', () => {
    const { container } = render(
      <ImageSplit leftPath="/l.png" rightPath="/r.png" leftName="l" rightName="r" />);
    expect(container.querySelectorAll('.image-pane')).toHaveLength(2);
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend && npx vitest run src/components/ImageViews.test.tsx`
Expected: FAIL（`Cannot find module './ImageViews'`）

- [ ] **Step 3: 实现 `ImageViews.tsx`**

`frontend/src/components/ImageViews.tsx`:
```tsx
import { useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { useImageTransform } from '../lib/useImageTransform';

export function ImagePane({ path, name, transform }: { path: string; name: string; transform: string }) {
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

function ZoomToolbar({ t, summary }: { t: ReturnType<typeof useImageTransform>; summary: string }) {
  return (
    <div className="image-split__toolbar">
      <button type="button" className="controlbar__btn" onClick={t.zoomOut}>−</button>
      <span className="controlbar__summary" style={{ minWidth: 56, textAlign: 'center' }}>{Math.round(t.zoom * 100)}%</span>
      <button type="button" className="controlbar__btn" onClick={t.zoomIn}>＋</button>
      <button type="button" className="controlbar__btn" onClick={t.reset}>重置</button>
      <span className="controlbar__spacer" />
      <span className="controlbar__summary">{summary}</span>
    </div>
  );
}

export function ImageSplit({ leftPath, rightPath, leftName, rightName }: { leftPath: string; rightPath: string; leftName: string; rightName: string; }) {
  const t = useImageTransform();
  return (
    <div className="image-split">
      <ZoomToolbar t={t} summary="滚轮缩放 · 拖拽平移（左右同步）" />
      <div
        className={`image-split__panes${t.dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={t.onWheel} onMouseDown={t.onMouseDown} onMouseMove={t.onMouseMove}
        onMouseUp={t.endDrag} onMouseLeave={t.endDrag}
      >
        <ImagePane key={leftPath} path={leftPath} name={leftName} transform={t.transform} />
        <ImagePane key={rightPath} path={rightPath} name={rightName} transform={t.transform} />
      </div>
    </div>
  );
}

export function SoloImage({ path, name }: { path: string; name: string }) {
  const t = useImageTransform();
  return (
    <div className="image-split">
      <ZoomToolbar t={t} summary="滚轮缩放 · 拖拽平移" />
      <div
        className={`image-split__panes image-split__panes--solo${t.dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={t.onWheel} onMouseDown={t.onMouseDown} onMouseMove={t.onMouseMove}
        onMouseUp={t.endDrag} onMouseLeave={t.endDrag}
      >
        <ImagePane key={path} path={path} name={name} transform={t.transform} />
      </div>
    </div>
  );
}
```

- [ ] **Step 4: 加 CSS 单列修饰**

在 `frontend/src/styles/app.css` 的 `.image-split__panes--dragging` 规则之后加入：
```css
.image-split__panes--solo { grid-template-columns: 1fr; }
```

- [ ] **Step 5: 运行确认测试通过**

Run: `cd frontend && npx vitest run src/components/ImageViews.test.tsx`
Expected: PASS（3 passed）

- [ ] **Step 6: 在 `App.tsx` 切换到新组件**

1. 顶部 import 区加入：
   ```tsx
   import { ImageSplit, SoloImage } from './components/ImageViews';
   ```
2. 删除 `App.tsx` 中本地的 `ImagePane`、`ImageSplit`、`SingleImage` 三个函数定义（Task 1 改过的 `ImageSplit` 一并删除——以新文件为准）。
3. 删除随之不再使用的 import：`convertFileSrc`（来自 `@tauri-apps/api/core`）、`useImageTransform`（Task 1 加的，那行删掉）。`openPath` 若 `App.tsx` 别处（`ErrorCard`）仍用则保留——`ErrorCard` 用到 `openPath`，**保留** `openPath` import。
4. 把 solo 图片视图渲染（当前约 188 行）从 `<SingleImage .../>` 改为：
   ```tsx
   <SoloImage path={wb.soloResult.file_path} name={wb.soloResult.file_name} />
   ```

- [ ] **Step 7: 类型检查 + 全量测试**

Run: `cd frontend && npm run build && npx vitest run`
Expected: tsc 退出 0；全部 PASS

- [ ] **Step 8: Commit**

```bash
git add frontend/src/components/ImageViews.tsx frontend/src/components/ImageViews.test.tsx frontend/src/App.tsx frontend/src/styles/app.css
git commit -m "fix(image): single image fills height via dedicated SoloImage view"
```

---

### Task 3: `useWorkbench` — `clearSide` + `resetToWelcome`

新增清除某一侧的能力，并让"回欢迎页"分支清空残留结果。

> **与 spec 的差异（有意为之）**：spec 写"把副作用改为始终 `runAuto`"。本计划改为**保留** `App` 现有副作用守卫（`if (left||right) runAuto()`），由 `clearSide` 在"另一侧也为空"时直接调 `resetToWelcome()`。可观察行为一致，且避免挂载时空输入触发 `runAuto` 引入 `act()` 警告。

**Files:**
- Modify: `frontend/src/features/workbench/useWorkbench.ts`
- Modify: `frontend/src/features/workbench/useWorkbench.test.tsx`

**Interfaces:**
- Produces: `clearSide(side: Side): void`（在 hook 返回对象中导出）

- [ ] **Step 1: 写失败测试**

在 `frontend/src/features/workbench/useWorkbench.test.tsx` 末尾追加：
```ts
describe('clearSide', () => {
  it('clears that side input', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setLeftInput('/a.png'); });
    act(() => { result.current.clearSide('left'); });
    expect(result.current.leftInput).toBe('');
  });

  it('clearing the last remaining side resets to welcome and clears results', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('solo');
    act(() => { result.current.clearSide('left'); });
    expect(result.current.leftInput).toBe('');
    expect(result.current.view).toBe('welcome');
    expect(result.current.soloResult).toBeNull();
  });

  it('single mirror: clearing one side then runAuto shows the remaining image (solo)', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('mirror');
    act(() => { result.current.clearSide('right'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('solo');
    expect(result.current.soloSide).toBe('left');
  });

  it('directory: clearing one folder then runAuto returns to welcome with summary cleared', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/L'); result.current.setRightInput('/R'); });
    await act(async () => { await result.current.runCompare(); });
    expect(result.current.directorySummary).not.toBeNull();
    act(() => { result.current.clearSide('left'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('welcome');
    expect(result.current.directorySummary).toBeNull();
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend && npx vitest run src/features/workbench/useWorkbench.test.tsx -t clearSide`
Expected: FAIL（`result.current.clearSide is not a function`）

- [ ] **Step 3: 实现 `resetToWelcome` + `clearSide`**

在 `useWorkbench.ts` 中：

3a. 在 `flashToast` 之后新增（`Side` 类型已在文件顶部 import）：
```ts
const resetToWelcome = useCallback(() => {
  setView('welcome');
  setPairResult(null);
  setSoloResult(null);
  setSoloSide(null);
  setDirectorySummary(null);
  setSelectedItemId(null);
  setErrorItem(null);
}, []);

const clearSide = useCallback((side: Side) => {
  const other = side === 'left' ? activeInputs.right : activeInputs.left;
  if (side === 'left') setLeftInput(''); else setRightInput('');
  setError(null);
  if (!other) resetToWelcome();
}, [activeInputs, resetToWelcome]);
```

3b. 在 `runAuto` 内，把"单文件两侧皆空"分支
```ts
        } else {
          setView('welcome');
        }
```
改为
```ts
        } else {
          resetToWelcome();
        }
```
并把目录模式的"未两侧齐全"分支
```ts
      } else {
        setView('welcome');
      }
```
改为
```ts
      } else {
        resetToWelcome();
      }
```
> 注意：**不要**动目录"扫描结果为空"那一处 `else setView('welcome')`（line ~285，紧跟 `if (visible[0]) await selectItem(...)`）——该处需保留 `directorySummary`（现有测试 "scan with no items falls back to welcome view" 依赖它非空）。

3c. 在 hook 的 `return { ... }` 中加入 `clearSide,`（放在 `selectItem` 附近）。

- [ ] **Step 4: 运行确认通过**

Run: `cd frontend && npx vitest run src/features/workbench/useWorkbench.test.tsx`
Expected: PASS（含原有用例，全部通过）

- [ ] **Step 5: Commit**

```bash
git add frontend/src/features/workbench/useWorkbench.ts frontend/src/features/workbench/useWorkbench.test.tsx
git commit -m "feat(workbench): clearSide + resetToWelcome clears stale results"
```

---

### Task 4: `SelectionBar` 组件 + 样式

**Files:**
- Create: `frontend/src/components/SelectionBar.tsx`
- Create: `frontend/src/components/SelectionBar.test.tsx`
- Modify: `frontend/src/styles/app.css`（新增 `.selbar*` 样式）

**Interfaces:**
- Consumes: `loadRecent`（`../lib/recentDirs`）、类型 `Side`/`WorkbenchMode`（`../lib/types`）
- Produces: `SelectionBar(props)`，props：
  ```ts
  mode: WorkbenchMode; leftInput: string; rightInput: string;
  onPickLeft(): void; onPickRight(): void;
  onPastePath(side: Side, path: string): void;
  onApplyPair(left: string, right: string): void;
  onClear(side: Side): void;
  onDrop(side: Side, path: string): void;
  ```
- DOM 约定（供测试与 App 集成）：根 `.selbar`；每槽 `.selbar__slot[data-side="left|right"]`，主按钮 `.selbar__slot-main`，清除按钮 `aria-label="清除左侧"/"清除右侧"`，下拉 `.selbar__menu`（含 `浏览…` 按钮、`粘贴路径后回车` 输入、最近列表）。

- [ ] **Step 1: 写失败测试**

`frontend/src/components/SelectionBar.test.tsx`:
```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SelectionBar } from './SelectionBar';
import { touchRecent } from '../lib/recentDirs';

function renderBar(over: Partial<Parameters<typeof SelectionBar>[0]> = {}) {
  const props = {
    mode: 'single' as const, leftInput: '', rightInput: '',
    onPickLeft: vi.fn(), onPickRight: vi.fn(), onPastePath: vi.fn(),
    onApplyPair: vi.fn(), onClear: vi.fn(), onDrop: vi.fn(),
    ...over,
  };
  render(<SelectionBar {...props} />);
  return props;
}

describe('SelectionBar', () => {
  beforeEach(() => localStorage.clear());

  it('empty slots show a placeholder and no clear button', () => {
    renderBar();
    expect(screen.getAllByText(/点击选择PNG 文件/)).toHaveLength(2);
    expect(screen.queryByLabelText('清除左侧')).toBeNull();
  });

  it('filled slot shows the basename and a working clear button', () => {
    const p = renderBar({ leftInput: 'C:/pics/photo_a.png' });
    expect(screen.getByText('photo_a.png')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('清除左侧'));
    expect(p.onClear).toHaveBeenCalledWith('left');
  });

  it('clicking a slot opens its dropdown; 浏览… calls the pick handler', () => {
    const p = renderBar();
    fireEvent.click(screen.getAllByText(/点击选择/)[0]);
    fireEvent.click(screen.getByRole('button', { name: '浏览…' }));
    expect(p.onPickLeft).toHaveBeenCalled();
  });

  it('pasting a path + Enter calls onPastePath for that side (directory mode)', () => {
    const p = renderBar({ mode: 'directory' });
    fireEvent.click(screen.getAllByText(/点击选择/)[1]); // 右槽
    const input = screen.getByPlaceholderText('粘贴路径后回车');
    fireEvent.change(input, { target: { value: 'D:/folder' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(p.onPastePath).toHaveBeenCalledWith('right', 'D:/folder');
  });

  it('recent list (mode-appropriate) applies a pair', () => {
    touchRecent('file', 'C:/a.png', 'C:/b.png');
    const p = renderBar({ mode: 'single' });
    fireEvent.click(screen.getAllByText(/点击选择/)[0]);
    fireEvent.click(screen.getByText(/a\.png ⇄ b\.png/));
    expect(p.onApplyPair).toHaveBeenCalledWith('C:/a.png', 'C:/b.png');
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend && npx vitest run src/components/SelectionBar.test.tsx`
Expected: FAIL（`Cannot find module './SelectionBar'`）

- [ ] **Step 3: 实现 `SelectionBar.tsx`**

`frontend/src/components/SelectionBar.tsx`:
```tsx
import { useEffect, useState } from 'react';
import { loadRecent } from '../lib/recentDirs';
import type { Side, WorkbenchMode } from '../lib/types';

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function SelectionBar({
  mode, leftInput, rightInput,
  onPickLeft, onPickRight, onPastePath, onApplyPair, onClear, onDrop,
}: {
  mode: WorkbenchMode;
  leftInput: string;
  rightInput: string;
  onPickLeft(): void;
  onPickRight(): void;
  onPastePath(side: Side, path: string): void;
  onApplyPair(left: string, right: string): void;
  onClear(side: Side): void;
  onDrop(side: Side, path: string): void;
}) {
  const [openSide, setOpenSide] = useState<Side | null>(null);

  useEffect(() => {
    if (!openSide) return;
    const close = (e: MouseEvent) => {
      const inside = (e.target as HTMLElement).closest('.selbar__menu, .selbar__slot-main');
      if (!inside) setOpenSide(null);
    };
    window.addEventListener('mousedown', close);
    return () => window.removeEventListener('mousedown', close);
  }, [openSide]);

  const kind = mode === 'directory' ? 'dir' : 'file';
  const noun = mode === 'directory' ? '文件夹' : 'PNG 文件';

  const renderSlot = (side: Side, value: string) => {
    const onPick = side === 'left' ? onPickLeft : onPickRight;
    const sideLabel = side === 'left' ? '左' : '右';
    const dropHandler = (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      const p = (file as unknown as { path?: string })?.path;
      if (p) { onDrop(side, p); setOpenSide(null); }
    };
    return (
      <div className={`selbar__slot${value ? ' selbar__slot--filled' : ' selbar__slot--empty'}`}
        data-side={side} onDragOver={(e) => e.preventDefault()} onDrop={dropHandler}>
        <span className="selbar__side">{sideLabel}</span>
        <button type="button" className="selbar__slot-main" title={value || '未选择'}
          onClick={() => setOpenSide(openSide === side ? null : side)}>
          {value ? basename(value) : `点击选择${noun} / 拖入…`}
        </button>
        {value && (
          <button type="button" className="selbar__clear" aria-label={`清除${sideLabel}侧`}
            onClick={() => { onClear(side); setOpenSide(null); }}>✕</button>
        )}
        {openSide === side && (
          <div className="selbar__menu">
            <button type="button" onClick={() => { onPick(); setOpenSide(null); }}>浏览…</button>
            <input type="text" placeholder="粘贴路径后回车"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const v = (e.target as HTMLInputElement).value.trim();
                  if (v) { onPastePath(side, v); setOpenSide(null); }
                } else if (e.key === 'Escape') setOpenSide(null);
              }} />
            {loadRecent(kind).map((p) => (
              <button key={`${p.left}|${p.right}`} type="button" title={`${p.left}\n${p.right}`}
                onClick={() => { onApplyPair(p.left, p.right); setOpenSide(null); }}>
                {basename(p.left)} ⇄ {basename(p.right)}
              </button>
            ))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="selbar">
      {renderSlot('left', leftInput)}
      {renderSlot('right', rightInput)}
    </div>
  );
}
```

- [ ] **Step 4: 运行确认通过**

Run: `cd frontend && npx vitest run src/components/SelectionBar.test.tsx`
Expected: PASS（5 passed）

- [ ] **Step 5: 加样式**

在 `frontend/src/styles/app.css` 末尾追加：
```css
/* === Selection bar === */
.selbar {
  display: flex;
  gap: var(--sp-3);
  padding: var(--sp-2) var(--sp-3);
  background: var(--bg-page);
  border-bottom: 1px solid var(--border-subtle);
}
.selbar__slot {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  height: 38px;
  padding: 0 11px;
  border: 1px solid var(--border-default);
  border-radius: 8px;
  background: var(--bg-elevated);
}
.selbar__slot--empty { border-style: dashed; background: transparent; }
.selbar__slot--filled { box-shadow: inset 0 0 0 1px rgba(91,157,217,.18); }
.selbar__side {
  font-size: var(--fs-xs);
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
  border-radius: 5px;
  padding: 1px 6px;
}
.selbar__slot-main {
  flex: 1;
  text-align: left;
  background: none;
  border: none;
  color: inherit;
  font-size: var(--fs-sm);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}
.selbar__slot--empty .selbar__slot-main { color: var(--text-secondary); }
.selbar__clear {
  width: 19px; height: 19px; line-height: 18px; text-align: center;
  border: none; border-radius: 5px;
  background: var(--border-default); color: var(--text-secondary); cursor: pointer;
}
.selbar__menu {
  position: absolute;
  top: 44px; left: 0; z-index: 20;
  min-width: 240px;
  padding: 5px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  box-shadow: 0 10px 28px rgba(0,0,0,.5);
}
.selbar__menu button {
  display: block; width: 100%; text-align: left;
  background: none; border: none; color: inherit;
  padding: 7px 9px; border-radius: 6px; font-size: var(--fs-sm); cursor: pointer;
}
.selbar__menu button:hover { background: var(--bg-page); }
.selbar__menu input {
  width: 100%; margin: 4px 0;
  padding: 6px 9px;
  background: var(--bg-page);
  border: 1px solid var(--border-default);
  border-radius: 6px; color: inherit; font-size: var(--fs-sm);
}
```

- [ ] **Step 6: 类型检查 + 全量测试**

Run: `cd frontend && npm run build && npx vitest run`
Expected: tsc 退出 0；全部 PASS

- [ ] **Step 7: Commit**

```bash
git add frontend/src/components/SelectionBar.tsx frontend/src/components/SelectionBar.test.tsx frontend/src/styles/app.css
git commit -m "feat(selection): SelectionBar component with filled/clear/reselect states"
```

---

### Task 5: 把 `SelectionBar` 接入 `App`（保留旧入口，便于逐步切换）

**Files:**
- Modify: `frontend/src/App.tsx`
- Modify: `frontend/src/App.test.tsx`（新增一条选择条集成测试）

**Interfaces:**
- Consumes: `SelectionBar`（Task 4）、`wb.clearSide`（Task 3）

- [ ] **Step 1: 写失败测试**

在 `frontend/src/App.test.tsx` 顶部 import 增加 `within`：
```tsx
import { render, screen, fireEvent, waitFor, act, within } from '@testing-library/react';
```
在 `describe('App (three-column shell)')` 内追加：
```tsx
it('selection bar: pick a file via 浏览…, see solo view, then ✕ clears back to welcome', async () => {
  vi.mocked(open).mockResolvedValueOnce('/test/photo.png');
  vi.mocked(workbenchApi.inspectSingle).mockResolvedValueOnce({
    side: 'left', file_path: '/test/photo.png', file_name: 'photo.png',
    raw_json: '{}', metadata: {}, error: null,
  });
  render(<App />);
  const bar = document.querySelector('.selbar') as HTMLElement;
  // 打开左槽下拉 → 浏览…
  await act(async () => {
    fireEvent.click(bar.querySelector('[data-side="left"] .selbar__slot-main') as HTMLElement);
  });
  await act(async () => {
    fireEvent.click(within(bar).getByRole('button', { name: '浏览…' }));
  });
  await waitFor(() => expect(document.querySelector('.detail-head')).not.toBeNull());
  // ✕ 清除左侧 → 回欢迎页
  fireEvent.click(screen.getByLabelText('清除左侧'));
  await waitFor(() => expect(document.querySelector('.detail-head')).toBeNull());
});
```

- [ ] **Step 2: 运行确认失败**

Run: `cd frontend && npx vitest run src/App.test.tsx -t "selection bar"`
Expected: FAIL（找不到 `.selbar`）

- [ ] **Step 3: 渲染 `SelectionBar` 并接线**

在 `App.tsx`：

3a. import 区加入：
```tsx
import { SelectionBar } from './components/SelectionBar';
```

3b. 在 `return (` 的 `<header className="topbar"> ... </header>` 之后、`{wb.error && ...}` 之前插入：
```tsx
      <SelectionBar
        mode={wb.mode}
        leftInput={wb.leftInput}
        rightInput={wb.rightInput}
        onPickLeft={() => void handlePickLeft()}
        onPickRight={() => void handlePickRight()}
        onPastePath={(side, p) => wb.tryDropPath(side, p)}
        onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }}
        onClear={(side) => wb.clearSide(side)}
        onDrop={(side, p) => wb.tryDropPath(side, p)}
      />
```

- [ ] **Step 4: 运行确认通过**

Run: `cd frontend && npx vitest run src/App.test.tsx`
Expected: PASS（新用例 + 原有用例全部通过；此时欢迎页同时存在 WelcomePane 槽与选择条，属预期的过渡态）

- [ ] **Step 5: Commit**

```bash
git add frontend/src/App.tsx frontend/src/App.test.tsx
git commit -m "feat(app): mount SelectionBar above the shell body"
```

---

### Task 6: 去重 — 删除欢迎页槽 / 侧栏目录芯片 / 详情头芯片，统一由选择条负责

**Files:**
- Modify: `frontend/src/components/WelcomePane.tsx` + `WelcomePane.test.tsx`
- Modify: `frontend/src/components/Sidebar.tsx` + `Sidebar.test.tsx`
- Modify: `frontend/src/App.tsx`（去掉 `DetailHeader` 芯片；收敛传给 WelcomePane/Sidebar 的 props）
- Modify: `frontend/src/App.test.tsx`（改写依赖旧"浏览"槽的用例与 `setupDirectoryScan`）

**Interfaces:**
- After: `WelcomePane` props 收敛为 `{ mode, onApplyPair }`；`Sidebar` 去掉 `leftDir/rightDir/onPickLeft/onPickRight/onApplyPair/onPastePath`。

- [ ] **Step 1: 改 `WelcomePane`（删槽，保留最近 + 提示）**

`frontend/src/components/WelcomePane.tsx`：
1. props 改为：
   ```tsx
   export function WelcomePane({ mode, onApplyPair }: {
     mode: WorkbenchMode;
     onApplyPair(left: string, right: string): void;
   }) {
   ```
2. 删除 `dropHandler` 定义与 `welcome2__slots` 整块（`<div className="welcome2__slots"> ... </div>`）。
3. 删除现在未使用的 import：`Side`（`../lib/types` 行里去掉 `Side`，保留 `WorkbenchMode`）。`onDrop/onPickLeft/onPickRight` 相关全部移除。

- [ ] **Step 2: 改 `WelcomePane.test.tsx`（去掉已删 props）**

把四处 `render(<WelcomePane ... />)` 的 props 收敛为只剩 `mode` 与 `onApplyPair`。例如：
```tsx
render(<WelcomePane mode="directory" onApplyPair={onApplyPair} />);
```
（删除每个 render 里的 `onDrop={...} onPickLeft={...} onPickRight={...}`。其余断言不变。）

- [ ] **Step 3: 运行 WelcomePane 测试**

Run: `cd frontend && npx vitest run src/components/WelcomePane.test.tsx`
Expected: PASS（4 passed）

- [ ] **Step 4: 改 `Sidebar`（删目录槽/下拉与相关 props）**

`frontend/src/components/Sidebar.tsx`：
1. props 类型与解构删除：`leftDir`、`rightDir`、`onPickLeft`、`onPickRight`、`onApplyPair`、`onPastePath`。
2. 删除 `const [pickMenu, setPickMenu] = useState...`（约 56 行）。
3. 删除监听 pickMenu 外部点击的 `useEffect`（约 83-91 行）。
4. 删除 `return` 内的 `<div className="sidebar__slots"> ... </div>` 整块（约 106-127 行，含 `pickMenu` 下拉）。
5. 删除文件底部 `DirChip` 组件定义（约 206-215 行）。
6. 删除随之未使用的 import：`loadRecent`（`../lib/recentDirs`）。`basename` 仍被 `DirChip` 之外引用？——`basename` 仅 `DirChip` 用；若删除后无引用则一并删除 `basename` 函数；如 `tsc` 报未使用再删。

- [ ] **Step 5: 改 `Sidebar.test.tsx`**

1. `renderBar` 的 `props` 删除 `leftDir/rightDir/onPickLeft/onPickRight/onApplyPair/onPastePath` 六项。
2. 删除两条针对已移除下拉的用例：`'dir chip opens a menu with recent pairs and applies one'` 与 `'pasting a path + Enter calls onPastePath for that side'`。
3. 其余用例不变（行渲染、搜索、筛选、进度、空态）。

- [ ] **Step 6: 运行 Sidebar 测试**

Run: `cd frontend && npx vitest run src/components/Sidebar.test.tsx`
Expected: PASS（剩余用例全通过）

- [ ] **Step 7: 改 `App.tsx`：去 DetailHeader 芯片 + 收敛 props**

1. `DetailHeader` 内：删除 `chips` 计算（`const isSingle = ...` 与 `const chips = ... : null;` 整段）以及 `return` 中 `{chips && (<span className="detail-head__chips"> ... </span>)}` 整块。`name` 计算保留（注意 `name` 当前用了 `wb.view`，与 `isSingle` 无关，保留）。
2. `App` 渲染 `WelcomePane` 处（约 174-179 行）改为：
   ```tsx
   <WelcomePane mode={wb.mode}
     onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }} />
   ```
3. `App` 渲染 `Sidebar` 处删除已移除的 props：`leftDir={wb.leftInput} rightDir={wb.rightInput}`、`onPickLeft=...`、`onPickRight=...`、`onApplyPair=...`、`onPastePath=...`。保留其余（`summary/filteredItems/...` 等）。

- [ ] **Step 8: 改 `App.test.tsx`：改写依赖旧槽的用例**

8a. 在文件内加一个按 `data-side` 打开槽的小工具，并重写 `setupDirectoryScan`：
```tsx
function openBarSlot(side: 'left' | 'right') {
  const slot = document.querySelector(`.selbar__slot[data-side="${side}"] .selbar__slot-main`) as HTMLElement;
  fireEvent.click(slot);
}

async function setupDirectoryScan() {
  vi.mocked(workbenchApi.pickFolder!).mockResolvedValueOnce('/L').mockResolvedValueOnce('/R');
  render(<App />);
  fireEvent.click(screen.getByText('目录'));
  await act(async () => {
    openBarSlot('left');
    fireEvent.click(screen.getByRole('button', { name: '浏览…' }));
  });
  await act(async () => {
    openBarSlot('right');
    fireEvent.click(screen.getByRole('button', { name: '浏览…' }));
  });
}
```

8b. 重写 `'single file mode: pick left → ...'` 用例的选择动作（其余断言不变）：
```tsx
render(<App />);
await act(async () => {
  (document.querySelector('.selbar__slot[data-side="left"] .selbar__slot-main') as HTMLElement).click();
});
await act(async () => {
  fireEvent.click(screen.getByRole('button', { name: '浏览…' }));
});
expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: false }));
await waitFor(() => expect(document.querySelector('.detail-head')).not.toBeNull());
expect(document.querySelector('.detail-head__name')?.textContent).toContain('photo.png');
```

8c. 重写 `'renders the welcome pane with pick slots on first load'`：
```tsx
it('renders the selection bar with two empty slots on first load', () => {
  render(<App />);
  expect(document.querySelector('.selbar')).not.toBeNull();
  expect(screen.getAllByText(/点击选择PNG 文件/)).toHaveLength(2);
  expect(document.querySelector('.sidebar')).toBeNull();
  expect(document.querySelector('.detail-head')).toBeNull();
});
```

8d. `'mode toggle switches the welcome noun between file and folder'` 保留（WelcomePane 标题文案未删）；如该用例还断言 `浏览` 按钮数量，请删除该断言（现已无 `浏览` 文案，仅 `浏览…`）。检查该用例当前仅断言 `拖入文件夹`/`拖入PNG 文件`，无需改。

8e. 其余使用 `setupDirectoryScan()` 的用例（目录扫描、点击另一行）自动适配，无需改动其断言。

- [ ] **Step 9: 全量测试 + 类型检查**

Run: `cd frontend && npm run build && npx vitest run`
Expected: tsc 退出 0；全部 PASS

- [ ] **Step 10: Commit**

```bash
git add frontend/src/components/WelcomePane.tsx frontend/src/components/WelcomePane.test.tsx \
        frontend/src/components/Sidebar.tsx frontend/src/components/Sidebar.test.tsx \
        frontend/src/App.tsx frontend/src/App.test.tsx
git commit -m "refactor(selection): SelectionBar replaces welcome/sidebar/detail-head selectors"
```

---

## 收尾验证（人工）

计划全部完成后，在 `png_metadata_compare/` 下：
```bash
cargo tauri build
```
安装/运行产物，手动确认：
1. 单文件/目录模式下选择条都显示"已选样式"，`✕` 可清除、点槽可重选（浏览/粘贴/最近）。
2. 只选一侧时点「图片」能看到完整单图，可缩放/平移/重置。
3. 双图镜像视图与原先一致。

> 备注：本计划不含已在工作区中的"窗口拖动"修复，那部分按需单独提交。

## Self-Review（已核对）

- **Spec 覆盖**：G1 反馈 → Task 4/5/6；G2 清除/重选 → Task 3（clearSide）+ Task 4/5；G3 单图 → Task 1/2；去重 → Task 6。✓
- **与 spec 的差异**：副作用守卫保留（非"始终 runAuto"），见 Task 3 顶部说明，行为等价。✓
- **占位符**：无 TODO/TBD，所有步骤含真实代码与命令。✓
- **类型/命名一致**：`useImageTransform`/`ImageTransform`、`SoloImage`、`clearSide`、`resetToWelcome`、`SelectionBar` props（`onClear`/`onDrop`/`onPastePath`/`onApplyPair`）跨任务一致；DOM 约定（`.selbar__slot[data-side]`/`.selbar__slot-main`/`aria-label="清除左侧"`/`浏览…`/`粘贴路径后回车`）在组件与测试间一致。✓
