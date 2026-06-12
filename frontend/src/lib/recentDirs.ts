export interface RecentPair {
  left: string;
  right: string;
  lastUsed: number;   // epoch ms
}

export type RecentKind = 'dir' | 'file';

const KEYS: Record<RecentKind, string> = {
  dir: 'recent.dirPairs',
  file: 'recent.filePairs',
};
const CAP = 8;

export function loadRecent(kind: RecentKind): RecentPair[] {
  try {
    const raw = localStorage.getItem(KEYS[kind]);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (p): p is RecentPair =>
        typeof p?.left === 'string' && typeof p?.right === 'string' && typeof p?.lastUsed === 'number',
    );
  } catch {
    return [];
  }
}

export function touchRecent(kind: RecentKind, left: string, right: string): void {
  const rest = loadRecent(kind).filter((p) => !(p.left === left && p.right === right));
  const next = [{ left, right, lastUsed: Date.now() }, ...rest].slice(0, CAP);
  try { localStorage.setItem(KEYS[kind], JSON.stringify(next)); } catch { /* storage full: 忽略 */ }
}

export function removeRecent(kind: RecentKind, left: string, right: string): void {
  const next = loadRecent(kind).filter((p) => !(p.left === left && p.right === right));
  try { localStorage.setItem(KEYS[kind], JSON.stringify(next)); } catch { /* ignore */ }
}
