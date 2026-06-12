# 比对树层级视觉改进 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 比对树的分组标题随层级缩进并配引导线、分级降调、滚动吸顶，让层级归属一眼可读。

**Architecture:** 方案 A（容器驱动）：缩进与引导线由嵌套容器 `.utree__nested` 的 margin/padding/border 自然产生；`GroupHead` 输出 `data-level`（封顶 3），样式三档降调 + 0/1/2 层 sticky 吸顶（第 3 层静态）。差异底色从缩进位置开始（接受的代价，见 spec）。

**Tech Stack:** React + TypeScript（vitest/@testing-library）、纯 CSS（frontend/src/styles/app.css）。

**Spec:** docs/superpowers/specs/2026-06-12-tree-hierarchy-visual-design.md

实际层级结构（重要）：顶层组（停靠线路/分站列表）= level 0，线路条目 = 1，途经站点数组 = 2，站点条目 = 3，站点字段叶子 = 4。所以 data-level 封顶 3：`0/1/2` 吸顶，`3` 与 `2` 同样式但不吸顶。

---

### Task 1: GroupHead 输出 data-level（替换 group-head--nested 二档）

**Files:**
- Modify: `frontend/src/components/GroupHead.tsx`
- Test: `frontend/src/components/GroupHead.test.tsx`

- [x] **Step 1: 改写失败测试**

把 `GroupHead.test.tsx` 中现有的 `applies nested class when level > 0` 测试（第 22-25 行）替换为：

```tsx
  it('emits data-level capped at 3 and drops the legacy nested class', () => {
    const { container, rerender } = render(<GroupHead label="x" />);
    expect(container.firstChild).toHaveAttribute('data-level', '0');
    rerender(<GroupHead label="x" level={1} />);
    expect(container.firstChild).toHaveAttribute('data-level', '1');
    rerender(<GroupHead label="x" level={2} />);
    expect(container.firstChild).toHaveAttribute('data-level', '2');
    rerender(<GroupHead label="x" level={5} />);
    expect(container.firstChild).toHaveAttribute('data-level', '3');
    expect(container.firstChild).not.toHaveClass('group-head--nested');
  });
```

- [x] **Step 2: 运行确认失败**

Run: `cd frontend; npx vitest run src/components/GroupHead.test.tsx`
Expected: FAIL —— `data-level` 属性不存在。

- [x] **Step 3: 最小实现**

`GroupHead.tsx`：删除 className 数组里的 `level > 0 ? 'group-head--nested' : ''` 一项，根元素加 `data-level`：

```tsx
  const statusCls = highlight && status ? STATUS_CLASS[status] : undefined;
  const cls = ['group-head', statusCls ?? ''].filter(Boolean).join(' ');
  return (
    <div className={cls} data-level={Math.min(level, 3)}>
```

（其余 props 与子元素不变；`level` prop 保留。）

- [x] **Step 4: 运行确认通过**

Run: `cd frontend; npx vitest run src/components/GroupHead.test.tsx`
Expected: PASS（全文件）。

- [x] **Step 5: Commit**

```bash
git add frontend/src/components/GroupHead.tsx frontend/src/components/GroupHead.test.tsx
git commit -m "refactor(tree): GroupHead emits data-level (cap 3), drop nested class"
```

---

### Task 2: 容器缩进 + 引导线 + 叶子去掉行内缩进

**Files:**
- Modify: `frontend/src/components/UnifiedTree.tsx:160`（Leaf 的 key span）
- Modify: `frontend/src/styles/app.css:327`（.utree__nested）、`app.css:329`（.utree__key）

- [x] **Step 1: Leaf 删除行内 paddingLeft**

`UnifiedTree.tsx` Leaf 组件中：

```tsx
      <span className="utree__key">
        {row.label}
        {row.isUnknown && <span className="utree__unknown">未识别</span>}
      </span>
```

（原来是 `<span className="utree__key" style={{ paddingLeft: 10 + level * 16 }}>`。`Leaf` 的 `level` 参数随之不再使用——从 `Leaf` 的 props 和 `RowView` 里对 `<Leaf ... level={level}>` 的传参中一并删除，避免未使用参数的 lint 报错。）

- [x] **Step 2: CSS 容器缩进 + 引导线 + 叶子基础缩进**

`app.css` 替换这两行：

```css
.utree__nested {
  margin-left: 10px;
  padding-left: 14px;
  border-left: 1px solid var(--border-subtle);
  content-visibility: auto;
  contain-intrinsic-size: auto 600px;
}
```

```css
.utree__key { color: var(--text-secondary); padding-left: 10px; }
```

- [x] **Step 3: 全套前端测试**

Run: `cd frontend; npx vitest run`
Expected: 全部通过（无测试断言行内 padding，已确认）。

- [x] **Step 4: Commit**

```bash
git add frontend/src/components/UnifiedTree.tsx frontend/src/styles/app.css
git commit -m "feat(tree): container-driven per-level indent with vertical guide lines"
```

---

### Task 3: 标题三档降调（data-level 样式）

**Files:**
- Modify: `frontend/src/styles/app.css:107-136`（.group-head 块）

- [x] **Step 1: 替换样式**

删除 `.group-head--nested` 块（app.css:122-128），`.group-head` 基础块后新增分档（先不加 sticky，Task 4 做）：

```css
.group-head[data-level="1"] {
  font-size: 10px;
  color: var(--group-head-nested);
  border-left-color: var(--group-rule-nested);
  margin: var(--sp-2) 0 2px;
}
.group-head[data-level="2"],
.group-head[data-level="3"] {
  font-size: 10px;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  border-left-color: transparent;
  margin: var(--sp-2) 0 2px;
}
```

