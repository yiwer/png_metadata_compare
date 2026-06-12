// frontend/src/components/DiffRail.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { DiffRail } from './DiffRail';
import type { DiffEntry } from '../lib/diffList';

// topGroup '线路 1 · B932' 与 buildDiffEntries 的就近分组输出对应：
// ancestors=['停靠线路','线路 1 · B932'] → slice(1).join(' › ') = '线路 1 · B932'
const entries: DiffEntry[] = [
  { path: 'Lines[B932|东].Direction', topGroup: '线路 1 · B932', label: '开往方向', status: 'modified', leftValue: 'A', rightValue: 'B' },
  { path: 'Lines[B932|东].NextStop', topGroup: '线路 1 · B932', label: '下一站', status: 'removed', leftValue: 'C', rightValue: '—' },
  { path: 'StopName', topGroup: '', label: '中文站名', status: 'added', leftValue: '—', rightValue: 'D' },
];

describe('DiffRail', () => {
  it('clusters entries by top group and shows count', () => {
    render(<DiffRail entries={entries} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    expect(screen.getByText('差异 3')).toBeInTheDocument();
    expect(screen.getByText('线路 1 · B932')).toBeInTheDocument();
    expect(screen.getByText(/开往方向/)).toBeInTheDocument();
  });

  it('clicking an entry calls onJump with its path', () => {
    const onJump = vi.fn();
    render(<DiffRail entries={entries} onJump={onJump} collapsed={false} onToggle={() => {}} />);
    fireEvent.click(screen.getByText(/开往方向/));
    expect(onJump).toHaveBeenCalledWith('Lines[B932|东].Direction');
  });

  it('copy button puts the diff text on the clipboard', () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<DiffRail entries={entries} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: '复制差异清单' }));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('开往方向: A → B'));
  });

  it('renders nothing but reopen handle when collapsed', () => {
    render(<DiffRail entries={entries} onJump={() => {}} collapsed onToggle={() => {}} />);
    expect(screen.queryByText('线路 1 · B932')).toBeNull();
  });

  it('shows 无差异 when empty', () => {
    render(<DiffRail entries={[]} onJump={() => {}} collapsed={false} onToggle={() => {}} />);
    expect(screen.getByText('无差异')).toBeInTheDocument();
  });
});
