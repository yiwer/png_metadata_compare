import { invoke } from '@tauri-apps/api/core';
import type { DirectorySummary, PairInspection, Side, SideInspection } from './types';

export interface WorkbenchApi {
  compareSingle(leftPath: string, rightPath: string): Promise<PairInspection>;
  scanDirectory(leftDir: string, rightDir: string): Promise<DirectorySummary>;
  inspectSingle(path: string, side: Side): Promise<SideInspection>;
}

export const workbenchApi: WorkbenchApi = {
  async compareSingle(leftPath: string, rightPath: string): Promise<PairInspection> {
    return invoke<PairInspection>('compare_single', {
      leftPath,
      rightPath,
    });
  },
  async scanDirectory(leftDir: string, rightDir: string): Promise<DirectorySummary> {
    return invoke<DirectorySummary>('scan_directory', {
      leftDir,
      rightDir,
    });
  },
  async inspectSingle(path: string, side: Side): Promise<SideInspection> {
    return invoke<SideInspection>('inspect_single', {
      path,
      side,
    });
  },
};
