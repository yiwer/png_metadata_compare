# UI Full Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the PNG Metadata Compare desktop app with a two-page architecture (DirectoryOverview + PairComparison), fully aligned with the MotherDuck design system and MotherDuck interaction patterns.

**Architecture:** App.tsx switches between two full-page views driven by `useWorkbench` state: `DirectoryOverview` (card grid for directory mode) and `PairComparison` (split-first layout shared by single-file mode and directory drill-down). The three existing Tauri API commands (`compare_single`, `scan_directory`, `inspect_single`) are unchanged.

**Tech Stack:** React 18, TypeScript, Tauri v2, CSS custom properties (no CSS framework). Tests via Vitest + React Testing Library. Run tests with `cd frontend && npm test -- --run`.

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `frontend/src/styles/tokens.css` | Add missing design tokens |
| Rewrite | `frontend/src/styles/app.css` | New layout system, all component CSS |
| Rewrite | `frontend/src/features/workbench/useWorkbench.ts` | Two-page state model |
| Modify | `frontend/src/components/MetadataTree.tsx` | Add `diffPathMap` prop for inline diff highlighting |
| Create | `frontend/src/components/FileCard.tsx` | Directory overview card |
| Create | `frontend/src/components/DiffStrip.tsx` | Center diff summary column |
| Create | `frontend/src/components/ViewModeStrip.tsx` | Tree/JSON/Image segmented control |
| Create | `frontend/src/components/PairComparison.tsx` | Split-first comparison page |
| Create | `frontend/src/components/DirectoryOverview.tsx` | Card grid directory page |
| Rewrite | `frontend/src/App.tsx` | New TopBar + view routing |
| Delete | `frontend/src/components/TabBar.tsx` | Replaced by ViewModeStrip |
| Delete | `frontend/src/components/ResultRail.tsx` | Replaced by DirectoryOverview |
| Delete | `frontend/src/components/PreviewStrip.tsx` | Info moved to TopBar |
| Delete | `frontend/src/components/InspectorPanel.tsx` | Removed from new design |
| Update | `frontend/src/features/workbench/useWorkbench.test.tsx` | Match new state API |

---

## Task 1: Update Design Tokens

**Files:**
- Modify: `frontend/src/styles/tokens.css`

- [ ] **Step 1: Replace tokens.css content**

```css
/* frontend/src/styles/tokens.css */
:root {
  /* Backgrounds */
  --color-bg: #f4efea;
  --color-surface: #f8f8f7;
  --color-card: #ffffff;

  /* Text / Ink */
  --color-ink: #383838;
  --color-muted: #818181;
  --color-disabled: #a1a1a1;

  /* Accents */
  --color-accent-yellow: #ffde00;
  --color-accent-blue: #6fc2ff;
  --color-accent-red: #ff7169;
  --color-accent-green: #22c55e;

  /* Diff state backgrounds */
  --color-diff-mod-bg: #fffde7;
  --color-diff-add-bg: #e8f5e9;
  --color-diff-rem-bg: #ffebe9;

  /* Borders */
  --border-strong: 2px solid var(--color-ink);
  --border-light: 1px solid #f1f1f1;

  /* Shadows — MotherDuck directional (left-down offset) */
  --shadow-lift: var(--color-ink) -4px 4px 0px 0px;
  --shadow-lift-sm: var(--color-ink) -2px 2px 0px 0px;
  --shadow-input-focus: var(--color-accent-blue) -3px 3px 0px 0px;

  /* Spacing (4px base) */
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-5: 1.25rem;   /* 20px */
  --space-6: 1.5rem;    /* 24px */

  /* Typography */
  --font-mono: 'IBM Plex Mono', 'Cascadia Mono', 'Courier New', monospace;

  /* Transitions */
  --transition-fast: 0.12s ease-in-out;
  --transition-card: 0.15s ease-out;
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/styles/tokens.css
git commit -m "design: update tokens to MotherDuck spec"
```

---

## Task 2: Rewrite useWorkbench.ts

**Files:**
- Rewrite: `frontend/src/features/workbench/useWorkbench.ts`
- Update: `frontend/src/features/workbench/useWorkbench.test.tsx`

- [ ] **Step 1: Read the existing test to understand what to preserve**

Open `frontend/src/features/workbench/useWorkbench.test.tsx` and note which behaviors are tested.

- [ ] **Step 2: Write the new useWorkbench.ts**

```typescript
// frontend/src/features/workbench/useWorkbench.ts
import { useState } from 'react';
import { workbenchApi } from '../../lib/api';
import type {
  BatchListItem,
  BatchListItemKind,
  DirectorySummary,
  PairInspection,
  WorkbenchMode,
} from '../../lib/types';
import type { WorkbenchApi } from '../../lib/api';

export type AppView = 'directory-overview' | 'pair-comparison';
export type ViewMode = 'tree' | 'json' | 'image';
export type ActiveFilter = 'all' | BatchListItemKind;

export interface DirectoryContext {
  /** 1-based index of this item among items with kind === 'different' */
  index: number;
  /** Total number of items with kind === 'different' */
  totalDifferent: number;
}

type ModeInputs = Record<WorkbenchMode, { left: string; right: string }>;

function emptyInputs(): ModeInputs {
  return {
    single: { left: '', right: '' },
    directory: { left: '', right: '' },
  };
}

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function useWorkbench(api: WorkbenchApi = workbenchApi) {
  const [mode, setModeState] = useState<WorkbenchMode>('single');
  const [view, setView] = useState<AppView>('pair-comparison');
  const [inputsByMode, setInputsByMode] = useState<ModeInputs>(emptyInputs);
  const [directorySummary, setDirectorySummary] = useState<DirectorySummary | null>(null);
  const [activeFilter, setActiveFilter] = useState<ActiveFilter>('all');
  const [pairResult, setPairResult] = useState<PairInspection | null>(null);
  const [directoryContext, setDirectoryContext] = useState<DirectoryContext | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('tree');
  const [diffHighlight, setDiffHighlight] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const activeInputs = inputsByMode[mode];

  function setMode(nextMode: WorkbenchMode) {
    setModeState(nextMode);
    setView(nextMode === 'single' ? 'pair-comparison' : 'directory-overview');
    setDirectorySummary(null);
    setPairResult(null);
    setDirectoryContext(null);
    setActiveFilter('all');
    setError(null);
  }

  function setLeftInput(value: string) {
    setInputsByMode((cur) => ({ ...cur, [mode]: { ...cur[mode], left: value } }));
  }

  function setRightInput(value: string) {
    setInputsByMode((cur) => ({ ...cur, [mode]: { ...cur[mode], right: value } }));
  }

  function toggleDiffHighlight() {
    setDiffHighlight((v) => !v);
  }

  function goBackToDirectory() {
    setView('directory-overview');
    setPairResult(null);
    setDirectoryContext(null);
  }

  async function navigateToPair(item: BatchListItem) {
    if (!item.left_path || !item.right_path) return;
    setIsLoading(true);
    setError(null);

    const differentItems = (directorySummary?.items ?? []).filter((i) => i.kind === 'different');
    const diffIndex = differentItems.findIndex((i) => i.id === item.id);
    setDirectoryContext(
      diffIndex >= 0
        ? { index: diffIndex + 1, totalDifferent: differentItems.length }
        : null,
    );

    try {
      const result = await api.compareSingle(item.left_path, item.right_path);
      setPairResult(result);
      setView('pair-comparison');
      setViewMode('tree');
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function runCompare() {
    setIsLoading(true);
    setError(null);

    try {
      if (mode === 'single') {
        const result = await api.compareSingle(activeInputs.left, activeInputs.right);
        setPairResult(result);
        setDirectorySummary(null);
        setDirectoryContext(null);
        setView('pair-comparison');
        setViewMode('tree');
        return;
      }

      const summary = await api.scanDirectory(activeInputs.left, activeInputs.right);
      setDirectorySummary(summary);
      setPairResult(null);
      setDirectoryContext(null);
      setActiveFilter('all');
      setView('directory-overview');
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  const filteredItems =
    activeFilter === 'all'
      ? (directorySummary?.items ?? [])
      : (directorySummary?.items ?? []).filter((i) => i.kind === activeFilter);

  return {
    mode,
    view,
    leftInput: activeInputs.left,
    rightInput: activeInputs.right,
    directorySummary,
    filteredItems,
    activeFilter,
    pairResult,
    directoryContext,
    viewMode,
    diffHighlight,
    isLoading,
    error,
    setMode,
    setLeftInput,
    setRightInput,
    setActiveFilter,
    setViewMode,
    toggleDiffHighlight,
    goBackToDirectory,
    navigateToPair,
    runCompare,
  };
}
```

