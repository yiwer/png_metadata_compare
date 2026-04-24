import { act, renderHook, waitFor } from '@testing-library/react';
import { useWorkbench } from './useWorkbench';
import type {
  DirectorySummary,
  PairInspection,
  SideInspection,
} from '../../lib/types';
import type { WorkbenchApi } from '../../lib/api';

describe('useWorkbench', () => {
  it('keeps mode-specific inputs and synchronizes active result inspection state', async () => {
    const pairInspection: PairInspection = {
      left: {
        side: 'left',
        file_path: 'C:/left/a.png',
        file_name: 'a.png',
        raw_json: '{"Title":"Left"}',
        metadata: { Title: 'Left' },
        error: null,
      },
      right: {
        side: 'right',
        file_path: 'C:/right/a.png',
        file_name: 'a.png',
        raw_json: '{"Title":"Right"}',
        metadata: { Title: 'Right' },
        error: null,
      },
      diff_root: {
        path: 'StopPlateMetadata',
        status: 'modified',
        left_value: null,
        right_value: null,
        summary: 'StopPlateMetadata modified',
        children: [],
      },
      diff_summary: {
        modified: 1,
        added: 0,
        removed: 0,
        reordered: 0,
        error: 0,
      },
      default_selected_path: 'StopPlateMetadata.Title',
    };

    const leftOnlyInspection: SideInspection = {
      side: 'left',
      file_path: 'C:/left/only.png',
      file_name: 'only.png',
      raw_json: '{"Title":"Only"}',
      metadata: { Title: 'Only' },
      error: null,
    };

    const directorySummary: DirectorySummary = {
      counts: {
        identical: 1,
        different: 1,
        left_only: 1,
        right_only: 0,
        error: 0,
      },
      items: [
        {
          id: 'different-1',
          kind: 'different',
          label: 'diff.png',
          left_path: 'C:/left/diff.png',
          right_path: 'C:/right/diff.png',
          difference_count: 3,
          match_strategy: 'file_name',
          message: null,
        },
        {
          id: 'left-only-1',
          kind: 'left_only',
          label: 'only.png',
          left_path: 'C:/left/only.png',
          right_path: null,
          difference_count: 0,
          match_strategy: null,
          message: 'Missing on right',
        },
      ],
    };

    const api: WorkbenchApi = {
      compareSingle: vi.fn().mockResolvedValue(pairInspection),
      scanDirectory: vi.fn().mockResolvedValue(directorySummary),
      inspectSingle: vi.fn().mockResolvedValue(leftOnlyInspection),
    };

    const { result } = renderHook(() => useWorkbench(api));

    act(() => {
      result.current.setLeftInput('C:/left/single.png');
      result.current.setRightInput('C:/right/single.png');
    });

    await act(async () => {
      await result.current.runCompare();
    });

    expect(api.compareSingle).toHaveBeenCalledWith('C:/left/single.png', 'C:/right/single.png');
    expect(result.current.mode).toBe('single');
    expect(result.current.activeInspection).toEqual(pairInspection);
    expect(result.current.activeSingleSideInspection).toBeNull();
    expect(result.current.activeNodePath).toBe('StopPlateMetadata.Title');
    expect(result.current.activeTab).toBe('diff');

    act(() => {
      result.current.setMode('directory');
    });

    expect(result.current.leftInput).toBe('');
    expect(result.current.rightInput).toBe('');
    expect(result.current.activeInspection).toBeNull();
    expect(result.current.activeSingleSideInspection).toBeNull();
    expect(result.current.directorySummary).toBeNull();

    act(() => {
      result.current.setLeftInput('C:/left-dir');
      result.current.setRightInput('C:/right-dir');
    });

    await act(async () => {
      await result.current.runCompare();
    });

    await waitFor(() => {
      expect(result.current.activeResultItem?.id).toBe('different-1');
    });

    expect(api.scanDirectory).toHaveBeenCalledWith('C:/left-dir', 'C:/right-dir');
    expect(api.compareSingle).toHaveBeenLastCalledWith('C:/left/diff.png', 'C:/right/diff.png');
    expect(result.current.directorySummary).toEqual(directorySummary);
    expect(result.current.activeInspection).toEqual(pairInspection);
    expect(result.current.activeSingleSideInspection).toBeNull();
    expect(result.current.activeNodePath).toBe('StopPlateMetadata.Title');
    expect(result.current.activeTab).toBe('diff');

    await act(async () => {
      await result.current.selectResultItem('left-only-1');
    });

    expect(api.inspectSingle).toHaveBeenCalledWith('C:/left/only.png', 'left');
    expect(result.current.activeResultItem?.id).toBe('left-only-1');
    expect(result.current.activeInspection).toBeNull();
    expect(result.current.activeSingleSideInspection).toEqual(leftOnlyInspection);
    expect(result.current.activeTab).toBe('left_metadata');
    expect(result.current.activeNodePath).toBeNull();

    act(() => {
      result.current.setMode('single');
    });

    expect(result.current.leftInput).toBe('C:/left/single.png');
    expect(result.current.rightInput).toBe('C:/right/single.png');
    expect(result.current.directorySummary).toBeNull();
    expect(result.current.activeResultItem).toBeNull();
  });
});
