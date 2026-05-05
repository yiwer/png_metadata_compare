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

  it('renders placeholder on the missing side for removed leaf', () => {
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
    // The right pane should have a placeholder for the "Hints" row.
    const placeholders = container.querySelectorAll('.kv--placeholder');
    expect(placeholders.length).toBeGreaterThan(0);
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
});
