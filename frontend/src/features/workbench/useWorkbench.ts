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
