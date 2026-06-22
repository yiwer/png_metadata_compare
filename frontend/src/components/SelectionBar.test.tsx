import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SelectionBar } from './SelectionBar';
import { touchRecent } from '../lib/recentDirs';

function renderBar(over: Partial<Parameters<typeof SelectionBar>[0]> = {}) {
  const props = {
    mode: 'single' as const, leftInput: '', rightInput: '',
    onPickLeft: vi.fn(), onPickRight: vi.fn(), onPastePath: vi.fn(),
    onApplyPair: vi.fn(), onClear: vi.fn(), onDrop: vi.fn(),
    ...over,
  };
  render(<SelectionBar {...props} />);
  return props;
}

describe('SelectionBar', () => {
  beforeEach(() => localStorage.clear());

  it('empty slots show a placeholder and no clear button', () => {
    renderBar();
    expect(screen.getAllByText(/点击选择PNG 文件/)).toHaveLength(2);
    expect(screen.queryByLabelText('清除左侧')).toBeNull();
  });

  it('filled slot shows the basename and a working clear button', () => {
    const p = renderBar({ leftInput: 'C:/pics/photo_a.png' });
    expect(screen.getByText('photo_a.png')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('清除左侧'));
    expect(p.onClear).toHaveBeenCalledWith('left');
  });

  it('clicking a slot opens its dropdown; 浏览… calls the pick handler', () => {
    const p = renderBar();
    fireEvent.click(screen.getAllByText(/点击选择/)[0]);
    fireEvent.click(screen.getByRole('button', { name: '浏览…' }));
    expect(p.onPickLeft).toHaveBeenCalled();
  });

  it('pasting a path + Enter calls onPastePath for that side (directory mode)', () => {
    const p = renderBar({ mode: 'directory' });
    fireEvent.click(screen.getAllByText(/点击选择/)[1]); // 右槽
    const input = screen.getByPlaceholderText('粘贴路径后回车');
    fireEvent.change(input, { target: { value: 'D:/folder' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(p.onPastePath).toHaveBeenCalledWith('right', 'D:/folder');
  });

  it('recent list (mode-appropriate) applies a pair', () => {
    touchRecent('file', 'C:/a.png', 'C:/b.png');
    const p = renderBar({ mode: 'single' });
    fireEvent.click(screen.getAllByText(/点击选择/)[0]);
    fireEvent.click(screen.getByText(/a\.png ⇄ b\.png/));
    expect(p.onApplyPair).toHaveBeenCalledWith('C:/a.png', 'C:/b.png');
  });
});