- [ ] **Step 3: Run tests to see what's broken**

```bash
cd frontend && npm test -- --run 2>&1 | head -60
```

Expected: some test failures about missing fields (`activeTab`, `selectResultItem`, etc.)

- [ ] **Step 4: Rewrite useWorkbench.test.tsx**

```typescript
// frontend/src/features/workbench/useWorkbench.test.tsx
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useWorkbench } from './useWorkbench';
import type { WorkbenchApi } from '../../lib/api';
import type { PairInspection, DirectorySummary } from '../../lib/types';

const mockInspection: PairInspection = {
  left: { side: 'left', file_path: '/a.png', file_name: 'a.png', raw_json: null, metadata: null, error: null },
  right: { side: 'right', file_path: '/b.png', file_name: 'b.png', raw_json: null, metadata: null, error: null },
  diff_root: { path: '', status: 'unchanged', left_value: null, right_value: null, summary: 'root', children: [] },
  diff_summary: { modified: 0, added: 0, removed: 0, reordered: 0, error: 0 },
  default_selected_path: null,
};

const mockSummary: DirectorySummary = {
  counts: { identical: 1, different: 2, left_only: 0, right_only: 0, error: 0 },
  items: [
    { id: '1', kind: 'different', label: 'a.png', left_path: '/l/a.png', right_path: '/r/a.png', difference_count: 1, match_strategy: 'file_name', message: null },
    { id: '2', kind: 'different', label: 'b.png', left_path: '/l/b.png', right_path: '/r/b.png', difference_count: 2, match_strategy: 'file_name', message: null },
    { id: '3', kind: 'identical', label: 'c.png', left_path: '/l/c.png', right_path: '/r/c.png', difference_count: 0, match_strategy: 'file_name', message: null },
  ],
};

function makeApi(overrides: Partial<WorkbenchApi> = {}): WorkbenchApi {
  return {
    compareSingle: vi.fn().mockResolvedValue(mockInspection),
    scanDirectory: vi.fn().mockResolvedValue(mockSummary),
    inspectSingle: vi.fn().mockResolvedValue({}),
    ...overrides,
  };
}

describe('useWorkbench', () => {
  it('starts in single mode, pair-comparison view', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    expect(result.current.mode).toBe('single');
    expect(result.current.view).toBe('pair-comparison');
  });

  it('switching mode resets state', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setMode('directory'); });
    expect(result.current.mode).toBe('directory');
    expect(result.current.view).toBe('directory-overview');
    expect(result.current.pairResult).toBeNull();
  });

  it('runCompare (single) calls compareSingle and sets pairResult', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runCompare(); });
    expect(api.compareSingle).toHaveBeenCalledWith('/a.png', '/b.png');
    expect(result.current.pairResult).toBe(mockInspection);
    expect(result.current.view).toBe('pair-comparison');
  });

  it('runCompare (directory) calls scanDirectory and sets view to directory-overview', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    expect(api.scanDirectory).toHaveBeenCalledWith('/left', '/right');
    expect(result.current.view).toBe('directory-overview');
    expect(result.current.directorySummary).toBe(mockSummary);
  });

  it('activeFilter filters items client-side', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    expect(result.current.filteredItems).toHaveLength(3);
    act(() => { result.current.setActiveFilter('different'); });
    expect(result.current.filteredItems).toHaveLength(2);
    act(() => { result.current.setActiveFilter('identical'); });
    expect(result.current.filteredItems).toHaveLength(1);
  });

  it('navigateToPair sets directoryContext index among different items', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => {
      await result.current.navigateToPair(mockSummary.items[1]); // second 'different' item
    });
    expect(result.current.directoryContext).toEqual({ index: 2, totalDifferent: 2 });
    expect(result.current.view).toBe('pair-comparison');
  });

  it('goBackToDirectory resets view and pairResult', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => { await result.current.navigateToPair(mockSummary.items[0]); });
    act(() => { result.current.goBackToDirectory(); });
    expect(result.current.view).toBe('directory-overview');
    expect(result.current.pairResult).toBeNull();
  });

  it('runCompare sets error on failure', async () => {
    const api = makeApi({ compareSingle: vi.fn().mockRejectedValue(new Error('parse failed')) });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runCompare(); });
    expect(result.current.error).toBe('parse failed');
  });

  it('toggleDiffHighlight flips diffHighlight', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    expect(result.current.diffHighlight).toBe(true);
    act(() => { result.current.toggleDiffHighlight(); });
    expect(result.current.diffHighlight).toBe(false);
  });
});
```

- [ ] **Step 5: Run tests**

```bash
cd frontend && npm test -- --run 2>&1 | tail -20
```

Expected: all tests in `useWorkbench.test.tsx` pass. Other test files may still fail (they depend on App.tsx).

- [ ] **Step 6: Commit**

