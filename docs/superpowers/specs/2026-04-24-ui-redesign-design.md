# UI 全面重设计 — Design Spec

**Date:** 2026-04-24  
**Project:** PNG Metadata Compare (Tauri + React/TypeScript)  
**Design Reference:** MotherDuck Design System Guide (Complete Edition) + Interaction Design Guide  

---

## 1. 设计目标

将现有 UI 从「样式接近 MotherDuck」升级为「完整符合 MotherDuck 规范」，同时重构信息架构以更好地服务核心用户故事：

- **单文件对比**：选两个 PNG → 比较元数据差异 → 探索具体变化
- **目录批量扫描**：选两个目录 → 扫描所有 PNG 对 → 概览结果 → 下钻到具体文件对

---

## 2. 信息架构：两页面模型

### 当前问题
现有布局是单页三栏（ResultRail + 主区 + Inspector），在目录模式下把「结果列表」和「详情分析」塞在同一屏幕，导致空间紧张、层级不清晰。

### 新架构
```
App
├── 目录总览页 (DirectoryOverview)   ← 仅目录模式
│     └── 点击卡片 →
└── 对比详情页 (PairComparison)      ← 单文件模式 + 目录 drill-down 共用
```

**单文件模式**：直接进入 PairComparison，TopBar 不显示返回按钮。  
**目录模式**：先进 DirectoryOverview，点卡片进 PairComparison，TopBar 显示「← Directory」返回。

模式切换：TopBar 右侧 Segmented Control（Single File / Directory）。切换时重置输入状态。

---

## 3. 视觉设计系统

### 3.1 颜色 Tokens（与现有 tokens.css 对齐，补充缺失项）

| Token | HEX | 用途 |
|-------|-----|------|
| `--color-bg` | `#F4EFEA` | 全局页面背景 |
| `--color-surface` | `#F8F8F7` | Toolbar、TopBar 次级背景 |
| `--color-card` | `#FFFFFF` | 卡片、面板背景 |
| `--color-ink` | `#383838` | 主文字、边框、深色背景 |
| `--color-muted` | `#818181` | 辅助标签、panel header |
| `--color-disabled` | `#A1A1A1` | 禁用态文字 |
| `--color-accent-yellow` | `#FFDE00` | 活跃状态、CTA 高亮、diff modified |
| `--color-accent-blue` | `#6FC2FF` | 主 CTA 背景、hover 边框、input focus shadow |
| `--color-accent-red` | `#FF7169` | 错误状态、diff removed |
| `--color-accent-green` | `#22C55E` | 成功状态、diff added |
| `--color-accent-teal` | `#6FC2FF` | left-only / right-only 状态（复用蓝色） |
| `--color-diff-mod-bg` | `#FFFDE7` | Diff 修改行背景（浅黄） |
| `--color-diff-add-bg` | `#E8F5E9` | Diff 新增行背景（浅绿） |
| `--color-diff-rem-bg` | `#FFEBE9` | Diff 删除行背景（浅红） |
| `--color-border-light` | `#F1F1F1` | 内部分隔线（panel 内行间） |

### 3.2 字体

商业字体 Aeonik Mono 不可用，使用以下备用栈：

```css
--font-mono: 'IBM Plex Mono', 'Cascadia Mono', 'Courier New', monospace;
```

所有 UI 文字统一使用 `--font-mono`。CTA、Nav、标签全部 `text-transform: uppercase`。

### 3.3 边框与阴影

- 所有主要组件边框：`2–3px solid #383838`，`border-radius: 0`（按钮 `border-radius: 2px`）
- 卡片/面板 hover 阴影：`box-shadow: #383838 -4px 4px 0px 0px`（左下偏移，模拟立体浮起）
- Input focus 阴影：`box-shadow: #6FC2FF -3px 3px 0px 0px`
- CTA 按钮按下阴影（Suggestion/Submit 变体）：`box-shadow: #383838 -2px 2px 0px 0px`

---

## 4. 组件规格

### 4.1 TopBar

```
背景: #383838
文字: #FFFFFF
高度: ~40px
padding: 10px 20px
边框底部: 无（与内容区分隔靠下方 toolbar 的 border-bottom）

内容布局 (flex, space-between):
  左: [返回按钮（目录 drill-down 时）] [Brand "PNG ⌁ Compare"]
  中: 当前文件名（目录 drill-down 时，#FFDE00 高亮）
  右: 模式切换 Segmented Control（总览页）/ 进度信息（详情页，如 "1 / 5 different"）
```

