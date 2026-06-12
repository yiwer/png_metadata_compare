// frontend/src/lib/diffList.test.ts
import { describe, it, expect } from 'vitest';
import { buildDiffEntries, buildDiffText } from './diffList';
import { buildMirrorRows } from './treeModel';
import type { DiffNode, JsonValue } from './types';

const diff = (children: DiffNode[]): DiffNode =>
  ({ path: '', status: 'modified', left_value: null, right_value: null, summary: '', children });
const node = (path: string, status: DiffNode['status']): DiffNode =>
  ({ path, status, left_value: null, right_value: null, summary: '', children: [] });

describe('buildDiffEntries', () => {
  it('collects changed leaves with top group label', () => {
    const left = { StopName: 'A', Lines: [{ LineName: 'B932', Direction: '东', NextStop: '尚都花园' }] };
    const right = { StopName: 'A', Lines: [{ LineName: 'B932', Direction: '东' }] };
    const rows = buildMirrorRows(left, right, diff([
      node('Lines[B932]', 'modified'),    // 容器变化不计入条目
      node('Lines[B932].Direction', 'modified'),
      node('Lines[B932].NextStop', 'removed'),
    ]));
    const entries = buildDiffEntries(rows);
    expect(entries).toHaveLength(2);
    // 就近分组：ancestors=['停靠线路','线路 1 · B932'] → topGroup='线路 1 · B932'（丢弃顶层容器名）
    expect(entries[0].topGroup).toBe('线路 1 · B932');
    expect(entries[0].label).toBe('开往方向');
    expect(entries.map((e) => e.status)).toEqual(['modified', 'removed']);
  });

  it('counts an array item that exists on one side as a single unit', () => {
    const left = { Lines: [] as JsonValue[] };
    const right = { Lines: [{ LineName: 'M197', Direction: '北' }] };
    const rows = buildMirrorRows(left, right, diff([node('Lines[M197]', 'added')]));
    const entries = buildDiffEntries(rows);
    expect(entries).toHaveLength(1);
    expect(entries[0].status).toBe('added');
    // ancestors=['停靠线路']，深度 1 → topGroup='停靠线路'
    expect(entries[0].topGroup).toBe('停靠线路');
  });
});

describe('buildDiffText', () => {
  it('formats modified / one-side lines', () => {
    const text = buildDiffText([
      { path: 'a', topGroup: '线路 1 · B932', label: '开往方向', status: 'modified', leftValue: '福城万达广场', rightValue: '福城天虹' },
      { path: 'b', topGroup: '线路 1 · B932', label: '下一站', status: 'removed', leftValue: '尚都花园', rightValue: '—' },
      { path: 'c', topGroup: '', label: '中文站名', status: 'added', leftValue: '—', rightValue: '新站' },
    ]);
    expect(text.split('\n')).toEqual([
      '线路 1 · B932 › 开往方向: 福城万达广场 → 福城天虹',
      '线路 1 · B932 › 下一站: 尚都花园（仅左）',
      '中文站名: 新站（仅右）',
    ]);
  });

  it('whole-item add/remove copies without literal null', () => {
    const left = { Lines: [] as JsonValue[] };
    const right = { Lines: [{ LineName: 'M197', Direction: '北' }] };
    const rows = buildMirrorRows(left, right, diff([node('Lines[M197]', 'added')]));
    const text = buildDiffText(buildDiffEntries(rows));
    expect(text).not.toContain('null');
    expect(text).toContain('（仅右侧整项）');
  });
});
