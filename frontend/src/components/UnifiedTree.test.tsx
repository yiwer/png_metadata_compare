// frontend/src/components/UnifiedTree.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { UnifiedTree } from './UnifiedTree';
import { buildMirrorRows } from '../lib/treeModel';
import type { DiffNode } from '../lib/types';

const left = { StopName: '翻身地铁站', RoadName: '创业一路' };
const right = { StopName: '翻身地铁站', RoadName: '创业二路' };
const diff: DiffNode = {
  path: '', status: 'modified', left_value: null, right_value: null, summary: '', children: [
    { path: 'RoadName', status: 'modified', left_value: '创业一路', right_value: '创业二路', summary: '', children: [] },
  ],
};

function rowsFor(l: unknown, r: unknown, d: DiffNode | null) {
  return buildMirrorRows(l as never, r as never, d);
}

describe('UnifiedTree', () => {
  it('renders one label cell and two value cells per leaf', () => {
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
      leftLabel="a.png" rightLabel="b.png" focusRequest={null} />);
    expect(screen.getAllByText('创业一路')).toHaveLength(1);
    expect(screen.getAllByText('创业二路')).toHaveLength(1);
    expect(screen.getByText('a.png')).toBeInTheDocument();
    expect(screen.getByText('b.png')).toBeInTheDocument();
  });

  it('marks modified rows with status class only when highlight on', () => {
    const { container, rerender } = render(
      <UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
        leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(container.querySelector('.utree__row--modified')).not.toBeNull();
    rerender(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight={false} onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(container.querySelector('.utree__row--modified')).toBeNull();
  });

  it('onlyDiff hides unchanged leaves', () => {
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(screen.queryByText('翻身地铁站')).toBeNull();
    expect(screen.getByText('创业二路')).toBeInTheDocument();
  });

  it('solo mode renders a single value column without diff classes', () => {
    const { container } = render(
      <UnifiedTree rows={rowsFor(left, null, null)} solo="left" highlight onlyDiff={false}
        leftLabel="a.png" rightLabel="" focusRequest={null} />);
    expect(screen.getByText('翻身地铁站')).toBeInTheDocument();
    expect(container.querySelectorAll('.utree__val')).toHaveLength(2); // 两个叶子各一个值列
    expect(container.querySelector('.utree__row--modified')).toBeNull();
  });

  it('group toggle collapses children', () => {
    const nested = { Lines: [{ LineName: 'B932', Direction: '东' }] };
    render(<UnifiedTree rows={rowsFor(nested, nested, null)} solo={null} highlight onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    // Lines 数组默认折叠（defaultOpen=false）→ 内容不可见
    expect(screen.queryByText('B932')).toBeNull();
    fireEvent.click(screen.getAllByRole('button', { name: '展开' })[0]);
    expect(screen.queryAllByText('B932').length).toBeGreaterThan(0);
  });

  it('copy button writes value to clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<UnifiedTree rows={rowsFor(left, right, diff)} solo={null} highlight onlyDiff={false}
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    const btns = screen.getAllByRole('button', { name: '复制左值' });
    fireEvent.click(btns[0]);
    expect(writeText).toHaveBeenCalled();
  });

  it('focusRequest expands collapsed ancestors and scrolls to the row', () => {
    const nested = { Lines: [{ LineName: 'B932', Direction: '东' }] };
    const rows = rowsFor(nested, nested, null);
    const spy = vi.spyOn(Element.prototype, 'scrollIntoView');
    const { rerender } = render(
      <UnifiedTree rows={rows} solo={null} highlight onlyDiff={false}
        leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(screen.queryByText('B932')).toBeNull(); // Lines 默认折叠
    const path = rows[0].children!.find((r) => r.path === 'Lines')!.children![0]
      .children!.find((r) => r.label === '线路名称')!.path;
    rerender(
      <UnifiedTree rows={rows} solo={null} highlight onlyDiff={false}
        leftLabel="a" rightLabel="b" focusRequest={{ path, seq: 1 }} />);
    expect(screen.queryAllByText('B932').length).toBeGreaterThan(0); // 祖先被展开
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });

  it('onlyDiff auto-expands collapsed array groups that contain diffs', () => {
    const l = { Lines: [{ LineName: 'B932', Direction: '东', NextStop: 'A' }] };
    const r = { Lines: [{ LineName: 'B932', Direction: '东', NextStop: 'B' }] };
    const d: DiffNode = {
      path: '', status: 'modified', left_value: null, right_value: null, summary: '', children: [
        { path: 'Lines[B932|东].NextStop', status: 'modified', left_value: 'A', right_value: 'B', summary: '', children: [] },
      ],
    };
    render(<UnifiedTree rows={rowsFor(l, r, d)} solo={null} highlight onlyDiff
      leftLabel="a" rightLabel="b" focusRequest={null} />);
    expect(screen.getByText('A')).toBeInTheDocument(); // 折叠数组被自动展开且差异行可见
  });
});