**模式切换按钮：**
```css
.mode-btn {
  padding: 5px 14px;
  border: 2px solid #fff;
  background: transparent;
  color: #fff;
  font-size: 10px;
  text-transform: uppercase;
}
.mode-btn.active {
  background: #FFDE00;
  color: #383838;
  border-color: #FFDE00;
}
```

**返回按钮：** color `#6FC2FF`，hover 下划线，无边框。

### 4.2 Toolbar

```
背景: #F8F8F7
border-bottom: 2px solid #383838
padding: 10px 20px
布局: flex, gap 8px, align-items: stretch
```

**Path 输入组：**
- Label（`font-size: 9px; text-transform: uppercase; color: #818181`）
- Input + Choose 按钮并排（input `border-right: 0`）
- Input focus：`box-shadow: #6FC2FF -3px 3px 0px 0px; transition: box-shadow 0.15s`

**CTA 双层容器（核心交互）：**
```
外层 .cta-outer:
  background: #383838
  border-radius: 2px
  padding: 2px

内层 .cta-btn:
  background: #6FC2FF
  color: #383838
  border: 2px solid #383838
  padding: 9px 20px
  font-size: 11px; text-transform: uppercase
  transition: transform 0.12s ease-in-out

.cta-outer:hover .cta-btn  → transform: translate(4px, -4px)
.cta-outer:active .cta-btn → transform: none
```

**vs 分隔符：** `font-size: 11px; color: #818181`，自身 flex-shrink: 0。

### 4.3 Status Dot

```css
.dot {
  width: 9px; height: 9px;
  border-radius: 50%;
  border: 1.5px solid #383838;
}
.dot-diff { background: #FFDE00 }
.dot-same { background: #22C55E }
.dot-err  { background: #FF7169 }
.dot-only { background: #6FC2FF }
```

---

## 5. 页面 1：目录总览（DirectoryOverview）

### 5.1 布局（从上到下）

1. **TopBar** — Brand + 模式切换
2. **Toolbar** — 左右目录路径 + Scan CTA
3. **Stats Bar** — 各状态计数 chip（仅扫描完成后显示）
4. **Filter Bar** — All / Different / Identical / Left-only / Right-only / Error
5. **Card Grid** — 文件对卡片

### 5.2 Stats Bar

```
背景: #F8F8F7
border-bottom: 2px solid #383838
padding: 7px 20px
布局: flex, gap 8px

每个 chip:
  border: 2px solid #383838
  background: #fff
  padding: 3px 12px
  内容: [dot] [数字] [状态文字]
```

### 5.3 Filter Bar

```
背景: #F8F8F7
border-bottom: 2px solid #383838
padding: 6px 20px

每个 filter-btn:
  border: 2px solid #383838
  border-right-width: 0（最后一个恢复）
  padding: 4px 14px
  font-size: 10px; text-transform: uppercase
  background: #fff

.filter-btn.active:
  background: #383838
  color: #fff

右侧: 文件总数（color: #818181; margin-left: auto）
```

过滤逻辑：客户端过滤已加载的 DirectorySummary，无需重新请求后端。

### 5.4 Card Grid

```
padding: 16px 20px
display: grid
grid-template-columns: repeat(auto-fill, minmax(200px, 1fr))
gap: 12px
```

**File Card：**
```css
.file-card {
  border: 2px solid #383838;
  background: #fff;
  cursor: pointer;
  transition: transform 0.15s ease-out, box-shadow 0.15s ease-out, border-color 0.15s;
}
.file-card:hover {
  transform: translateY(-4px) scale(1.01);
  box-shadow: #383838 -4px 4px 0px 0px;
  border-color: #6FC2FF;
}
.file-card:active {
  transform: translateY(-2px) scale(1.005);
  box-shadow: #383838 -2px 2px 0px 0px;
}
```

**Card Header（状态色）：**
```
border-bottom: 2px solid #383838
padding: 7px 10px
font-size: 10px; text-transform: uppercase; font-weight: 600
display: flex; align-items: center; gap: 6px

.header-diff { background: #FFDE00 }
.header-same { background: #E8F5E9 }
.header-err  { background: #FFEBE9 }
.header-only { background: #EBF9FF }
```