```bash
git add frontend/src/features/workbench/useWorkbench.ts frontend/src/features/workbench/useWorkbench.test.tsx
git commit -m "feat: rewrite useWorkbench for two-page architecture"
```

---

## Task 3: Add diffPathMap to MetadataTree

**Files:**
- Modify: `frontend/src/components/MetadataTree.tsx`

The DiffNode tree uses paths like `workflow.version`. MetadataTree builds paths with the same convention (`prefix.key`). Adding an optional `diffPathMap` prop lets callers pass `Map<path, DiffStatus>` so nodes can be highlighted.

- [ ] **Step 1: Write the updated MetadataTree.tsx**

```typescript
// frontend/src/components/MetadataTree.tsx
import { useState } from 'react';
import type { DiffStatus, JsonValue } from '../lib/types';

export function MetadataTree({
  value,
  prefix = '',
  diffPathMap,
  highlight = false,
}: {
  value: JsonValue;
  prefix?: string;
  diffPathMap?: Map<string, DiffStatus>;
  highlight?: boolean;
}) {
  if (Array.isArray(value)) {
    return (
      <div className="meta-branch">
        {value.map((entry, index) => {
          const path = `${prefix}[${index}]`;
          return (
            <MetadataTree key={path} value={entry} prefix={path} diffPathMap={diffPathMap} highlight={highlight} />
          );
        })}
      </div>
    );
  }

  if (value && typeof value === 'object') {
    return (
      <div className="meta-branch">
        {Object.entries(value).map(([key, child]) => {
          const path = prefix ? `${prefix}.${key}` : key;
          return (
            <MetadataNode key={path} path={path} label={key} child={child} diffPathMap={diffPathMap} highlight={highlight} />
          );
        })}
      </div>
    );
  }

  const label = prefix || 'value';
  const status = diffPathMap?.get(label);
  return (
    <div className={nodeClass(status, highlight)}>
      <span className="node-key">{label}</span>
      <span className="node-val">{String(value)}</span>
      {highlight && status && status !== 'unchanged' && (
        <span className={`node-badge node-badge--${status}`}>{statusSymbol(status)}</span>
      )}
    </div>
  );
}

function MetadataNode({
  path,
  label,
  child,
  diffPathMap,
  highlight,
}: {
  path: string;
  label: string;
  child: JsonValue;
  diffPathMap?: Map<string, DiffStatus>;
  highlight: boolean;
}) {
  const [open, setOpen] = useState(true);
  const hasChildren = child !== null && typeof child === 'object';
  const status = diffPathMap?.get(path);

  return (
    <div className={`meta-node ${nodeClass(status, highlight)}`}>
      <button
        type="button"
        className="meta-row"
        onClick={() => { if (hasChildren) setOpen((v) => !v); }}
      >
        {hasChildren && <span className="meta-toggle">{open ? '▼' : '▶'}</span>}
        <span className="node-dot" data-status={highlight && status ? status : undefined} />
        <span className="node-key">{label}</span>
        {!hasChildren && <span className="node-val">{String(child)}</span>}
        {highlight && status && status !== 'unchanged' && (
          <span className={`node-badge node-badge--${status}`}>{statusSymbol(status)}</span>
        )}
      </button>
      {hasChildren && open && (
        <div className="meta-children">
          <MetadataTree value={child} prefix={path} diffPathMap={diffPathMap} highlight={highlight} />
        </div>
      )}
    </div>
  );
}

function nodeClass(status: DiffStatus | undefined, highlight: boolean): string {
  if (!highlight || !status || status === 'unchanged') return '';
  return `node--${status}`;
}

function statusSymbol(status: DiffStatus): string {
  switch (status) {
    case 'added': return '+';
    case 'removed': return '−';
    case 'modified': return '~';
    case 'reordered': return '⇄';
    default: return '!';
  }
}
```

- [ ] **Step 2: Run tests to ensure no regression**

```bash
cd frontend && npm test -- --run 2>&1 | tail -20
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/MetadataTree.tsx
git commit -m "feat: add diffPathMap highlight support to MetadataTree"
```

---

## Task 4: Create FileCard Component

**Files:**
- Create: `frontend/src/components/FileCard.tsx`

- [ ] **Step 1: Create FileCard.tsx**

```typescript
// frontend/src/components/FileCard.tsx
import type { BatchListItem, BatchListItemKind } from '../lib/types';

const STATUS_LABEL: Record<BatchListItemKind, string> = {
  different: 'Different',
  identical: 'Identical',
  left_only: 'Left Only',
  right_only: 'Right Only',
  error: 'Error',
};

export function FileCard({
  item,
  style,
  onClick,
}: {
  item: BatchListItem;
  style?: React.CSSProperties;
  onClick(): void;
}) {
  const label = STATUS_LABEL[item.kind];
  const metaText = cardMeta(item);

  return (
    <button type="button" className={`file-card file-card--${item.kind}`} style={style} onClick={onClick}>
      <div className="card-header">
        <span className={`status-dot status-dot--${item.kind}`} />
        {label}
      </div>
      <div className="card-body">
        <div className="card-name">{item.label}</div>
        <div className={`card-meta${item.kind === 'error' ? ' card-meta--error' : ''}`}>{metaText}</div>
      </div>
    </button>
  );
}

function cardMeta(item: BatchListItem): string {
  if (item.kind === 'error') return item.message ?? 'parse failed';
  if (item.kind === 'identical') return 'no changes';
  if (item.kind === 'left_only') return 'not in right dir';
  if (item.kind === 'right_only') return 'not in left dir';
  if (item.difference_count > 0) return `${item.difference_count} change${item.difference_count !== 1 ? 's' : ''}`;
  return 'different';
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/FileCard.tsx
git commit -m "feat: add FileCard component"
```

---

## Task 5: Create DiffStrip Component

**Files:**
- Create: `frontend/src/components/DiffStrip.tsx`

The DiffStrip is the 140px center column showing a summary of changed nodes from the DiffNode tree.

- [ ] **Step 1: Create DiffStrip.tsx**

