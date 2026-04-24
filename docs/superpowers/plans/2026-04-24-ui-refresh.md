# UI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix window drag, move mode toggle to the left with underline-active style, increase toolbar input height, apply MotherDuck theme refinements, and switch all UI text to Chinese-primary.

**Architecture:** Pure frontend changes — CSS (`app.css`) and React component text/structure (`App.tsx`, `PairComparison.tsx`, `DirectoryOverview.tsx`, `FileCard.tsx`). No backend, no new state, no new components. Each task is independently verifiable by running the dev server.

**Tech Stack:** React 18, Vite, Tauri v2, TypeScript, CSS custom properties (tokens in `tokens.css`)

---

## File Map

| File | Changes |
|---|---|
| `frontend/src/styles/app.css` | drag region, mode-toggle CSS, toolbar height, MotherDuck refinements |
| `frontend/src/App.tsx` | move toggle to left, add separator element, remove `data-tauri-drag-region` |
| `frontend/src/components/PairComparison.tsx` | Chinese labels, placeholders, button text, EmptyState strings |
| `frontend/src/components/DirectoryOverview.tsx` | Chinese labels, placeholders, button text, filter labels, EmptyState strings |
| `frontend/src/components/FileCard.tsx` | Chinese STATUS_LABEL, Chinese cardMeta strings |

---

### Task 1: CSS — Window Drag Region

**Files:**
- Modify: `frontend/src/styles/app.css` (`.topbar`, `.win-btn`, `.mode-btn`, `.back-btn`, `button`, `input`)

The frameless window (`decorations: false` in `tauri.conf.json`) needs `-webkit-app-region: drag` on the topbar element itself, and `-webkit-app-region: no-drag` on every interactive child. Currently only `.win-btn` has `no-drag`; other buttons are missing it.

- [ ] **Step 1: Add drag region to `.topbar`**

In `frontend/src/styles/app.css`, find the `.topbar` rule and add the property:

```css
.topbar {
  background: var(--color-ink);
  color: #fff;
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
  height: 46px;
  -webkit-app-region: drag;
}
```

- [ ] **Step 2: Ensure all interactive topbar children have `no-drag`**

Find the `.win-btn` rule and add no-drag to the other interactive elements too. Add these lines right after the `.win-btn` rule:

```css
.win-btn {
  width: 36px;
  height: 32px;
  background: transparent;
  border: none;
  color: #aaa;
  font-size: 13px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.1s, color 0.1s;
  -webkit-app-region: no-drag;
}

.back-btn {
  background: none;
  border: none;
  color: var(--color-accent-blue);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1px;
  cursor: pointer;
  padding: 0;
  text-decoration: none;
  -webkit-app-region: no-drag;
}
```

Also add to `.mode-btn` (will be updated in Task 2 anyway, but add it now to the existing rule):
```css
/* find the existing .mode-btn rule and add: */
-webkit-app-region: no-drag;
```

- [ ] **Step 3: Remove `data-tauri-drag-region` from App.tsx**

Open `frontend/src/App.tsx`. Remove the `data-tauri-drag-region` attribute from the three topbar divs:

```tsx
{/* Before */}
<div className="topbar-left" data-tauri-drag-region>
<div className="topbar-center" data-tauri-drag-region>
<div className="topbar-right" data-tauri-drag-region>

{/* After */}
<div className="topbar-left">
<div className="topbar-center">
<div className="topbar-right">
```

- [ ] **Step 4: Verify drag works**

Run `npm run tauri dev` from the project root. Once the window opens, click and drag on an empty area of the topbar (left of the brand, or the center area). The window should move. Clicking the mode-toggle buttons should NOT drag — they should respond as buttons.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/styles/app.css frontend/src/App.tsx
git commit -m "fix: add -webkit-app-region drag to topbar for frameless window"
```

---

### Task 2: CSS — Mode Toggle Redesign

**Files:**
- Modify: `frontend/src/styles/app.css` (`.mode-toggle`, `.mode-btn`, `.mode-btn--active`, new `.mode-toggle-sep`)
- Modify: `frontend/src/App.tsx` (move toggle into `.topbar-left`, add separator div)

Replace the current full-yellow-fill active style with a yellow underline, wrap buttons in a thin-border container, and add a 1px inner separator between buttons.

- [ ] **Step 1: Replace the `.mode-toggle` and `.mode-btn` CSS block**

Find and replace the entire section from `/* Mode toggle in topbar` to end of `.mode-btn--active` in `app.css`:

```css
/* ── MODE TOGGLE ── */
.mode-toggle {
  display: flex;
  align-items: center;
  border: 1px solid rgba(255,255,255,0.18);
  height: 26px;
  overflow: hidden;
}