**Card Body：**
```
padding: 8px 10px
.card-name: font-size 11px, font-weight 600, word-break: break-all
.card-meta: font-size 10px, color #818181
            error 时 color #FF7169
```

**卡片进场动画：**
```css
@keyframes cardEnter {
  from { opacity: 0; transform: translateY(8px) }
  to   { opacity: 1; transform: translateY(0) }
}
/* 每张卡片 animation-delay: index * 30ms，最多 300ms */
```

点击卡片：导航到 PairComparison 视图，将该文件对的左右路径预填入 Toolbar，并自动触发 `compare_single` 调用（无需用户再点 Compare）。TopBar 显示返回按钮和当前位置（如 "2 / 5 different"，仅统计 different 状态的卡片序号）。

---

## 6. 页面 2：对比详情（PairComparison）

### 6.1 布局（从上到下）

1. **TopBar** — 返回按钮（目录来源时）+ Brand + 文件名 + 进度
2. **Toolbar** — 左右文件路径 + Compare CTA
3. **View Mode Strip** — Tree / JSON / Image + Diff 高亮开关 + 变更计数
4. **Split Body** — 左面板 | Diff 摘要竖条 | 右面板

### 6.2 View Mode Strip

```
背景: #F8F8F7
border-bottom: 2px solid #383838
padding: 7px 20px
布局: flex, align-items: center, gap: 16px

"View" label: font-size 9px, uppercase, color #818181

Segmented Control (.seg-group):
  每个 .seg:
    border: 2px solid #383838
    border-right-width: 0（最后一个恢复）
    padding: 4px 14px
    font-size: 10px; text-transform: uppercase
    background: #fff
  .seg.active:
    background: #6FC2FF; font-weight: 600

右侧 (margin-left: auto):
  "Highlight Diffs" 文字 + Toggle 开关 + 变更数 badge
  
Toggle ON 状态:
  background: #FFDE00; border: 1.5px solid #383838
  knob 在右侧（#383838 圆形）

变更数 badge:
  background: #FFDE00; border: 1.5px solid #383838
  padding: 3px 10px; font-size: 10px
  显示 "N changes"，0 changes 时 background: #F1F1F1
```

### 6.3 Split Body

```
display: flex
min-height: calc(100vh - [topbar+toolbar+viewstrip 高度])
```

**左/右面板（.split-panel）：**
```css
flex: 1;
border-right: 2px solid #383838; /* 右面板无此边框 */
padding: 12px 14px;
background: #fff;
overflow-y: auto;
```

Panel Header：`font-size: 9px; text-transform: uppercase; color: #818181; border-bottom: 1.5px solid #F1F1F1; margin-bottom: 8px; padding-bottom: 5px`

**Diff 摘要竖条（.diff-strip）：**
```css
width: 140px;
flex-shrink: 0;
border-left: 2px solid #383838;
border-right: 2px solid #383838;
background: #FFFDE7;
padding: 10px 8px;
overflow-y: auto;
```

每条 diff-row：
```css
.diff-row { font-size: 10px; padding: 3px 6px; margin-bottom: 3px }
.diff-row.mod { background: #FFFDE7; border-left: 2px solid #E1C427 }
.diff-row.add { background: #E8F5E9; border-left: 2px solid #22C55E }
.diff-row.rem { background: #FFEBE9; border-left: 2px solid #FF7169 }
```

若无差异，竖条显示：`"No changes"` 居中，color `#818181`。

### 6.4 Tree 视图

每个 .tree-node：
```css
padding: 3px 0;
display: flex; align-items: center; gap: 5px;
font-size: 10px; cursor: pointer;
transition: background 0.1s;
```

- hover → `background: #F4EFEA`
- Diff 高亮开关 ON 时：
  - modified 节点 → `background: #FFFDE7`
  - added 节点 → `background: #E8F5E9`
  - removed 节点 → `background: #FFEBE9`

展开/折叠：点击父节点切换，子节点用 `contentSlideUp` 动画进入。

```css
@keyframes contentSlideUp {
  from { opacity: 0; transform: translateY(8px) }
  to   { opacity: 1; transform: translateY(0) }
}
```

Node dot（7px 圆）颜色：普通 `#818181`，modified `#E1C427`，added `#22C55E`，removed `#FF7169`。