```typescript
// frontend/src/components/DiffStrip.tsx
import type { DiffNode, DiffStatus } from '../lib/types';

function collectChanges(node: DiffNode, acc: DiffNode[] = []): DiffNode[] {
  if (node.status !== 'unchanged' && node.status !== 'error') {
    acc.push(node);
  }
  for (const child of node.children) {
    collectChanges(child, acc);
  }
  return acc;
}

const STATUS_SYMBOL: Record<DiffStatus, string> = {
  modified: '~',
  added: '+',
  removed: '−',
  reordered: '⇄',
  error: '!',
  unchanged: '',
};

export function DiffStrip({ root }: { root: DiffNode | null }) {
  if (!root) {
    return (
      <div className="diff-strip diff-strip--empty">
        <span className="diff-strip__label">Diff</span>
      </div>
    );
  }

  const changes = collectChanges(root);

  if (changes.length === 0) {
    return (
      <div className="diff-strip diff-strip--none">
        <span className="diff-strip__label">Diff</span>
        <span className="diff-strip__no-changes">No changes</span>
      </div>
    );
  }

  return (
    <div className="diff-strip">
      <span className="diff-strip__label">Changes ({changes.length})</span>
      <div className="diff-strip__list">
        {changes.map((node) => (
          <div key={node.path} className={`diff-row diff-row--${node.status}`}>
            <span className="diff-row__symbol">{STATUS_SYMBOL[node.status]}</span>
            <span className="diff-row__path">{shortPath(node.path)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function shortPath(path: string): string {
  const parts = path.split('.');
  return parts[parts.length - 1] ?? path;
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/DiffStrip.tsx
git commit -m "feat: add DiffStrip component"
```

---

## Task 6: Create ViewModeStrip Component

**Files:**
- Create: `frontend/src/components/ViewModeStrip.tsx`

- [ ] **Step 1: Create ViewModeStrip.tsx**

```typescript
// frontend/src/components/ViewModeStrip.tsx
import type { ViewMode } from '../features/workbench/useWorkbench';

const MODES: { id: ViewMode; label: string }[] = [
  { id: 'tree', label: 'Tree' },
  { id: 'json', label: 'JSON' },
  { id: 'image', label: 'Image' },
];

export function ViewModeStrip({
  viewMode,
  diffHighlight,
  changeCount,
  onViewMode,
  onToggleDiff,
}: {
  viewMode: ViewMode;
  diffHighlight: boolean;
  changeCount: number;
  onViewMode(mode: ViewMode): void;
  onToggleDiff(): void;
}) {
  return (
    <div className="view-strip">
      <span className="view-strip__label">View</span>
      <div className="seg-group" role="group" aria-label="View mode">
        {MODES.map((m) => (
          <button
            key={m.id}
            type="button"
            className={`seg${viewMode === m.id ? ' seg--active' : ''}`}
            aria-pressed={viewMode === m.id}
            onClick={() => onViewMode(m.id)}
          >
            {m.label}
          </button>
        ))}
      </div>
      <div className="view-strip__right">
        <span className="view-strip__diff-label">Highlight Diffs</span>
        <button
          type="button"
          className={`diff-toggle${diffHighlight ? ' diff-toggle--on' : ''}`}
          aria-label={diffHighlight ? 'Disable diff highlight' : 'Enable diff highlight'}
          onClick={onToggleDiff}
        >
          <span className="diff-toggle__knob" />
        </button>
        <span className={`change-badge${changeCount === 0 ? ' change-badge--zero' : ''}`}>
          {changeCount} {changeCount === 1 ? 'change' : 'changes'}
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/ViewModeStrip.tsx
git commit -m "feat: add ViewModeStrip component"
```

---

## Task 7: Create PairComparison Page

**Files:**
- Create: `frontend/src/components/PairComparison.tsx`

This page renders the split-first comparison view: left panel | diff strip | right panel, with the ViewModeStrip above.

- [ ] **Step 1: Create a diff path helper in a new lib file**

```typescript
// frontend/src/lib/diffUtils.ts
import type { DiffNode, DiffStatus } from './types';

export function buildDiffPathMap(node: DiffNode, map = new Map<string, DiffStatus>()): Map<string, DiffStatus> {
  if (node.status !== 'unchanged') {
    map.set(node.path, node.status);
  }
  for (const child of node.children) {
    buildDiffPathMap(child, map);
  }
  return map;
}

export function totalDiffCount(summary: { modified: number; added: number; removed: number; reordered: number; error: number }): number {
  return summary.modified + summary.added + summary.removed + summary.reordered + summary.error;
}
```

- [ ] **Step 2: Create PairComparison.tsx**

```typescript
// frontend/src/components/PairComparison.tsx
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { DiffStrip } from './DiffStrip';
import { EmptyState } from './EmptyState';
import { MetadataTree } from './MetadataTree';
import { ViewModeStrip } from './ViewModeStrip';
import { buildDiffPathMap, totalDiffCount } from '../lib/diffUtils';
import type { PairInspection } from '../lib/types';
import type { ViewMode } from '../features/workbench/useWorkbench';

export function PairComparison({
  mode,
  leftInput,
  rightInput,
  pairResult,
  viewMode,
  diffHighlight,
  isLoading,
  error,
  onLeftInput,
  onRightInput,
  onCompare,
  onPickLeft,
  onPickRight,
  onViewMode,
  onToggleDiff,
}: {
  mode: 'single' | 'directory';
  leftInput: string;
  rightInput: string;
  pairResult: PairInspection | null;
  viewMode: ViewMode;
  diffHighlight: boolean;
  isLoading: boolean;
  error: string | null;
  onLeftInput(v: string): void;
  onRightInput(v: string): void;
  onCompare(): void;
  onPickLeft(): void;
  onPickRight(): void;
  onViewMode(mode: ViewMode): void;
  onToggleDiff(): void;
}) {
  const diffPathMap = pairResult ? buildDiffPathMap(pairResult.diff_root) : undefined;
  const changeCount = pairResult ? totalDiffCount(pairResult.diff_summary) : 0;
  const leftLabel = pairResult?.left.file_name ?? 'Left';
  const rightLabel = pairResult?.right.file_name ?? 'Right';

  return (
    <>
      <div className="toolbar">
        <div className="path-group">
          <span className="path-label">{mode === 'directory' ? 'Left File' : 'Left PNG'}</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={leftInput}
              onChange={(e) => onLeftInput(e.target.value)}
              placeholder="Path to left PNG…"
            />
            <button type="button" className="choose-btn" onClick={onPickLeft}>Choose</button>
          </div>
        </div>
        <span className="vs-divider">vs</span>
        <div className="path-group">
          <span className="path-label">{mode === 'directory' ? 'Right File' : 'Right PNG'}</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={rightInput}
              onChange={(e) => onRightInput(e.target.value)}
              placeholder="Path to right PNG…"
            />
            <button type="button" className="choose-btn" onClick={onPickRight}>Choose</button>
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
              {isLoading ? 'Comparing…' : 'Compare'}
            </button>
          </div>
        </div>
      </div>

      {error && (
        <div className="status-banner status-banner--error">{error}</div>
      )}

      <ViewModeStrip
        viewMode={viewMode}
        diffHighlight={diffHighlight}
        changeCount={changeCount}
        onViewMode={onViewMode}
        onToggleDiff={onToggleDiff}
      />

      <div className="split-body">
        <div className="split-panel split-panel--left">
          <div className="panel-header">{leftLabel}</div>
          <SplitPanelContent
            side="left"
            pairResult={pairResult}
            viewMode={viewMode}
            diffPathMap={diffHighlight ? diffPathMap : undefined}
          />
        </div>

        <DiffStrip root={pairResult?.diff_root ?? null} />

        <div className="split-panel split-panel--right">
          <div className="panel-header">{rightLabel}</div>
          <SplitPanelContent
            side="right"
            pairResult={pairResult}
            viewMode={viewMode}
            diffPathMap={diffHighlight ? diffPathMap : undefined}
          />
        </div>
      </div>
    </>
  );
}

function SplitPanelContent({
  side,
  pairResult,
  viewMode,
  diffPathMap,
}: {
  side: 'left' | 'right';
  pairResult: PairInspection | null;
  viewMode: ViewMode;
  diffPathMap?: Map<string, import('../lib/types').DiffStatus>;
}) {
  if (!pairResult) {
    return (
      <EmptyState
        title="Choose inputs and compare"
        body="Results appear here after you run a comparison."
      />
    );
  }

  const sideData = side === 'left' ? pairResult.left : pairResult.right;

  if (viewMode === 'tree') {
    if (!sideData.metadata) {
      return <EmptyState title="No metadata" body="This file has no embedded metadata." />;
    }
    return (
      <MetadataTree
        value={sideData.metadata}
        diffPathMap={diffPathMap}
        highlight={!!diffPathMap}
      />
    );
  }

  if (viewMode === 'json') {
    const raw = sideData.raw_json;
    if (!raw) return <EmptyState title="No JSON" body="No raw JSON payload found." />;
    return <pre className="json-block">{formatJson(raw, diffPathMap, diffPathMap !== undefined)}</pre>;
  }

  if (viewMode === 'image') {
    const filePath = sideData.file_path;
    return (
      <div className="image-panel">
        <div className="image-frame">
          <img
            src={`asset://localhost/${encodeURIComponent(filePath.replace(/\\/g, '/'))}`}
            alt={sideData.file_name}
            className="image-preview"
            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
          />
        </div>
        <button
          type="button"
          className="open-btn"
          onClick={() => void openPath(filePath)}
        >
          Open Original ↗
        </button>
      </div>
    );
  }

  return null;
}