.mode-toggle-sep {
  width: 1px;
  height: 100%;
  background: rgba(255,255,255,0.18);
  flex-shrink: 0;
}

.mode-btn {
  padding: 0 12px;
  height: 26px;
  border: none;
  background: transparent;
  color: rgba(255,255,255,0.45);
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  font-weight: 600;
  cursor: pointer;
  position: relative;
  -webkit-app-region: no-drag;
  transition: color var(--transition-fast);
}

.mode-btn:hover {
  color: rgba(255,255,255,0.75);
}

.mode-btn--active {
  color: #fff;
}

.mode-btn--active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 4px;
  right: 4px;
  height: 2px;
  background: var(--color-accent-yellow);
}
```

- [ ] **Step 2: Add topbar-left vertical separator CSS**

Add after the `.brand` rule:

```css
.topbar-vsep {
  width: 1px;
  height: 20px;
  background: rgba(255,255,255,0.20);
  flex-shrink: 0;
}
```

- [ ] **Step 3: Update App.tsx — move toggle to left, add separator**

Replace the entire `<header className="topbar">` block in `App.tsx` with:

```tsx
<header className="topbar">
  {/* Left: brand + separator + mode toggle (when visible) + back */}
  <div className="topbar-left">
    <span className="brand">PNG ⌁ Compare</span>

    {showModeToggle && (
      <>
        <div className="topbar-vsep" />
        <div className="mode-toggle" role="group" aria-label="模式">
          <button
            type="button"
            className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
            onClick={() => wb.setMode('single')}
          >
            单文件
          </button>
          <div className="mode-toggle-sep" />
          <button
            type="button"
            className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
            onClick={() => wb.setMode('directory')}
          >
            目录
          </button>
        </div>
      </>
    )}

    {wb.view === 'pair-comparison' && wb.directoryContext && (
      <>
        <div className="topbar-vsep" />
        <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
          ← 返回目录
        </button>
      </>
    )}
  </div>

  {/* Center: filename or progress — pure drag region */}
  <div className="topbar-center">
    {!showModeToggle && wb.pairResult ? (
      <span className="topbar-filename">{wb.pairResult.left.file_name}</span>
    ) : !showModeToggle && progressLabel ? (
      <span className="topbar-progress">{progressLabel}</span>
    ) : null}
  </div>

  {/* Right: window controls */}
  <div className="topbar-right">
    <div className="win-controls">
      <button type="button" className="win-btn" onClick={() => void win.minimize()} aria-label="最小化">─</button>
      <button type="button" className="win-btn" onClick={() => void win.toggleMaximize()} aria-label="最大化">□</button>
      <button type="button" className="win-btn win-btn--close" onClick={() => void win.close()} aria-label="关闭">✕</button>
    </div>
  </div>
