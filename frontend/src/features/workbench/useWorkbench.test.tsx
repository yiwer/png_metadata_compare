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
    inspectSingle: vi.fn().mockResolvedValue({
      side: 'left',
      file_path: '/x.png',
      file_name: 'x.png',
      raw_json: null,
      metadata: null,
      error: null,
    }),
    ...overrides,
  };
}

describe('useWorkbench', () => {
  it('starts in single mode, welcome view', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    expect(result.current.mode).toBe('single');
    expect(result.current.view).toBe('welcome');
  });

  it('switching mode resets state', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setMode('directory'); });
    expect(result.current.mode).toBe('directory');
    expect(result.current.view).toBe('welcome');
    expect(result.current.pairResult).toBeNull();
  });

  it('runCompare (single) calls compareSingle and sets pairResult', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runCompare(); });
    expect(api.compareSingle).toHaveBeenCalledWith('/a.png', '/b.png');
    expect(result.current.pairResult).toBe(mockInspection);
    expect(result.current.view).toBe('mirror');
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
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    act(() => { result.current.setActiveFilter('all'); });
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
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => {
      await result.current.navigateToPair(mockSummary.items[1]); // second 'different' item
    });
    expect(result.current.directoryContext).toEqual({ index: 2, totalDifferent: 2 });
    expect(result.current.view).toBe('mirror');
  });

  it('goBackToDirectory resets view and pairResult', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/left'); result.current.setRightInput('/right'); });
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

  it('view starts as welcome when both inputs empty', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    expect(result.current.view).toBe('welcome');
  });

  it('runAuto: single mode + only left filled → solo (left)', async () => {
    const sideInspection = {
      side: 'left' as const, file_path: '/a.png', file_name: 'a.png',
      raw_json: '{"k":1}', metadata: { k: 1 }, error: null,
    };
    const api = makeApi({ inspectSingle: vi.fn().mockResolvedValue(sideInspection) });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(api.inspectSingle).toHaveBeenCalledWith('/a.png', 'left');
    expect(result.current.view).toBe('solo');
    expect(result.current.soloSide).toBe('left');
    expect(result.current.soloResult).toBe(sideInspection);
  });

  it('runAuto: single mode + both filled → mirror', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('mirror');
    expect(result.current.pairResult).toBe(mockInspection);
  });

  it('runAuto: directory mode + both filled → directory-overview', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.setLeftInput('/L'); result.current.setRightInput('/R'); });
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.view).toBe('directory-overview');
  });

  it('tryDropPath: dropping .png while in directory mode auto-switches to single', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setMode('directory'); });
    act(() => { result.current.tryDropPath('left', '/some/file.png'); });
    expect(result.current.mode).toBe('single');
    expect(result.current.leftInput).toBe('/some/file.png');
    expect(result.current.toast).toMatch(/已切换到单文件模式/);
  });

  it('tryDropPath: dropping non-png while in single mode auto-switches to directory', () => {
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.tryDropPath('right', '/some/folder'); });
    expect(result.current.mode).toBe('directory');
    expect(result.current.rightInput).toBe('/some/folder');
    expect(result.current.toast).toMatch(/已切换到目录模式/);
  });

  it('slot bar collapses after first successful analysis with both filled', async () => {
    const api = makeApi();
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setLeftInput('/a.png'); result.current.setRightInput('/b.png'); });
    expect(result.current.slotBarCollapsed).toBe(false);
    await act(async () => { await result.current.runAuto(); });
    expect(result.current.slotBarCollapsed).toBe(true);
  });

  it('navigateToPair: left_only item → solo left', async () => {
    const onlyLeft = {
      id: 'L', kind: 'left_only' as const, label: 'x.png',
      left_path: '/L/x.png', right_path: null, difference_count: 0,
      match_strategy: 'file_name' as const, message: null,
    };
    const sideInspection = {
      side: 'left' as const, file_path: '/L/x.png', file_name: 'x.png',
      raw_json: null, metadata: { Foo: 'bar' }, error: null,
    };
    const api = makeApi({ inspectSingle: vi.fn().mockResolvedValue(sideInspection) });
    const { result } = renderHook(() => useWorkbench(api));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => { await result.current.navigateToPair(onlyLeft as any); });
    expect(api.inspectSingle).toHaveBeenCalledWith('/L/x.png', 'left');
    expect(result.current.view).toBe('solo');
    expect(result.current.soloSide).toBe('left');
  });

  it('navigateToPair: surfaces error when paths are missing', async () => {
    const broken = {
      id: 'X', kind: 'different' as const, label: 'x.png',
      left_path: null, right_path: null, difference_count: 5,
      match_strategy: 'file_name' as const, message: null,
    };
    const { result } = renderHook(() => useWorkbench(makeApi()));
    act(() => { result.current.setMode('directory'); });
    await act(async () => { await result.current.runCompare(); });
    await act(async () => { await result.current.navigateToPair(broken as any); });
    expect(result.current.error).toMatch(/路径缺失/);
  });
});
