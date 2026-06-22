# 常驻选择条 + 单图修复 — 设计

日期：2026-06-22 · 分支：`redesign/ui-comfort`

## 背景与问题

当前选择左右文件/目录的交互存在三处不足：

1. **缺少"已选"反馈**：`WelcomePane` 的左右槽位只显示「左侧 [浏览]」，从不显示选中的文件/目录名。目录模式下只选一侧不会触发扫描（`runAuto` 需要两侧都填），界面停在欢迎页，对已选目录**零反馈**。
2. **无法清除、重选不便**：任何地方都没有"清除某一侧"的控件；唯一的重选入口是 `DetailHeader` 芯片（且只在单文件模式、进入视图后才有，只能重开对话框、不能清除）。
3. **只选一侧时单图看不到**：`SingleImage`（`App.tsx`）复用了为双图设计的 `.image-split`（`grid-template-rows: 36px 1fr`）和 `.image-split__panes`（`grid-template-columns: 1fr 1fr`）。因为单图没有工具条，唯一的 `panes` 落进了 36px 行、且只占左半列 → 图被压成约 36px 高、半宽的细条，几乎不可见（不是加载失败，所以双图镜像视图正常）。

## 目标

- **G1**：选择左/右后有清晰的"已选"样式（文件名/目录名）。单文件、目录两种模式一致。
- **G2**：每一侧可一键清除（✕），并能方便地重选。
- **G3**：只选一侧时，点「图片」能正常看到单张图片，且支持缩放/平移/重置。

## 非目标（YAGNI）

- 槽位里不做缩略图预览。
- 只选一侧时**不**默认进入「图片」视图，仍默认「树」，由用户点「图片」。
- 不新增三项需求之外的功能。

---

## 设计 A：常驻选择条 `SelectionBar`（满足 G1 + G2）

### 位置与可见性
- 新组件 `SelectionBar`，渲染在 `topbar` 与 `shell-body` 之间，**单文件/目录模式、欢迎页/对比页全程常驻**。
- 它成为左右来源选择的**唯一入口**，接管原先分散在三处的选择 UI（见"去重"）。

### 槽位状态
两个槽 `左` / `右`：
- **空**：虚线占位「＋ 点击选择 {PNG 文件 | 文件夹} / 拖入…」。整槽可点、可拖放。
- **已选**：可选小图标 + `basename`（`title` 显示完整路径）+ `✕` 清除按钮。

### 交互（两模式一致）
- 点槽位（空或已选）→ 弹出下拉菜单：**浏览…** / **粘贴路径后回车** / **最近**（`loadRecent(kind)`，`kind = mode === 'directory' ? 'dir' : 'file'`）。沿用现有 `Sidebar` pickmenu 的逻辑与样式，迁移到选择条。
- 点 `✕` → 仅清除该侧（`stopPropagation`，不触发下拉）。
- 拖放文件/目录到槽 → 复用 `tryDropPath(side, path)`（保留跨模式自动切换行为）。
- 点击外部关闭下拉（复用 `Sidebar` 现有的 outside-click effect 模式）。

### Props
```
mode: WorkbenchMode
leftInput: string
rightInput: string
onPickLeft(): void
onPickRight(): void
onPastePath(side: Side, path: string): void
onApplyPair(left: string, right: string): void   // 最近
onClear(side: Side): void                          // 新增
onDrop(side: Side, path: string): void             // -> tryDropPath
```

### 清除语义（`clearSide` + runAuto 复算）
- 在 `useWorkbench` 新增 `clearSide(side)`：把该侧 input 置空。
- 把 `App` 中的副作用从「仅当有输入时 runAuto」改为**始终** `void wb.runAuto()`（依赖仍是 `leftInput, rightInput, mode`），让 `runAuto` 统一复算视图：
  - 单文件，两侧空 → 欢迎页。
  - 单文件，仅剩一侧 → 该侧单图(solo)。
  - 目录，非两侧齐全 → 欢迎页。
- `runAuto` 的两个 "回欢迎页" 分支统一调用 `resetToWelcome()`，**同时清空** `directorySummary / pairResult / soloResult / soloSide / selectedItemId / error`，避免清除后侧栏/内容残留旧结果。

