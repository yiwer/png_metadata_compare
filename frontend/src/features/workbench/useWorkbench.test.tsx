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
