import { Channel, invoke } from '@tauri-apps/api/core';
import type { DirectorySummary, PairInspection, ScanProgress, Side, SideInspection } from './types';

export interface WorkbenchApi {
  compareSingle(leftPath: string, rightPath: string): Promise<PairInspection>;
  scanDirectory(
    leftDir: string,
    rightDir: string,
    onProgress?: (progress: ScanProgress) => void,
  ): Promise<DirectorySummary>;
  inspectSingle(path: string, side: Side): Promise<SideInspection>;
  cancelScan(): Promise<void>;
  pickFolder?(): Promise<string | null>;
}

export const workbenchApi: WorkbenchApi = {
  async compareSingle(leftPath: string, rightPath: string): Promise<PairInspection> {
    return invoke<PairInspection>('compare_single', {
      leftPath,
      rightPath,
    });
  },
  async scanDirectory(
    leftDir: string,
    rightDir: string,
    onProgress?: (progress: ScanProgress) => void,
  ): Promise<DirectorySummary> {
    const channel = new Channel<ScanProgress>();
    if (onProgress) channel.onmessage = onProgress;
    return invoke<DirectorySummary>('scan_directory', {
      leftDir,
      rightDir,
      onProgress: channel,
    });
  },
  async inspectSingle(path: string, side: Side): Promise<SideInspection> {
    return invoke<SideInspection>('inspect_single', {
      path,
      side,
    });
  },
  async cancelScan(): Promise<void> {
    await invoke('cancel_scan');
  },
  async pickFolder(): Promise<string | null> {
    return invoke<string | null>('pick_folder');
  },
};