### 去重（确认同意）
- 移除 `WelcomePane` 的左右槽位块；`WelcomePane` 仅保留「最近」列表 + 快捷键提示 + 「拖入…」引导文案。
- 移除 `Sidebar` 的 `sidebar__slots`（`DirChip`）与 pickmenu，以及随之不再使用的 props（`leftDir/rightDir/onPickLeft/onPickRight/onApplyPair/onPastePath`）和 `pickMenu` 状态；`Sidebar` 保留搜索/筛选/列表/页脚/右键菜单。
- 移除 `DetailHeader` 的 `chips` 块及其计算。

---

## 设计 B：单图修复（满足 G3）

### 根因
见"背景"第 3 点：`SingleImage` 复用双图网格，单 pane 落入 36px 行 + 半列 → 塌陷。

### 方案
- 抽出共享 hook `useImageTransform()`：封装 `zoom / offset / dragRef` 与 `onWheel / onMouseDown / onMouseMove / endDrag / zoomIn / zoomOut / reset / transform / dragging`（即 `ImageSplit` 现有内联逻辑）。
- `ImageSplit`（双图）与新的单图视图都用该 hook，去重缩放/平移逻辑。
- 单图视图：工具条（缩放/平移/重置，与双图一致）+ 单列 pane 容器：
  - 复用 `.image-split`（含工具条行 → pane 落入 `1fr` 行），新增修饰类把列改为单列：`.image-split__panes--solo { grid-template-columns: 1fr; }`。
  - 渲染一个 `ImagePane`，`transform` 来自 hook。

---

## 受影响文件

| 文件 | 改动 |
|---|---|
| `frontend/src/components/SelectionBar.tsx` | **新增** 选择条组件（含槽位 + 下拉） |
| `frontend/src/lib/useImageTransform.ts` | **新增** 缩放/平移共享 hook |
| `frontend/src/App.tsx` | 渲染 `SelectionBar`；副作用改为始终 `runAuto`；重写 `SingleImage`（工具条 + 单列 + hook）；`ImageSplit` 改用 hook；删除 `DetailHeader` 芯片 |
| `frontend/src/features/workbench/useWorkbench.ts` | 新增 `clearSide`；`runAuto` 回欢迎页分支统一 `resetToWelcome`（清空结果） |
| `frontend/src/components/WelcomePane.tsx` | 删除左右槽位，保留最近 + 提示 |
| `frontend/src/components/Sidebar.tsx` | 删除目录槽位/pickmenu 及相关 props/state |
| `frontend/src/styles/app.css` | 新增 `selbar__*` 样式；新增 `.image-split__panes--solo` 单列；按需调整 |

## 测试

- `SelectionBar.test.tsx`（新增）：空/已选两态渲染；`✕` 调 `onClear`；点槽位开下拉；粘贴路径回车调 `onPastePath`；点最近调 `onApplyPair`；拖放调 `onDrop`。
- `useWorkbench.test.tsx`（更新）：`clearSide` —— 两侧空→`welcome` 且结果清空；单文件剩一侧→`solo`；目录→`welcome` 且 `directorySummary` 清空。
- `Sidebar.test.tsx` / `WelcomePane.test.tsx`（更新）：移除对已删槽位/pickmenu 的断言。
- `App.test.tsx`（更新）：选择条入口替换原槽位入口的相关断言。
- 单图：组件测试断言单图视图渲染工具条 + 单 pane + 单列类（视觉效果人工在 `cargo tauri build` 后验证）。

## 边界与风险

- 始终调用 `runAuto` 后，挂载时两侧为空会跑一次 → `resetToWelcome`，幂等无副作用。
- 目录扫描进行中清除某侧 → input 变化触发新 `runAuto`（`scanSeqRef` 自增使旧扫描结果作废）走欢迎分支；可顺带调用 `cancelScan`（次要，实现时确认）。
- 槽位下拉与窗口拖动区（`data-tauri-drag-region`）不重叠：选择条不加拖动属性。