</header>
```

- [ ] **Step 4: Verify toggle appearance**

In the running app, confirm:
- Toggle sits left of center, next to the brand
- Active button shows white text + thin yellow underline at bottom edge
- Inactive button shows dim text
- Clicking switches correctly

- [ ] **Step 5: Commit**

```bash
git add frontend/src/styles/app.css frontend/src/App.tsx
git commit -m "feat: move mode toggle left with border-group and yellow-underline active state"
```

---

### Task 3: CSS — Toolbar Height & MotherDuck Refinements

**Files:**
- Modify: `frontend/src/styles/app.css` (`.toolbar`, `.path-group`, `.cta-btn`, `.path-group:focus-within`, `.cta-outer`, `.choose-btn`, `.cta-outer:hover .cta-btn`)

- [ ] **Step 1: Increase toolbar padding**

Find `.toolbar` rule and update `padding`:
```css
.toolbar {
  background: var(--color-surface);
  border-bottom: var(--border-strong);
  padding: 18px 16px;   /* was 14px 20px */
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-shrink: 0;
}
```

- [ ] **Step 2: Increase path-group padding and gap**

Find `.path-group` and update:
```css
.path-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;           /* was 5px */
  min-width: 0;
  padding: 13px 14px; /* was 10px 12px */
  border: 2px dashed #ccc;
  background: #fff;
  transition: border-color var(--transition-fast);
}
```

- [ ] **Step 3: Increase CTA button padding**

Find `.cta-btn` and update `padding`:
```css
.cta-btn {
  display: block;
  background: var(--color-accent-blue);
  color: var(--color-ink);
  border: 2px solid var(--color-ink);
  padding: 13px 22px;   /* was 9px 20px */
  font-family: var(--font-mono);
  font-size: 11px;
  text-transform: uppercase;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  letter-spacing: 1px;
  transition: transform var(--transition-fast);
}
```

- [ ] **Step 4: Add focus-within shadow to path-group**

Find `.path-group:focus-within` and add `box-shadow`:
```css
.path-group:focus-within {
  border-color: var(--color-ink);
  border-style: solid;
  box-shadow: var(--shadow-input-focus);   /* add this line */
}
```

- [ ] **Step 5: Update CTA outer border-radius and hover lift**

Find `.cta-outer`:
```css
.cta-outer {
  background: var(--color-ink);
  border-radius: 2px;   /* was 0 */
  padding: 2px;
}
```

Find `.cta-outer:hover .cta-btn`:
```css
.cta-outer:hover .cta-btn { transform: translate(7px, -7px); }  /* was 4px, -4px */
```

- [ ] **Step 6: Add border-radius to choose button**

Find `.choose-btn` and add:
```css
.choose-btn {
  border: var(--border-strong);
  border-radius: 2px;   /* add this */
  padding: 4px 12px;
  margin-top: 4px;
  font-size: 10px;
  text-transform: uppercase;
  background: var(--color-ink);
  color: #fff;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  letter-spacing: 1px;
  font-weight: 600;
  transition: background var(--transition-fast);
}
```

- [ ] **Step 7: Verify visually**

In the running app:
- Toolbar inputs should feel tall and substantial (~90px toolbar)
- Clicking into an input shows the blue directional shadow on the path-group
- CTA hover lifts more aggressively (7px)
- CTA outer and Choose buttons have a slight radius

- [ ] **Step 8: Commit**

```bash
git add frontend/src/styles/app.css
git commit -m "feat: increase toolbar height and apply MotherDuck theme refinements"
```

---

### Task 4: Chinese Text — PairComparison

**Files:**
- Modify: `frontend/src/components/PairComparison.tsx`

- [ ] **Step 1: Update labels, placeholders, and buttons in PairComparison**

Apply all Chinese text changes to `PairComparison.tsx`. Replace the JSX returned by `PairComparison` (the toolbar section only — lines 50–88):

```tsx
<div className="toolbar">
  <div className="path-group">
    <span className="path-label">{mode === 'directory' ? '左文件' : '左图'}</span>
    <div className="path-input-row">
      <input
        className="path-input"
        value={leftInput}
        onChange={(e) => onLeftInput(e.target.value)}
        placeholder={mode === 'directory' ? '左侧文件路径…' : '左侧 PNG 路径…'}
      />
      <button type="button" className="choose-btn" onClick={onPickLeft}>选择</button>
    </div>
  </div>
  <div className="cta-wrap">
    <div className="cta-outer">
      <button
        type="button"
        className="cta-btn"
        disabled={isLoading || !leftInput || !rightInput}
        onClick={onCompare}
      >
        {isLoading ? '对比中…' : '对比'}
      </button>
    </div>
  </div>
  <div className="path-group">
    <span className="path-label">{mode === 'directory' ? '右文件' : '右图'}</span>
    <div className="path-input-row">
      <input
        className="path-input"
        value={rightInput}
        onChange={(e) => onRightInput(e.target.value)}
        placeholder={mode === 'directory' ? '右侧文件路径…' : '右侧 PNG 路径…'}
      />
      <button type="button" className="choose-btn" onClick={onPickRight}>选择</button>
    </div>
  </div>
</div>
```

- [ ] **Step 2: Update EmptyState strings in SplitPanelContent**

Find all three `<EmptyState` usages inside `SplitPanelContent` and update:

```tsx
// When no pairResult yet:
<EmptyState
  title="选择文件并对比"
  body="选择左右两侧的 PNG 文件，点击对比后结果显示在此处。"
/>

// When no metadata:
<EmptyState title="无元数据" body="该文件不含嵌入式元数据。" />

// When no JSON:
<EmptyState title="无 JSON" body="未找到原始 JSON 数据。" />
```

- [ ] **Step 3: Verify**

In the running app, open the Single File view. Confirm labels say `左图` / `右图`, button says `选择`, CTA says `对比`. The empty state in the result panels says `选择文件并对比`.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/PairComparison.tsx
git commit -m "feat: Chinese-primary text in PairComparison"
```

---

### Task 5: Chinese Text — DirectoryOverview & FileCard

**Files:**
- Modify: `frontend/src/components/DirectoryOverview.tsx`
- Modify: `frontend/src/components/FileCard.tsx`

- [ ] **Step 1: Update FILTERS array in DirectoryOverview**

Find the `FILTERS` constant at the top of `DirectoryOverview.tsx` and replace:

