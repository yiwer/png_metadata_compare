import { describe, it, expect, beforeEach } from 'vitest';
import { loadRecent, touchRecent, removeRecent } from './recentDirs';

describe('recentDirs', () => {
  beforeEach(() => localStorage.clear());

  it('touch adds to front and dedupes by both paths', () => {
    touchRecent('dir', 'L1', 'R1');
    touchRecent('dir', 'L2', 'R2');
    touchRecent('dir', 'L1', 'R1');
    const list = loadRecent('dir');
    expect(list.map((p) => p.left)).toEqual(['L1', 'L2']);
  });

  it('caps at 8 entries', () => {
    for (let i = 0; i < 10; i++) touchRecent('dir', `L${i}`, `R${i}`);
    expect(loadRecent('dir')).toHaveLength(8);
    expect(loadRecent('dir')[0].left).toBe('L9');
  });

  it('file and dir lists are independent', () => {
    touchRecent('dir', 'D', 'D2');
    touchRecent('file', 'F', 'F2');
    expect(loadRecent('dir')).toHaveLength(1);
    expect(loadRecent('file')).toHaveLength(1);
  });

  it('remove drops the matching pair', () => {
    touchRecent('dir', 'L1', 'R1');
    touchRecent('dir', 'L2', 'R2');
    removeRecent('dir', 'L1', 'R1');
    expect(loadRecent('dir').map((p) => p.left)).toEqual(['L2']);
  });

  it('survives corrupted storage', () => {
    localStorage.setItem('recent.dirPairs', '{not json');
    expect(loadRecent('dir')).toEqual([]);
  });
});
