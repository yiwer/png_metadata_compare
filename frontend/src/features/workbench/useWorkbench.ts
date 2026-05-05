// frontend/src/features/workbench/useWorkbench.ts
import { useEffect, useRef, useState } from 'react';
import { workbenchApi } from '../../lib/api';
import type {
  BatchListItem,
  BatchListItemKind,
  DirectorySummary,
  PairInspection,
  SideInspection,
  WorkbenchMode,
} from '../../lib/types';
import type { WorkbenchApi } from '../../lib/api';

export type AppView = 'welcome' | 'solo' | 'mirror' | 'directory-overview';
export type ViewMode = 'tree' | 'json' | 'image';
export type ActiveFilter = 'all' | BatchListItemKind;
export type Side = 'left' | 'right';

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
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

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

  function goBackToDirectory() {
    setView('directory-overview');
    setPairResult(null);
    setSoloResult(null);
    setSoloSide(null);
    setDirectoryContext(null);
    setInDirectorySubview(false);
  }

  async function navigateToPair(item: BatchListItem) {
    setIsLoading(true);
    setError(null);

    const differentItems = (directorySummary?.items ?? []).filter((i) => i.kind === 'different');
    const diffIndex = differentItems.findIndex((i) => i.id === item.id);
    setDirectoryContext(
      diffIndex >= 0 ? { index: diffIndex + 1, totalDifferent: differentItems.length } : null,
    );

    try {
      setInDirectorySubview(true);
      if (item.kind === 'left_only' && item.left_path) {
        const result = await api.inspectSingle(item.left_path, 'left');
        setSoloResult(result);
        setSoloSide('left');
        setView('solo');
        setViewMode('tree');
      } else if (item.kind === 'right_only' && item.right_path) {
        const result = await api.inspectSingle(item.right_path, 'right');
        setSoloResult(result);
        setSoloSide('right');
        setView('solo');
        setViewMode('tree');
      } else if (item.left_path && item.right_path) {
        const result = await api.compareSingle(item.left_path, item.right_path);
        setPairResult(result);
        setView('mirror');
        setViewMode('tree');
      } else {
        setError('无法打开此项目：路径缺失');
      }
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function runAuto() {
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
          setSlotBarCollapsed(true);
        } else {
          setView('welcome');
          setSlotBarCollapsed(false);
        }
        return;
      }
      // directory mode
      if (left && right) {
        const summary = await api.scanDirectory(left, right);
        setDirectorySummary(summary);
        setPairResult(null);
        setSoloResult(null); setSoloSide(null);
        setDirectoryContext(null);
        setActiveFilter('different');
        setView('directory-overview');
        setSlotBarCollapsed(true);
      } else {
        setView('welcome');
        setSlotBarCollapsed(false);
      }
    } catch (err) {
      setError(formatError(err));
    } finally {
      setIsLoading(false);
    }
  }

  // Backwards-compatible alias used by older tests.
  const runCompare = runAuto;

  const filteredItems =
    activeFilter === 'all'
      ? (directorySummary?.items ?? [])
      : (directorySummary?.items ?? []).filter((i) => i.kind === activeFilter);

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
        if (items[next]) void navigateToPair(items[next]);
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
    error,
    toast,
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