```tsx
const FILTERS: { id: ActiveFilter; label: string }[] = [
  { id: 'all',        label: '全部' },
  { id: 'different',  label: '差异' },
  { id: 'identical',  label: '相同' },
  { id: 'left_only',  label: '仅左侧' },
  { id: 'right_only', label: '仅右侧' },
  { id: 'error',      label: '错误' },
];
```

- [ ] **Step 2: Update toolbar labels, placeholders, and CTA in DirectoryOverview**

Find the `return (` JSX block in `DirectoryOverview` and replace the toolbar section:

```tsx
<div className="toolbar">
  <div className="path-group">
    <span className="path-label">左目录</span>
    <div className="path-input-row">
      <input
        className="path-input"
        value={leftInput}
        onChange={(e) => onLeftInput(e.target.value)}
        placeholder="左侧目录路径…"
      />
      <button type="button" className="choose-btn" onClick={onPickLeft}>选择</button>
    </div>
  </div>
  <div className="cta-wrap">
    <div className="cta-outer">
      <button
        type="button"
        className="cta-btn"
        disabled={isLoading || !leftInput || !rightInput}
        onClick={onScan}
      >
        {isLoading ? '扫描中…' : '扫描'}
      </button>
    </div>
  </div>
  <div className="path-group">
    <span className="path-label">右目录</span>
    <div className="path-input-row">
      <input
        className="path-input"
        value={rightInput}
        onChange={(e) => onRightInput(e.target.value)}
        placeholder="右侧目录路径…"
      />
      <button type="button" className="choose-btn" onClick={onPickRight}>选择</button>
    </div>
  </div>
</div>
```

- [ ] **Step 3: Update EmptyState strings in DirectoryOverview**

Find the three `<EmptyState` usages in `DirectoryOverview.tsx` and replace:

```tsx
// isLoading:
<EmptyState title="扫描中…" body="正在查找并对比 PNG 文件，请稍候…" />

// no items after scan (directorySummary exists but filteredItems empty):
<EmptyState
  title="无结果"
  body="没有文件与当前筛选条件匹配。"
/>

// not yet scanned (directorySummary is null and not loading):
<EmptyState
  title="选择两个目录并扫描"
  body="选择左右目录路径，点击扫描后结果显示在此处。"
/>
```

To find which EmptyState is which, look at the condition logic around them in `DirectoryOverview.tsx` around lines 118–131.

- [ ] **Step 4: Update STATUS_LABEL in FileCard**

Find the `STATUS_LABEL` constant in `FileCard.tsx` and replace:

```tsx
const STATUS_LABEL: Record<BatchListItemKind, string> = {
  different:  '差异',
  identical:  '相同',
  left_only:  '仅左侧',
  right_only: '仅右侧',
  error:      '错误',
};
```

- [ ] **Step 5: Update cardMeta strings in FileCard**

Find the `cardMeta` function in `FileCard.tsx` and replace:

```tsx
function cardMeta(item: BatchListItem): string {
  if (item.kind === 'error')      return item.message ?? '解析失败';
  if (item.kind === 'identical')  return '无变更';
  if (item.kind === 'left_only')  return '右侧目录中不存在';
  if (item.kind === 'right_only') return '左侧目录中不存在';
  if (item.difference_count > 0)
    return `${item.difference_count} 处变更`;
  return '存在差异';
}
```

- [ ] **Step 6: Verify**

In the running app, switch to Directory mode. Confirm:
- Toolbar says `左目录` / `右目录` with `选择` buttons and `扫描` CTA
- Filter bar shows `全部 / 差异 / 相同 / 仅左侧 / 仅右侧 / 错误`
- FileCards show Chinese status in header and Chinese meta text in body
- Empty state before scanning says `选择两个目录并扫描`

- [ ] **Step 7: Commit**

```bash
git add frontend/src/components/DirectoryOverview.tsx frontend/src/components/FileCard.tsx
git commit -m "feat: Chinese-primary text in DirectoryOverview and FileCard"
```

---

## Verification Checklist

After all tasks:

- [ ] Frameless window can be dragged by clicking any empty topbar area
- [ ] Mode toggle is left of center, inside thin-border container, yellow underline on active
- [ ] Toolbar inputs are tall and comfortable (~90px total toolbar)
- [ ] Clicking into an input shows blue directional shadow on the path-group
- [ ] CTA hover lifts 7px, has 2px border-radius
- [ ] All visible UI text is Chinese (labels, buttons, filters, empty states, placeholders)
- [ ] Brand name `PNG ⌁ Compare` is unchanged
- [ ] TypeScript compiles with no errors: `npm run build` in `frontend/`
