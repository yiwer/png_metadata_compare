# UI Refresh Design Spec
Date: 2026-04-24

## Scope

Five targeted changes to the Tauri frontend:

1. Fix window drag (frameless window can't be moved by dragging the topbar)
2. Mode toggle — move left, redesign active state and grouping
3. Toolbar input height — increase to ~90px tall
4. MotherDuck light theme refinements
5. UI text — Chinese primary, English decorative

---

## 1. Window Drag Fix

**Problem:** `data-tauri-drag-region` attributes on topbar divs are unreliable on Windows with `decorations: false`.

**Fix:** Use CSS `-webkit-app-region` directly.

In `app.css`:
```css
.topbar {
  -webkit-app-region: drag;
}
/* All interactive children must opt out */
.topbar button,
.topbar input,
.mode-btn,
.back-btn,
.win-btn {
  -webkit-app-region: no-drag;
}
```

Remove `data-tauri-drag-region` attributes from `App.tsx` — they are no longer needed. The `.topbar` background itself becomes the drag handle; any empty topbar area (center, gaps) is draggable by default.

---

## 2. Mode Toggle

**Position:** Moves from `.topbar-center` to `.topbar-left`, immediately after the brand name.

**Separator:** A `1px` vertical rule (`rgba(255,255,255,0.20)`, height `20px`) divides brand from toggle group.

**Wrapper (`.mode-toggle`):**
```css
.mode-toggle {
  display: flex;
  align-items: center;
  border: 1px solid rgba(255,255,255,0.18);
  height: 26px;
  overflow: hidden;
}
```

**Inner separator** between buttons:
```css
.mode-toggle-sep {
  width: 1px;
  height: 100%;
  background: rgba(255,255,255,0.18);
  flex-shrink: 0;
}
```

**Button (`.mode-btn`):**
```css
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
.mode-btn:hover { color: rgba(255,255,255,0.75); }
```

**Active state (`.mode-btn--active`):**
```css
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

**`.topbar-center`** becomes a pure drag region. In pair-comparison view it shows `.topbar-filename` or `.topbar-progress` as before (centered text). When `showModeToggle` is true it is empty — the toggle is now on the left.

---

## 3. Toolbar Input Height

Increase padding throughout the toolbar so inputs feel substantial (~90px total toolbar height).

```css
.toolbar {
  padding: 18px 16px;   /* was 14px 20px */
}

.path-group {
  padding: 13px 14px;   /* was 10px 12px */
  gap: 6px;             /* was 5px (between label and input row) */
}

.cta-btn {
  padding: 13px 22px;   /* was 9px 20px */
}
```

No structural changes — same elements, just more vertical space.

---

## 4. MotherDuck Light Theme Refinements

### Input focus shadow
Replace border-style toggle with MotherDuck's directional focus shadow:
```css
.path-group:focus-within {
  border-color: var(--color-ink);
  border-style: solid;
  box-shadow: var(--shadow-input-focus);  /* -3px 3px 0 0 #6fc2ff */
}
```

### CTA hover lift
Increase translate to match MotherDuck primary CTA spec:
```css
.cta-outer:hover .cta-btn {
  transform: translate(7px, -7px);   /* was 4px -4px */
}
```

### CTA outer border-radius
```css
.cta-outer {
  border-radius: 2px;   /* was 0 */
}
```

### Choose button border-radius
```css
.choose-btn {
  border-radius: 2px;   /* was 0 */
}
```

### FileCard hover (DirectoryOverview)
Apply MotherDuck Report Card hover pattern to `.file-card`:
```css
.file-card {
  transition: transform 0.15s ease-out, box-shadow 0.15s ease-out, border-color 0.15s ease-out;
}
.file-card:hover {
  transform: translateY(-4px) scale(1.02);
  box-shadow: var(--shadow-lift);           /* -4px 4px 0 0 #383838 */
  border-color: var(--color-accent-blue);   /* #6fc2ff */
}
.file-card:active {
  transform: translateY(-2px) scale(1.01);
  box-shadow: var(--shadow-lift-sm);        /* -2px 2px 0 0 #383838 */
}
```

---

## 5. UI Text — Chinese Primary, English Decorative

All visible user-facing strings switch to Chinese as the primary label with English in smaller/muted decoration where space allows.

| Location | Before | After |
|---|---|---|
| Mode toggle btn 1 | `Single File` | `单文件` |
| Mode toggle btn 2 | `Directory` | `目录` |
| Toolbar label left (single) | `Left PNG` | `左图` |
| Toolbar label right (single) | `Right PNG` | `右图` |
| Toolbar label left (dir) | `Left File` | `左文件` |
| Toolbar label right (dir) | `Right File` | `右文件` |
| Toolbar label left dir-overview | `Left Directory` | `左目录` |
| Toolbar label right dir-overview | `Right Directory` | `右目录` |
| CTA compare | `Compare` | `对比` |
| CTA scan | `Scan` | `扫描` |
| CTA loading compare | `Comparing…` | `对比中…` |
| CTA loading scan | `Scanning…` | `扫描中…` |
| Choose button | `Choose` | `选择` |
| Back button | `← Directory` | `← 返回目录` |
| Filter: all | `All` | `全部` |
| Filter: different | `Different` | `差异` |
| Filter: identical | `Identical` | `相同` |
| Filter: left_only | `Left-only` | `仅左侧` |
| Filter: right_only | `Right-only` | `仅右侧` |
| Filter: error | `Error` | `错误` |
| Placeholder left | `Path to left PNG…` | `左侧 PNG 路径…` |
| Placeholder right | `Path to right PNG…` | `右侧 PNG 路径…` |
| Placeholder left dir | `Path to left folder…` | `左侧目录路径…` |
| Placeholder right dir | `Path to right folder…` | `右侧目录路径…` |
| Progress label | `{n} / {m} different` | `{n} / {m} 个差异` |
| Brand (topbar) | `PNG ⌁ Compare` | unchanged (brand) |

Empty state and error text in `EmptyState.tsx` and `StatusBanner.tsx` are also updated to Chinese. Exact strings TBD at implementation but the pattern is Chinese sentence + English keyword in parentheses where helpful (e.g. `选择两个文件夹后点击扫描 (SCAN)`).

---

## Files Changed

- `frontend/src/styles/app.css` — drag region, mode toggle, toolbar heights, theme refinements, FileCard hover
- `frontend/src/App.tsx` — remove `data-tauri-drag-region`, add `.mode-toggle-sep` element, move toggle to left
- `frontend/src/components/PairComparison.tsx` — Chinese text strings
- `frontend/src/components/DirectoryOverview.tsx` — Chinese text strings
- `frontend/src/components/EmptyState.tsx` — Chinese text
- `frontend/src/components/StatusBanner.tsx` — Chinese text (if any hardcoded strings)
- `frontend/src/components/FileCard.tsx` — verify hover CSS class exists

---

## Out of Scope

- Dark mode
- Font change (IBM Plex Mono stays; Aeonik Mono is a commercial font)
- Layout changes beyond toolbar height
- New features
