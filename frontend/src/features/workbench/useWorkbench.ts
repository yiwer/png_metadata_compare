// frontend/src/features/workbench/useWorkbench.ts
import { useEffect, useMemo, useRef, useState } from 'react';
import { workbenchApi } from '../../lib/api';
import type {
  BatchListItem,
  BatchListItemKind,
  DirectorySummary,
  PairInspection,
  ScanProgress,
  SideInspection,
  WorkbenchMode,
} from '../../lib/types';
import type { WorkbenchApi } from '../../lib/api';

export type AppView = 'welcome' | 'solo' | 'mirror' | 'directory-overview';
export type ViewMode = 'tree' | 'json' | 'image';
export type ActiveFilter = 'all' | BatchListItemKind;
export type Side = 'left' | 'right';
export type SortKey = 'diff-desc' | 'name-asc';

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

  const [inDirectorySubview, setInDirectorySubview] = useState<boolean>(false);

  const [viewMode, setViewMode] = useState<ViewMode>('tree');
  const [diffHighlight, setDiffHighlight] = useState(true);
  const [onlyDiff, setOnlyDiff] = useState(false);

  const [slotBarCollapsed, setSlotBarCollapsed] = useState(false);

  const [isLoading, setIsLoading] = useState(false);
  const [scanProgress, setScanProgress] = useState<ScanProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  // New state for sidebar selection & search
  const [searchQuery, setSearchQuery] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('diff-desc');
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [errorItem, setErrorItem] = useState<BatchListItem | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [railCollapsed, setRailCollapsed] = useState(false);

  // Directory scans are long-running; this guards against a superseded scan
  // overwriting the progress/results of a newer one.
  const scanSeqRef = useRef(0);
  // Selection calls are also async; this guards against a stale selectItem
  // call (e.g. auto-select from a previous scan) clobbering a newer result.
  const selectSeqRef = useRef(0);

  const toastTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    return () => {
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, []);

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
    setInDirectorySubview(false);
    setSearchQuery('');
    setSelectedItemId(null);
    setErrorItem(null);
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
      setSlotBarCollapsed(false);
      setSearchQuery('');
      setSelectedItemId(null);
      setErrorItem(null);
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
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    setToast(msg);
    toastTimerRef.current = setTimeout(() => setToast(null), 2200);
  }

  function toggleDiffHighlight() { setDiffHighlight((v) => !v); }
  function toggleOnlyDiff() { setOnlyDiff((v) => !v); }
  function toggleSlotBarCollapsed() { setSlotBarCollapsed((v) => !v); }
  function toggleSidebarCollapsed() { setSidebarCollapsed((v) => !v); }
  function toggleRailCollapsed() { setRailCollapsed((v) => !v); }

  function goBackToDirectory() {
    setView('directory-overview');
    setPairResult(null);
    setSoloResult(null);
    setSoloSide(null);
    setDirectoryContext(null);
    setInDirectorySubview(false);
  }

  async function selectItem(item: BatchListItem, itemsOverride?: BatchListItem[]) {
    const sid = ++selectSeqRef.current;
    setSelectedItemId(item.id);
    setErrorItem(null);
    setIsLoading(true);
    setError(null);

    const allItems = itemsOverride ?? directorySummary?.items ?? [];
    const differentItems = allItems.filter((i) => i.kind === 'different');
    const diffIndex = differentItems.findIndex((i) => i.id === item.id);
    setDirectoryContext(
      diffIndex >= 0 ? { index: diffIndex + 1, totalDifferent: differentItems.length } : null,
    );

    try {
      setInDirectorySubview(true);
      if (item.kind === 'error') {
        setErrorItem(item);
        setPairResult(null);
        setSoloResult(null);
        setSoloSide(null);
        setView('directory-overview'); // Task 7 will change to 'error'
      } else if (item.kind === 'left_only' && item.left_path) {
        const result = await api.inspectSingle(item.left_path, 'left');
        if (selectSeqRef.current !== sid) return;
        setSoloResult(result);
        setSoloSide('left');
        setView('solo');
        setViewMode('tree');
      } else if (item.kind === 'right_only' && item.right_path) {
        const result = await api.inspectSingle(item.right_path, 'right');
        if (selectSeqRef.current !== sid) return;
        setSoloResult(result);
        setSoloSide('right');
        setView('solo');
        setViewMode('tree');
      } else if (item.left_path && item.right_path) {
        const result = await api.compareSingle(item.left_path, item.right_path);
        if (selectSeqRef.current !== sid) return;
        setPairResult(result);
        setView('mirror');
        setViewMode('tree');
      } else {
        setError('无法打开此项目：路径缺失');
      }
    } catch (err) {
      if (selectSeqRef.current === sid) setError(formatError(err));
    } finally {
      if (selectSeqRef.current === sid) setIsLoading(false);
    }
  }

  // Backwards-compatible alias used by older tests.
  const navigateToPair = selectItem;

  async function selectByOffset(delta: number) {
    if (filteredItems.length === 0) return;
    const cur = filteredItems.findIndex((i) => i.id === selectedItemId);
    const next = cur < 0 ? 0 : Math.min(filteredItems.length - 1, Math.max(0, cur + delta));
    if (next === cur) return;
    await selectItem(filteredItems[next]);
  }

  async function selectNext() { await selectByOffset(1); }
  async function selectPrev() { await selectByOffset(-1); }

  // Task 8 之前后端还没有 cancel_scan 命令，这里先落桩：
  // api.cancelScan 是可选成员，缺省时本函数是 no-op。
  async function cancelScan() {
    try { await api.cancelScan?.(); } catch { /* 后端未实现/未启动时静默 */ }
  }

  async function runAuto() {
    const runId = ++scanSeqRef.current;
    selectSeqRef.current++; // invalidate any in-flight selectItem from a previous run
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
          // Keep slot bar expanded — only collapse once both sides are filled.
          setSlotBarCollapsed(false);
        } else {
          setView('welcome');
          setSlotBarCollapsed(false);
        }
        return;
      }
      // directory mode
      if (left && right) {
        const summary = await api.scanDirectory(left, right, (progress) => {
          if (scanSeqRef.current === runId) setScanProgress(progress);
        });
        if (scanSeqRef.current !== runId) return; // superseded by a newer scan
        setDirectorySummary(summary);
        setPairResult(null);
        setSoloResult(null); setSoloSide(null);
        setDirectoryContext(null);
        setView('directory-overview');
        setSlotBarCollapsed(true);
        const defaultFilter: ActiveFilter = summary.counts.different > 0 ? 'different' : 'all';
        setActiveFilter(defaultFilter);
        setSearchQuery('');
        setSelectedItemId(null);
        const visible = sortItems(
          summary.items.filter((i) => defaultFilter === 'all' || i.kind === defaultFilter),
          sortKey,
        );
        if (visible[0]) await selectItem(visible[0], summary.items);
      } else {
        setView('welcome');
        setSlotBarCollapsed(false);
      }
    } catch (err) {
      if (scanSeqRef.current === runId) setError(formatError(err));
    } finally {
      if (scanSeqRef.current === runId) {
        setIsLoading(false);
        setScanProgress(null);
      }
    }
  }

  // Backwards-compatible alias used by older tests.
  const runCompare = runAuto;

  const query = searchQuery.trim().toLowerCase();
  const filteredItems = useMemo(
    () => sortItems(
      (directorySummary?.items ?? [])
        .filter((i) => activeFilter === 'all' || i.kind === activeFilter)
        .filter((i) => !query || i.label.toLowerCase().includes(query)),
      sortKey,
    ),
    [directorySummary, activeFilter, query, sortKey],
  );

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
        if ((view === 'mirror' || view === 'solo') && inDirectorySubview) {
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
        if (items[next]) void selectItem(items[next]);
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
    inDirectorySubview,
    viewMode,
    diffHighlight,
    onlyDiff,
    slotBarCollapsed,
    isLoading,
    scanProgress,
    error,
    toast,
    // new
    searchQuery,
    setSearchQuery,
    sortKey,
    setSortKey,
    selectedItemId,
    selectItem,
    selectNext,
    selectPrev,
    errorItem,
    sidebarCollapsed,
    railCollapsed,
    toggleSidebarCollapsed,
    toggleRailCollapsed,
    setRailCollapsed,
    cancelScan,
    // setters
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
