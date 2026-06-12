# 比对树层级视觉改进 — 设计

日期：2026-06-12
状态：已确认（方案 A，四项全选）

## 问题

`UnifiedTree` 中分组标题（`GroupHead`）不随层级缩进：「停靠线路」（0 层）、「线路 1 · M208」（1 层）、「途经站点」（2 层）全部顶格渲染，仅叶子字段按 `10 + level * 16` 行内缩进；`.utree__nested` 的 `border-left` 因容器无 padding 全部叠在同一 x 位置。展开后不同层级的标题混在一列，归属关系不直观。

## 目标

1. **层级缩进**：分组标题与叶子字段随层级逐层右移、相互对齐。
2. **竖向引导线**：每层嵌套一条浅色竖线，连接同一分组的子项。
3. **标题分级降调**：层级越深标题样式越轻，三档（0 / 1 / ≥2）。
4. **分组标题吸顶**：滚动长列表时当前分组标题粘性固定，知道自己在哪条线路里。

## 方案：容器驱动缩进（方案 A）

缩进、引导线、吸顶全部由嵌套容器 `.utree__nested` 的结构自然产生，不做负边距通铺。
已知代价并接受：hover 与差异底色（添加/删除/修改行）从缩进位置开始，不通铺到最左（GitHub 文件树风格，左侧参差反而强化层级）。

### 改动点

**`frontend/src/components/UnifiedTree.tsx`**
- `Leaf` 删除行内 `paddingLeft: 10 + level * 16`，改用 CSS 基础缩进（行内不再依赖 level）。
- 嵌套容器保持现有 `.utree__nested`，缩进由 CSS 承担。

**`frontend/src/components/GroupHead.tsx`**
- 输出 `data-level`（0 / 1 / 2，≥2 一律为 2），替换现有二档的 `group-head--nested` class；样式分档全部走 `[data-level]` 选择器。
- 测试同步：`GroupHead.test.tsx` 断言 data-level。

**`frontend/src/styles/app.css`**
- `.utree__nested { margin-left: 10px; padding-left: 14px; border-left: 1px solid var(--border-subtle); }` —— 引导线位置落在父标题折叠箭头下方；缩进量 = 10(margin) + 14(padding)，深层自然累计。
- 标题三档：
  - `[data-level="0"]`：维持现状（底色横条 + 2px 左色条 + 大写字距）。
  - `[data-level="1"]`：无底色、保留左色条、字号略小、颜色次级。
  - `[data-level="2"]`：无底色无色条、最小字号、三级文字色。
  - 差异状态色（--add/--rem/--mod/--err）优先级高于分档降调（保持现有行为）。
- 吸顶：`.group-head { position: sticky; }`，`top` 按层级累计（0 层 `top: 0`，1 层 `top: H0`，2 层 `top: H0+H1`，H 为各档标题实际高度，用 CSS 变量定义）；吸顶标题加实底背景（`var(--bg-panel)`）避免滚动内容穿透；`z-index` 浅层更高。≥3 层的标题不吸顶（`position: static`），避免堆叠过高。
- 风险：`.utree__nested` 现有 `content-visibility: auto` 与 sticky 的交互需实测；若引导线/吸顶在跳转定位（flash 动画、scrollIntoView）下异常，对受影响层级关闭 `content-visibility` 降级处理。

### 不做

- 虚拟滚动、行级引导线绘制、负边距通铺（方案 B）。
- 展开/收起的交互逻辑变化（默认展开态、onlyDiff 联动等维持现状）。

## 验证

- `GroupHead.test.tsx`：三档 data-level 渲染断言。
- `UnifiedTree.test.tsx`：嵌套结构 + 叶子不再有行内 padding 的断言（如有必要）。
- 全套 vitest 通过；`cargo tauri build --no-bundle` 后启动应用截图，人工确认四项视觉效果与跳转定位（n/p、差异栏点击）不回归。