function formatJson(raw: string, _diffPathMap?: Map<string, import('../lib/types').DiffStatus>, _highlight?: boolean): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/diffUtils.ts frontend/src/components/PairComparison.tsx
git commit -m "feat: add PairComparison page and diffUtils"
```

---

## Task 8: Create DirectoryOverview Page

**Files:**
- Create: `frontend/src/components/DirectoryOverview.tsx`

- [ ] **Step 1: Create DirectoryOverview.tsx**

```typescript
// frontend/src/components/DirectoryOverview.tsx
import { FileCard } from './FileCard';
import { EmptyState } from './EmptyState';
import type { ActiveFilter } from '../features/workbench/useWorkbench';
import type { BatchListItem, BatchListItemKind, DirectorySummary } from '../lib/types';

const FILTERS: { id: ActiveFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'different', label: 'Different' },
  { id: 'identical', label: 'Identical' },
  { id: 'left_only', label: 'Left-only' },
  { id: 'right_only', label: 'Right-only' },
  { id: 'error', label: 'Error' },
];

export function DirectoryOverview({
  leftInput,
  rightInput,
  directorySummary,
  filteredItems,
  activeFilter,
  isLoading,
  error,
  onLeftInput,
  onRightInput,
  onScan,
  onPickLeft,
  onPickRight,
  onFilter,
  onSelectItem,
}: {
  leftInput: string;
  rightInput: string;
  directorySummary: DirectorySummary | null;
  filteredItems: BatchListItem[];
  activeFilter: ActiveFilter;
  isLoading: boolean;
  error: string | null;
  onLeftInput(v: string): void;
  onRightInput(v: string): void;
  onScan(): void;
  onPickLeft(): void;
  onPickRight(): void;
  onFilter(f: ActiveFilter): void;
  onSelectItem(item: BatchListItem): void;
}) {
  const counts = directorySummary?.counts;

  return (
    <>
      <div className="toolbar">
        <div className="path-group">
          <span className="path-label">Left Directory</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={leftInput}
              onChange={(e) => onLeftInput(e.target.value)}
              placeholder="Path to left folder…"
            />
            <button type="button" className="choose-btn" onClick={onPickLeft}>Choose</button>
          </div>
        </div>
        <span className="vs-divider">vs</span>
        <div className="path-group">
          <span className="path-label">Right Directory</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={rightInput}
              onChange={(e) => onRightInput(e.target.value)}
              placeholder="Path to right folder…"
            />
            <button type="button" className="choose-btn" onClick={onPickRight}>Choose</button>
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
              {isLoading ? 'Scanning…' : 'Scan'}
            </button>
          </div>
        </div>
      </div>

      {error && <div className="status-banner status-banner--error">{error}</div>}

      {counts && (
        <div className="stats-bar">
          {counts.different > 0 && <StatChip kind="different" count={counts.different} />}
          {counts.identical > 0 && <StatChip kind="identical" count={counts.identical} />}
          {counts.left_only > 0 && <StatChip kind="left_only" count={counts.left_only} />}
          {counts.right_only > 0 && <StatChip kind="right_only" count={counts.right_only} />}
          {counts.error > 0 && <StatChip kind="error" count={counts.error} />}
        </div>
      )}

      {directorySummary && (
        <div className="filter-bar">
          {FILTERS.map((f) => (
            <button
              key={f.id}
              type="button"
              className={`filter-btn${activeFilter === f.id ? ' filter-btn--active' : ''}`}
              onClick={() => onFilter(f.id)}
            >
              {f.label}
            </button>
          ))}
          <span className="filter-count">{filteredItems.length} files</span>
        </div>
      )}

      <div className="card-grid">
        {filteredItems.length === 0 && directorySummary && (
          <EmptyState title="No results" body="No files match the selected filter." />
        )}
        {!directorySummary && !isLoading && (
          <EmptyState
            title="Choose directories and scan"
            body="All PNG files found in both directories will be compared and shown here."
          />
        )}
        {filteredItems.map((item, index) => (
          <FileCard
            key={item.id}
            item={item}
            style={{ animationDelay: `${Math.min(index * 30, 300)}ms` }}
            onClick={() => {
              if (item.left_path && item.right_path) {
                onSelectItem(item);
              }
            }}
          />
        ))}
      </div>
    </>
  );
}

const KIND_LABEL: Record<BatchListItemKind, string> = {
  different: 'different',
  identical: 'identical',
  left_only: 'left-only',
  right_only: 'right-only',
  error: 'error',
};