注意：原 `--nested` 块里的 `background: transparent` 不再需要（基础块背景在 Task 4 统一改为实底）。状态色类（`.group-head--add` 等，app.css:133-136）声明在分档之后、选择器特异性相同时靠源顺序覆盖分档——保持这 4 行位于分档规则**之后**即可。

- [x] **Step 2: 全套前端测试 + 提交**

Run: `cd frontend; npx vitest run`
Expected: 全部通过。

```bash
git add frontend/src/styles/app.css
git commit -m "feat(tree): three-tier group head styling by data-level"
```

---

### Task 4: 吸顶（sticky）+ 实底背景

**Files:**
- Modify: `frontend/src/styles/app.css`（.group-head 基础块 + 分档块 + 状态色块）

- [x] **Step 1: 基础块加 sticky 与实底**

`.group-head` 基础块（app.css:107-121）改为：

```css
.group-head {
  font: 500 var(--fs-xs)/var(--lh-xs) var(--font-ui);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--group-head);
  padding: 6px 10px;
  background: var(--bg-page);   /* 吸顶时挡住滚动内容；原 --group-bg 即 transparent */
  border-left: 2px solid var(--group-head);
  border-radius: 2px;
  margin: var(--sp-3) 0 var(--sp-1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  position: sticky;
  top: 0;
  z-index: 5;
  min-height: 28px;
  box-sizing: border-box;
}
```

- [x] **Step 2: 分档偏移与第 3 层不吸**

Task 3 的两个分档块里追加 sticky 相关行，变为：

```css
.group-head[data-level="1"] {
  font-size: 10px;
  color: var(--group-head-nested);
  border-left-color: var(--group-rule-nested);
  margin: var(--sp-2) 0 2px;
  top: 28px;
  z-index: 4;
  min-height: 24px;
}
.group-head[data-level="2"],
.group-head[data-level="3"] {
  font-size: 10px;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  border-left-color: transparent;
  margin: var(--sp-2) 0 2px;
  top: 52px;
  z-index: 3;
  min-height: 22px;
}
.group-head[data-level="3"] { position: static; }
```

（偏移 = 上层 min-height 累计：0 层 28 → 1 层 top 28；1 层 24 → 2 层 top 52。）

- [x] **Step 3: 状态色块换成不透明叠色**

状态色背景是半透明（--add-bg 等），吸顶会透出底下内容。app.css:133-136 替换为：

```css
.group-head--add { background: linear-gradient(var(--add-bg), var(--add-bg)) var(--bg-page); border-left-color: var(--add-text); color: var(--add-text); }
.group-head--rem { background: linear-gradient(var(--rem-bg), var(--rem-bg)) var(--bg-page); border-left-color: var(--rem-text); color: var(--rem-text); }
.group-head--mod { background: linear-gradient(var(--mod-bg), var(--mod-bg)) var(--bg-page); border-left-color: var(--mod-new); color: var(--mod-new); }
.group-head--err { background: linear-gradient(rgba(255, 124, 224, 0.10), rgba(255, 124, 224, 0.10)) var(--bg-page); border-left-color: var(--err-text); color: var(--err-text); }
```

- [x] **Step 4: 全套前端测试 + 提交**

Run: `cd frontend; npx vitest run`
Expected: 全部通过。

```bash
git add frontend/src/styles/app.css
git commit -m "feat(tree): sticky group heads with stacked per-level offsets"
```

---

### Task 5: 构建 + 视觉验证（含 content-visibility 回退决策）

**Files:**
- 无源码改动预期；若 sticky 在 `content-visibility: auto` 容器内异常 → Modify: `frontend/src/styles/app.css`（.utree__nested）

- [x] **Step 1: 构建**

Run: `cargo tauri build --no-bundle`
Expected: `Built application at: ...\target\release\png_metadata_compare.exe`

- [x] **Step 2: 启动应用并截图验证**

用截图回路（启动 exe → 截图 → 杀进程，参考既往 tmp/diag/shot.ps1 模式，临时脚本放 tmp/，验证完删除）。选一对有线路差异的 PNG（tmp/ 下的 石清大道 对），逐项确认：
1. 分组标题随层级右移，叶子字段与标题对齐成阶梯；
2. 每层一条竖向引导线，位置在父标题箭头下方；
3. 三档标题强弱分明（0 层横条最强 → 2/3 层最轻）；
4. 展开 32 项途经站点后滚动：停靠线路/线路 N/途经站点 三条标题依次叠在顶部，文字无穿透；
5. 差异栏点击跳转（flash 高亮 + 滚动定位）仍正常。

- [x] **Step 3: 若第 4/5 项异常 → 移除 content-visibility 回退**

```css
.utree__nested {
  margin-left: 10px;
  padding-left: 14px;
  border-left: 1px solid var(--border-subtle);
}
```

（当前数据规模（数百行）下放弃渲染跳过的代价可接受。）重跑 Step 1-2 确认。

- [x] **Step 4: 收尾**

Run: `cd frontend; npx vitest run` 与 `cargo test`
Expected: 全部通过。删除临时截图脚本与图片。

```bash
git add -A frontend/src docs/superpowers/plans/2026-06-12-tree-hierarchy-visual.md
git commit -m "feat(tree): hierarchy visual pass verified; fallbacks applied if any"
```

---

## 执行记录（2026-06-12）

全部 5 个任务完成（子代理驱动，每任务双重审查）。计划外的两处修正：状态色选择器补 `[data-level]` 特异性；`data-path` 包裹 div 使 sticky 包含块为零行程的 Critical 由评审发现，修复为 GroupHead `dataPath` prop（1f398de）。Task 5 验证 5 项全过（证据 tmp/diag5/），`content-visibility: auto` 与 sticky 实测无冲突，**未回退**。遗留跟进：`group-head--reord` 自始无对应 CSS 规则（先在先有）。
