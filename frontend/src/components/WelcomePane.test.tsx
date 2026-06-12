import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WelcomePane } from './WelcomePane';
import { touchRecent } from '../lib/recentDirs';

describe('WelcomePane', () => {
  beforeEach(() => localStorage.clear());

  it('lists recent pairs for the current mode and applies on click', () => {
    touchRecent('dir', 'C:/tmp/bim_v1', 'C:/tmp/bim_v2');
    const onApplyPair = vi.fn();
    render(<WelcomePane mode="directory" onApplyPair={onApplyPair} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    fireEvent.click(screen.getByText(/bim_v1/));
    expect(onApplyPair).toHaveBeenCalledWith('C:/tmp/bim_v1', 'C:/tmp/bim_v2');
  });

  it('removes an entry via its × button without applying', () => {
    touchRecent('dir', 'C:/a', 'C:/b');
    const onApplyPair = vi.fn();
    render(<WelcomePane mode="directory" onApplyPair={onApplyPair} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: '删除该记录' }));
    expect(onApplyPair).not.toHaveBeenCalled();
    expect(screen.queryByText(/C:\/a/)).toBeNull();
  });

  it('shows mode-appropriate hint', () => {
    render(<WelcomePane mode="single" onApplyPair={() => {}} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    expect(screen.getByText(/PNG 文件/)).toBeInTheDocument();
  });

  it('refreshes the list when mode changes without remount', () => {
    touchRecent('file', 'C:/f1.png', 'C:/f2.png');
    touchRecent('dir', 'C:/d1', 'C:/d2');
    const { rerender } = render(
      <WelcomePane mode="single" onApplyPair={() => {}} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    expect(screen.getByText(/f1\.png/)).toBeInTheDocument();
    rerender(
      <WelcomePane mode="directory" onApplyPair={() => {}} onDrop={() => {}} onPickLeft={() => {}} onPickRight={() => {}} />);
    expect(screen.queryByText(/f1\.png/)).toBeNull();
    expect(screen.getByText(/d1/)).toBeInTheDocument();
  });
});
