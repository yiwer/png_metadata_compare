import { invoke } from '@tauri-apps/api/core';
import type { DirectorySummary, PairInspection, Side, SideInspection } from './types';

export async function compareSingle(leftPath: string, rightPath: string): Promise<PairInspection> {
  return invoke<PairInspection>('compare_single', {
    leftPath,
    rightPath,
  });
}

export async function scanDirectory(
  leftDir: string,
  rightDir: string,
): Promise<DirectorySummary> {
  return invoke<DirectorySummary>('scan_directory', {
    leftDir,
    rightDir,
  });
}

export async function inspectSingle(path: string, side: Side): Promise<SideInspection> {
  return invoke<SideInspection>('inspect_single', {
    path,
    side,
  });
}