### 6.5 JSON 视图

左右各显示对应文件的格式化 JSON，等宽字体，行内语法高亮（key 深色，string 蓝色，number 绿色，null 红色）。Diff 高亮开关 ON 时，差异行整行背景色与 Tree 视图一致。

### 6.6 Image 视图

左右各显示对应 PNG 缩略图（object-fit: contain），下方各有「Open Original ↗」按钮（次级按钮样式）。中间 Diff 竖条显示元数据变更数（无图片 diff）。

图片加载前显示骨架占位：`background: #F1F1F1; border: 1.5px solid #C0C0C0`。

### 6.7 Empty State

比较前（无结果）：居中显示提示文字 + 指示箭头，color `#818181`，`font-size: 13px; text-transform: uppercase`。

---

## 7. 状态与错误处理

| 状态 | 处理方式 |
|------|---------|
| 扫描/比较中 | Scan/Compare 按钮文字变为 "Scanning…" / "Comparing…"，禁用态样式 |
| 后端报错 | TopBar 下方出现 StatusBanner（`background: #FFEBE9; border-bottom: 2px solid #FF7169`），自动消失 |
| 目录扫描 0 结果 | Card Grid 区域显示 Empty State |
| 文件 parse 失败 | 卡片显示 Error 状态，详情页 Split Body 显示错误信息 |

---

## 8. 状态管理重构

现有 `useWorkbench.ts` 以单页三栏为模型设计，需重构以支持两页面架构。

**新状态结构：**
```typescript
type AppView = 'directory-overview' | 'pair-comparison'

interface AppState {
  mode: 'single' | 'directory'
  view: AppView
  // 输入
  leftPath: string
  rightPath: string
  // 目录模式结果
  directorySummary: DirectorySummary | null
  activeFilter: 'all' | 'different' | 'identical' | 'left-only' | 'right-only' | 'error'
  // 对比详情
  pairResult: PairInspection | null
  // 从目录来时的上下文（用于返回和进度显示）
  directoryContext: { index: number; totalDifferent: number } | null
  // UI 状态
  viewMode: 'tree' | 'json' | 'image'
  diffHighlight: boolean
  loading: boolean
  error: string | null
}
```

`useWorkbench.ts` 保持为自定义 hook，删除 `activeInspection`、`selectedNode`、`activeTab` 等已不存在的状态字段。

---

## 9. 组件变更清单（对比现有代码）

| 现有组件 | 变更 |
|---------|------|
| `App.tsx` | 重构路由逻辑：DirectoryOverview ↔ PairComparison 切换 |
| `Toolbar.tsx` | 重建：添加 path label、双层 CTA、Choose 按钮并排 |
| `TabBar.tsx` | **删除**，替换为 View Mode Strip（Segmented Control）|
| `ResultRail.tsx` | **删除**，替换为 DirectoryOverview 页面（Card Grid）|
| `PreviewStrip.tsx` | **删除**，信息整合到 TopBar |
| `InspectorPanel.tsx` | **删除**，详情直接在 Split Panel 内展示 |
| `DiffTree.tsx` | 保留逻辑，重建样式（inline diff highlight）|
| `MetadataTree.tsx` | 保留逻辑，重建样式 |
| `RawJsonPanel.tsx` | 保留逻辑，整合到 JSON 视图 |
| `ImagePane.tsx` | 保留逻辑，整合到 Image 视图 |
| `EmptyState.tsx` | 保留，更新样式 |
| `StatusBanner.tsx` | 保留，更新样式 |
| `tokens.css` | 补充缺失 tokens（diff 背景色、disabled 色）|
| `app.css` | 完全重写 |
| **新建** `DirectoryOverview.tsx` | Card Grid 页面 |
| **新建** `PairComparison.tsx` | Split-first 对比详情页 |
| **新建** `ViewModeStrip.tsx` | Tree/JSON/Image 切换条 |
| **新建** `DiffStrip.tsx` | 中间 Diff 摘要竖条 |
| **新建** `FileCard.tsx` | 目录卡片 |

---

## 10. 不在本次范围内

- 键盘快捷键 / 无障碍（aria）
- 暗色模式
- 响应式（移动端）—— 保持桌面优先
- 后端 API 变更 —— 三个 Tauri 命令接口不变