function StatChip({ kind, count }: { kind: BatchListItemKind; count: number }) {
  return (
    <div className="stat-chip">
      <span className={`status-dot status-dot--${kind}`} />
      {count} {KIND_LABEL[kind]}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/DirectoryOverview.tsx frontend/src/components/FileCard.tsx
git commit -m "feat: add DirectoryOverview page with card grid"
```

---

## Task 9: Rewrite App.tsx

**Files:**
- Rewrite: `frontend/src/App.tsx`
- Delete: `frontend/src/components/TabBar.tsx`, `ResultRail.tsx`, `PreviewStrip.tsx`, `InspectorPanel.tsx`

- [ ] **Step 1: Rewrite App.tsx**

```typescript
// frontend/src/App.tsx
import { open } from '@tauri-apps/plugin-dialog';
import { DirectoryOverview } from './components/DirectoryOverview';
import { PairComparison } from './components/PairComparison';
import { useWorkbench } from './features/workbench/useWorkbench';

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

  const progressLabel =
    wb.directoryContext
      ? `${wb.directoryContext.index} / ${wb.directoryContext.totalDifferent} different`
      : null;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="topbar-left">
          {wb.view === 'pair-comparison' && wb.directoryContext && (
            <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
              ← Directory
            </button>
          )}
          <span className="brand">PNG ⌁ Compare</span>
        </div>

        <div className="topbar-center">
          {wb.view === 'pair-comparison' && wb.pairResult && (
            <span className="topbar-filename">{wb.pairResult.left.file_name}</span>
          )}
        </div>

        <div className="topbar-right">
          {wb.view === 'directory-overview' || wb.directoryContext === null ? (
            <div className="mode-toggle" role="group" aria-label="Mode">
              <button
                type="button"
                className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
                onClick={() => wb.setMode('single')}
              >
                Single File
              </button>
              <button
                type="button"
                className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
                onClick={() => wb.setMode('directory')}
              >
                Directory
              </button>
            </div>
          ) : (
            progressLabel && <span className="topbar-progress">{progressLabel}</span>
          )}
        </div>
      </header>

      {wb.view === 'directory-overview' ? (
        <DirectoryOverview
          leftInput={wb.leftInput}
          rightInput={wb.rightInput}
          directorySummary={wb.directorySummary}
          filteredItems={wb.filteredItems}
          activeFilter={wb.activeFilter}
          isLoading={wb.isLoading}
          error={wb.error}
          onLeftInput={wb.setLeftInput}
          onRightInput={wb.setRightInput}
          onScan={() => { void wb.runCompare(); }}
          onPickLeft={() => { void handlePickLeft(); }}
          onPickRight={() => { void handlePickRight(); }}
          onFilter={wb.setActiveFilter}
          onSelectItem={(item) => { void wb.navigateToPair(item); }}
        />
      ) : (
        <PairComparison
          mode={wb.mode}
          leftInput={wb.leftInput}
          rightInput={wb.rightInput}
          pairResult={wb.pairResult}
          viewMode={wb.viewMode}
          diffHighlight={wb.diffHighlight}
          isLoading={wb.isLoading}
          error={wb.error}
          onLeftInput={wb.setLeftInput}
          onRightInput={wb.setRightInput}
          onCompare={() => { void wb.runCompare(); }}
          onPickLeft={() => { void handlePickLeft(); }}
          onPickRight={() => { void handlePickRight(); }}
          onViewMode={wb.setViewMode}
          onToggleDiff={wb.toggleDiffHighlight}
        />
      )}
    </div>
  );
}
```

- [ ] **Step 2: Delete obsolete components**

```bash
cd frontend && rm src/components/TabBar.tsx src/components/ResultRail.tsx src/components/PreviewStrip.tsx src/components/InspectorPanel.tsx
```

- [ ] **Step 3: Run tests**

```bash
cd frontend && npm test -- --run 2>&1 | tail -30
```

Expected: App.test.tsx may have failures if it tested old components. Fix any import errors.

- [ ] **Step 4: Update App.test.tsx if needed**

If App.test.tsx imports deleted components, replace the test with a smoke test:

```typescript
// frontend/src/App.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import App from './App';

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

  it('renders mode toggle buttons', () => {
    render(<App />);
    expect(screen.getByText('Single File')).toBeTruthy();
    expect(screen.getByText('Directory')).toBeTruthy();
  });
});
```

- [ ] **Step 5: Run tests — all must pass**

```bash
cd frontend && npm test -- --run 2>&1 | tail -30
```

Expected: all tests pass (green).

- [ ] **Step 6: Commit**

```bash
git add frontend/src/App.tsx frontend/src/App.test.tsx
git commit -m "feat: rewrite App.tsx with two-page routing"
```

---

## Task 10: Rewrite app.css

**Files:**
- Rewrite: `frontend/src/styles/app.css`

- [ ] **Step 1: Rewrite app.css**

```css
/* frontend/src/styles/app.css */
* { box-sizing: border-box; margin: 0; padding: 0 }

html, body, #root {
  height: 100%;
}

body {
  font-family: var(--font-mono);
  background: var(--color-bg);
  color: var(--color-ink);
  font-size: 13px;
}

button, input {
  font: inherit;
  color: inherit;
}

