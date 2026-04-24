import { useState } from 'react';
import { compareSingle, inspectSingle, scanDirectory } from '../../lib/api';
import type {
  AnalysisTab,
  BatchListItem,
  DirectorySummary,
  PairInspection,
  SideInspection,
  Side,
  WorkbenchMode,
} from '../../lib/types';

type ModeInputs = Record<WorkbenchMode, { left: string; right: string }>;

const DEFAULT_TAB: AnalysisTab = 'diff';

function emptyInputs(): ModeInputs {
  return {
    single: { left: '', right: '' },
    directory: { left: '', right: '' },
  };
}

function defaultTabForSingleSide(side: Side): AnalysisTab {
  return side === 'left' ? 'left-metadata' : 'right-metadata';
}

function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

export function useWorkbench() {
  const [mode, setModeState] = useState<WorkbenchMode>('single');
  const [inputsByMode, setInputsByMode] = useState<ModeInputs>(emptyInputs);
  const [directorySummary, setDirectorySummary] = useState<DirectorySummary | null>(null);
  const [activeResultItem, setActiveResultItem] = useState<BatchListItem | null>(null);
  const [activeInspection, setActiveInspection] = useState<PairInspection | null>(null);
  const [activeSingleSideInspection, setActiveSingleSideInspection] =
    useState<SideInspection | null>(null);
  const [activeTab, setActiveTab] = useState<AnalysisTab>(DEFAULT_TAB);
  const [activeNodePath, setActiveNodePath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [errorBanner, setErrorBanner] = useState<string | null>(null);

  const activeInputs = inputsByMode[mode];

  function clearRenderedState() {
    setDirectorySummary(null);
    setActiveResultItem(null);
    setActiveInspection(null);
    setActiveSingleSideInspection(null);
    setActiveTab(DEFAULT_TAB);
    setActiveNodePath(null);
    setErrorBanner(null);
  }

  function setMode(nextMode: WorkbenchMode) {
    setModeState(nextMode);
    clearRenderedState();
  }

  function setLeftInput(value: string) {
    setInputsByMode((current) => ({
      ...current,
      [mode]: {
        ...current[mode],
        left: value,
      },
    }));
  }

  function setRightInput(value: string) {
    setInputsByMode((current) => ({
      ...current,
      [mode]: {
        ...current[mode],
        right: value,
      },
    }));
  }

  async function loadItemInspection(item: BatchListItem, summary: DirectorySummary | null) {
    setActiveResultItem(item);
    setActiveInspection(null);
    setActiveSingleSideInspection(null);
    setActiveNodePath(null);

    if (summary) {
      setDirectorySummary(summary);
    }

    if (item.left_path && item.right_path) {
      const inspection = await compareSingle(item.left_path, item.right_path);
      setActiveInspection(inspection);
      setActiveSingleSideInspection(null);
      setActiveTab(DEFAULT_TAB);
      setActiveNodePath(inspection.default_selected_path);
      return;
    }

    if (item.left_path || item.right_path) {
      const side: Side = item.left_path ? 'left' : 'right';
      const inspection = await inspectSingle(item.left_path ?? item.right_path ?? '', side);
      setActiveInspection(null);
      setActiveSingleSideInspection(inspection);
      setActiveTab(defaultTabForSingleSide(side));
      setActiveNodePath(null);
      return;
    }

    setActiveTab('raw-json');
  }

  async function selectResultItem(itemId: string) {
    const item = directorySummary?.items.find((entry) => entry.id === itemId) ?? null;
    if (!item) {
      return;
    }

    setIsLoading(true);
    setErrorBanner(null);

    try {
      await loadItemInspection(item, directorySummary);
    } catch (error) {
      setErrorBanner(formatError(error));
    } finally {
      setIsLoading(false);
    }
  }

  async function runCompare() {
    setIsLoading(true);
    setErrorBanner(null);

    try {
      if (mode === 'single') {
        const inspection = await compareSingle(activeInputs.left, activeInputs.right);
        setDirectorySummary(null);
        setActiveResultItem(null);
        setActiveInspection(inspection);
        setActiveSingleSideInspection(null);
        setActiveTab(DEFAULT_TAB);
        setActiveNodePath(inspection.default_selected_path);
        return;
      }

      const summary = await scanDirectory(activeInputs.left, activeInputs.right);
      setDirectorySummary(summary);
      setActiveResultItem(null);
      setActiveInspection(null);
      setActiveSingleSideInspection(null);
      setActiveTab(DEFAULT_TAB);
      setActiveNodePath(null);

      const firstItem = summary.items[0];
      if (firstItem) {
        await loadItemInspection(firstItem, summary);
      }
    } catch (error) {
      clearRenderedState();
      setErrorBanner(formatError(error));
    } finally {
      setIsLoading(false);
    }
  }

  return {
    mode,
    leftInput: activeInputs.left,
    rightInput: activeInputs.right,
    directorySummary,
    activeResultItem,
    activeInspection,
    activeSingleSideInspection,
    activeTab,
    activeNodePath,
    isLoading,
    errorBanner,
    setMode,
    setLeftInput,
    setRightInput,
    setActiveTab,
    setActiveNodePath,
    runCompare,
    selectResultItem,
  };
}
