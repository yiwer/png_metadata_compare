import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { MirrorTree } from './MirrorTree';
import type { DiffNode } from '../lib/types';

const diff: DiffNode = {
  path: '', status: 'modified', left_value: null, right_value: null, summary: '',
  children: [
    { path: 'FrameSize', status: 'modified', left_value: '"1050x1660"', right_value: '"1200x1800"', summary: '', children: [] },
    { path: 'Hints', status: 'removed', left_value: '"x"', right_value: null, summary: '', children: [] },
  ],
};

const left = { StopName: '翻身', FrameSize: '1050x1660', Hints: 'x' };
const right = { StopName: '翻身', FrameSize: '1200x1800' };

describe('MirrorTree', () => {
  it('renders both panes with values', () => {
    render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(screen.getAllByText('翻身').length).toBe(2);
    expect(screen.getByText('1050x1660')).toBeTruthy();
    expect(screen.getByText('1200x1800')).toBeTruthy();
  });

  it('applies kv--mod class to modified rows when highlight is on', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(container.querySelectorAll('.kv--mod').length).toBeGreaterThan(0);
  });

  it('omits status classes when highlight is off', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight={false}
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    expect(container.querySelectorAll('.kv--mod').length).toBe(0);
  });

  it('renders missing side as em dash with the same label as the present side', () => {
    const { container } = render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff={false}
        leftLabel="L"
        rightLabel="R"
      />,
    );
    // No "仅另一侧存在" placeholder anymore — both sides render the row uniformly.
    expect(container.querySelectorAll('.kv--placeholder').length).toBe(0);
    // The Hints row exists on both panes; right side shows em-dash since the data is absent.
    const dashes = Array.from(container.querySelectorAll('.kv__val')).filter(
      (n) => n.textContent === '—',
    );
    expect(dashes.length).toBeGreaterThan(0);
  });

  it('hides unchanged rows when onlyDiff is true', () => {
    render(
      <MirrorTree
        left={left as any}
        right={right as any}
        diffRoot={diff}
        highlight
        onlyDiff
        leftLabel="L"
        rightLabel="R"
      />,
    );
    // StopName is unchanged → not visible
    expect(screen.queryByText('翻身')).toBeNull();
    // FrameSize is modified → still visible
    expect(screen.getByText('1050x1660')).toBeTruthy();
  });

  it('resets folding state when input data changes', () => {
    const { rerender } = render(
      <MirrorTree
        left={left as any} right={right as any} diffRoot={diff}
        highlight onlyDiff={false} leftLabel="L" rightLabel="R"
      />,
    );

    // Render with completely different data; previously-closed groups must reset.
    const otherDiff = {
      path: '', status: 'unchanged' as const, left_value: null, right_value: null, summary: '', children: [],
    };
    const otherLeft = { Lines: [{ LineName: 'X9' }] };
    const otherRight = { Lines: [{ LineName: 'X9' }] };
    rerender(
      <MirrorTree
        left={otherLeft as any} right={otherRight as any} diffRoot={otherDiff}
        highlight onlyDiff={false} leftLabel="L" rightLabel="R"
      />,
    );
    // Lines is an array (default-closed). Its array-item label should NOT be visible.
    expect(screen.queryByText(/线路 1 · X9/)).toBeNull();
  });
});