/* ── APP SHELL ── */
.app-shell {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

/* ── TOPBAR ── */
.topbar {
  background: var(--color-ink);
  color: #fff;
  padding: 10px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.topbar-center {
  flex: 1;
  text-align: center;
}

.topbar-right {
  display: flex;
  align-items: center;
}

.brand {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 2px;
  color: #fff;
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
}

.back-btn:hover { text-decoration: underline; }

.topbar-filename {
  font-size: 11px;
  color: var(--color-accent-yellow);
  letter-spacing: 1px;
  text-transform: uppercase;
}

.topbar-progress {
  font-size: 10px;
  color: #818181;
  letter-spacing: 0.5px;
}

/* Mode toggle in topbar */
.mode-toggle { display: flex; }

.mode-btn {
  padding: 5px 14px;
  border: 2px solid #fff;
  background: transparent;
  color: #fff;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  cursor: pointer;
  letter-spacing: 0.5px;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.mode-btn:first-child { border-right-width: 0; }

.mode-btn--active {
  background: var(--color-accent-yellow);
  color: var(--color-ink);
  border-color: var(--color-accent-yellow);
}

/* ── TOOLBAR ── */
.toolbar {
  background: var(--color-surface);
  border-bottom: var(--border-strong);
  padding: 10px 20px;
  display: flex;
  gap: 8px;
  align-items: flex-end;
  flex-shrink: 0;
}

.path-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.path-label {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--color-muted);
}

.path-input-row {
  display: flex;
}

.path-input {
  flex: 1;
  border: var(--border-strong);
  border-right-width: 0;
  padding: 7px 10px;
  background: #fff;
  min-width: 0;
  font-size: 11px;
  transition: box-shadow 0.15s;
}

.path-input:focus {
  outline: none;
  box-shadow: var(--shadow-input-focus);
}

.choose-btn {
  border: var(--border-strong);
  padding: 7px 12px;
  font-size: 10px;
  text-transform: uppercase;
  background: var(--color-bg);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  letter-spacing: 0.5px;
}

.choose-btn:hover { background: var(--color-surface); }

.vs-divider {
  font-size: 11px;
  color: var(--color-muted);
  flex-shrink: 0;
  padding-bottom: 9px;
}

/* CTA double-layer button */
.cta-wrap { display: flex; align-items: flex-end; }

.cta-outer {
  background: var(--color-ink);
  border-radius: 2px;
  padding: 2px;
}

.cta-btn {
  display: block;
  background: var(--color-accent-blue);
  color: var(--color-ink);
  border: 2px solid var(--color-ink);
  padding: 9px 20px;
  font-family: var(--font-mono);
  font-size: 11px;
  text-transform: uppercase;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  letter-spacing: 1px;
  transition: transform var(--transition-fast);
}

.cta-outer:hover .cta-btn { transform: translate(4px, -4px); }
.cta-outer:active .cta-btn { transform: none; }

.cta-btn:disabled {
  background: var(--color-surface);
  color: var(--color-disabled);
  cursor: not-allowed;
  border-color: var(--color-disabled);
}
.cta-outer:hover .cta-btn:disabled { transform: none; }

/* ── STATUS BANNER ── */
.status-banner {
  padding: 8px 20px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  flex-shrink: 0;
}

.status-banner--error {
  background: var(--color-diff-rem-bg);
  border-bottom: 2px solid var(--color-accent-red);
  color: var(--color-accent-red);
}

/* ── STATUS DOT ── */
.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  border: 1.5px solid var(--color-ink);
  flex-shrink: 0;
  display: inline-block;
}

.status-dot--different  { background: var(--color-accent-yellow); }
.status-dot--identical  { background: var(--color-accent-green); }
.status-dot--error      { background: var(--color-accent-red); }
.status-dot--left_only,
.status-dot--right_only { background: var(--color-accent-blue); }

/* ── STATS BAR ── */
.stats-bar {
  background: var(--color-surface);
  border-bottom: var(--border-strong);
  padding: 7px 20px;
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  flex-shrink: 0;
}

.stat-chip {
  border: var(--border-strong);
  background: var(--color-card);
  padding: 3px 12px;
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 5px;
}

/* ── FILTER BAR ── */
.filter-bar {
  background: var(--color-surface);
  border-bottom: var(--border-strong);
  padding: 6px 20px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.filter-btn {
  border: var(--border-strong);
  border-right-width: 0;
  padding: 4px 14px;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  background: var(--color-card);
  cursor: pointer;
  letter-spacing: 0.5px;
}

.filter-btn:last-of-type { border-right-width: 2px; }
.filter-btn:hover { background: var(--color-surface); }

.filter-btn--active {
  background: var(--color-ink);
  color: #fff;
}

.filter-count {
  margin-left: auto;
  font-size: 10px;
  color: var(--color-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* ── CARD GRID ── */
.card-grid {
  padding: 16px 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
  align-content: start;
  flex: 1;
  overflow-y: auto;
}

/* ── FILE CARD ── */
.file-card {
  border: var(--border-strong);
  background: var(--color-card);
  cursor: pointer;
  text-align: left;
  padding: 0;
  transition:
    transform var(--transition-card),
    box-shadow var(--transition-card),
    border-color var(--transition-card);
  animation: cardEnter 0.3s ease-out both;
}

.file-card:hover {
  transform: translateY(-4px) scale(1.01);
  box-shadow: var(--shadow-lift);
  border-color: var(--color-accent-blue);
}

.file-card:active {
  transform: translateY(-2px) scale(1.005);
  box-shadow: var(--shadow-lift-sm);
}

@keyframes cardEnter {
  from { opacity: 0; transform: translateY(8px); }
  to   { opacity: 1; transform: translateY(0); }
}

.card-header {
  border-bottom: var(--border-strong);
  padding: 7px 10px;
  font-size: 10px;
  text-transform: uppercase;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  letter-spacing: 0.5px;
}

.file-card--different  .card-header { background: var(--color-accent-yellow); }
.file-card--identical  .card-header { background: var(--color-diff-add-bg); }
.file-card--error      .card-header { background: var(--color-diff-rem-bg); }
.file-card--left_only  .card-header,
.file-card--right_only .card-header { background: #ebf9ff; }

.card-body { padding: 8px 10px; }

.card-name {
  font-size: 11px;
  font-weight: 600;
  word-break: break-all;
  margin-bottom: 3px;
}

.card-meta {
  font-size: 10px;
  color: var(--color-muted);
}

.card-meta--error { color: var(--color-accent-red); }

/* ── VIEW STRIP ── */
.view-strip {
  background: var(--color-surface);
  border-bottom: var(--border-strong);
  padding: 7px 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
}

.view-strip__label {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--color-muted);
}

.seg-group { display: flex; }

.seg {
  border: var(--border-strong);
  border-right-width: 0;
  padding: 4px 14px;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  background: var(--color-card);
  cursor: pointer;
  letter-spacing: 0.5px;
}

.seg:last-child { border-right-width: 2px; }
.seg:hover { background: var(--color-surface); }
.seg--active { background: var(--color-accent-blue); font-weight: 600; }

.view-strip__right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}

.view-strip__diff-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.diff-toggle {
  width: 32px;
  height: 16px;
  background: #f1f1f1;
  border: 1.5px solid var(--color-ink);
  border-radius: 8px;
  position: relative;
  cursor: pointer;
  padding: 0;
  transition: background 0.15s;
}

.diff-toggle--on { background: var(--color-accent-yellow); }

.diff-toggle__knob {
  width: 11px;
  height: 11px;
  background: var(--color-ink);
  border-radius: 50%;
  position: absolute;
  top: 1.5px;
  left: 1px;
  transition: left 0.15s;
}

.diff-toggle--on .diff-toggle__knob { left: 17px; }

.change-badge {
  background: var(--color-accent-yellow);
  border: 1.5px solid var(--color-ink);
  padding: 2px 10px;
  font-size: 10px;
}

.change-badge--zero {
  background: #f1f1f1;
}

/* ── SPLIT BODY ── */
.split-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.split-panel {
  flex: 1;
  padding: 12px 14px;
  background: var(--color-card);
  overflow-y: auto;
  min-width: 0;
}

.split-panel--left {
  border-right: var(--border-strong);
}

.panel-header {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--color-muted);
  border-bottom: 1px solid #f1f1f1;
  margin-bottom: 8px;
  padding-bottom: 5px;
}

/* ── DIFF STRIP ── */
.diff-strip {
  width: 140px;
  flex-shrink: 0;
  border-left: var(--border-strong);
  border-right: var(--border-strong);
  background: var(--color-diff-mod-bg);
  padding: 10px 8px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diff-strip--empty,
.diff-strip--none {
  align-items: center;
  justify-content: center;
}

.diff-strip__label {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--color-muted);
  margin-bottom: 6px;
  flex-shrink: 0;
}

.diff-strip__no-changes {
  font-size: 10px;
  color: var(--color-muted);
  text-transform: uppercase;
}

.diff-strip__list {
  display: flex;
  flex-direction: column;
  gap: 3px;
  overflow-y: auto;
}

.diff-row {
  font-size: 10px;
  padding: 3px 4px;
  display: flex;
  align-items: center;
  gap: 4px;
  border-left: 2px solid transparent;
}

.diff-row--modified  { background: var(--color-diff-mod-bg); border-left-color: #e1c427; }
.diff-row--added     { background: var(--color-diff-add-bg); border-left-color: var(--color-accent-green); }
.diff-row--removed   { background: var(--color-diff-rem-bg); border-left-color: var(--color-accent-red); }
.diff-row--reordered { background: #ebf9ff; border-left-color: var(--color-accent-blue); }

.diff-row__symbol { font-size: 11px; font-weight: 600; width: 10px; flex-shrink: 0; }
.diff-row__path   { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* ── METADATA TREE ── */
.meta-branch { display: flex; flex-direction: column; gap: 1px; }

.meta-node { }

.meta-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 3px 4px;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  font-size: 10px;
  border-radius: 0;
  transition: background 0.1s;
}

.meta-row:hover { background: var(--color-bg); }

.meta-toggle { font-size: 8px; width: 10px; flex-shrink: 0; }

.node-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-muted);
  flex-shrink: 0;
}

.node-dot[data-status="modified"]  { background: #e1c427; }
.node-dot[data-status="added"]     { background: var(--color-accent-green); }
.node-dot[data-status="removed"]   { background: var(--color-accent-red); }
.node-dot[data-status="reordered"] { background: var(--color-accent-blue); }

.node-key { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.node-val { color: var(--color-muted); font-size: 9px; white-space: nowrap; }

.node-badge {
  font-size: 9px;
  font-weight: 700;
  width: 14px;
  text-align: center;
  flex-shrink: 0;
}

.node-badge--modified  { color: #e1c427; }
.node-badge--added     { color: var(--color-accent-green); }
.node-badge--removed   { color: var(--color-accent-red); }
.node-badge--reordered { color: var(--color-accent-blue); }

.meta-children { margin-left: 14px; }

/* Diff highlight backgrounds */
.node--modified  { background: var(--color-diff-mod-bg); }
.node--added     { background: var(--color-diff-add-bg); }
.node--removed   { background: var(--color-diff-rem-bg); }

/* ── JSON BLOCK ── */
.json-block {
  font-family: var(--font-mono);
  font-size: 10px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  padding: 8px;
}

/* ── IMAGE PANEL ── */
.image-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
}

.image-frame {
  flex: 1;
  border: var(--border-strong);
  background: #f1f1f1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 140px;
  overflow: hidden;
}

.image-preview {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.open-btn {
  border: var(--border-strong);
  padding: 5px 12px;
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  background: var(--color-bg);
  cursor: pointer;
  align-self: flex-start;
  letter-spacing: 0.5px;
}

.open-btn:hover { background: var(--color-surface); }

/* ── EMPTY STATE ── */
.empty-state {
  padding: var(--space-6);
  text-align: center;
  color: var(--color-muted);
}

.empty-state h2 {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 6px;
}

.empty-state p {
  font-size: 11px;
  line-height: 1.5;
}
```

- [ ] **Step 2: Run tests — all must pass**

```bash
cd frontend && npm test -- --run 2>&1 | tail -30
```

Expected: all tests green.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/styles/app.css
git commit -m "design: complete app.css rewrite for MotherDuck redesign"
```

---

## Task 11: Build and Verify

- [ ] **Step 1: Run TypeScript type check**

```bash
cd frontend && npx tsc --noEmit 2>&1 | head -50
```

Fix any type errors before continuing. Common issues: missing imports, wrong prop types.

- [ ] **Step 2: Build the frontend**

```bash
cd frontend && npm run build 2>&1 | tail -20
```

Expected: build succeeds with no errors.

- [ ] **Step 3: Run the Tauri dev build**

```bash
cargo tauri dev
```

Open the app and verify:
- Brand "PNG ⌁ Compare" in dark topbar
- Single File / Directory mode toggle (yellow active state)
- Single file mode → PairComparison page with left/right inputs + Compare CTA
- Directory mode → DirectoryOverview page (empty state)
- After scanning a directory → card grid with colored headers
- Clicking a card → navigates to PairComparison with paths filled, auto-compares
- Back button returns to directory overview
- Tree / JSON / Image segmented control switches view mode
- Diff highlight toggle colors tree nodes
- Diff strip shows changed field list

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: complete UI redesign — MotherDuck design system"
```

---

## Self-Review Checklist

**Spec coverage:**
- ✅ Two-page model (DirectoryOverview + PairComparison) — Tasks 8, 9
- ✅ CTA double-layer button — Task 10 (app.css)
- ✅ Card grid with cardEnter animation — Task 10 (app.css) + Task 4
- ✅ Filter bar (client-side) — Tasks 2, 8
- ✅ Stats bar — Task 8
- ✅ View mode strip (Tree/JSON/Image) — Task 6
- ✅ Diff strip (center column) — Task 5
- ✅ Inline diff highlight in MetadataTree — Task 3
- ✅ buildDiffPathMap helper — Task 7
- ✅ directoryContext (back button + progress) — Task 2, 9
- ✅ Input focus shadow — Task 10 (app.css)
- ✅ Card hover lift effect — Task 10 (app.css)
- ✅ Design tokens update — Task 1
- ✅ Delete obsolete components — Task 9
- ✅ StatusBanner for errors — Tasks 8, 9 (inline error div)
- ✅ Empty state — Task 9 (EmptyState component preserved)

**Placeholder check:** No TBDs, no "implement later". All code blocks are complete.

**Type consistency:**
- `AppView` defined in useWorkbench.ts, imported where needed
- `ViewMode` defined in useWorkbench.ts, imported in ViewModeStrip + PairComparison
- `ActiveFilter` defined in useWorkbench.ts, imported in DirectoryOverview
- `buildDiffPathMap` defined in `lib/diffUtils.ts`, imported in PairComparison
- `totalDiffCount` defined in `lib/diffUtils.ts`, imported in PairComparison
- `filteredItems` returned from useWorkbench, consumed in DirectoryOverview
- `BatchListItemKind` used in FileCard and DirectoryOverview — matches `types.ts`
