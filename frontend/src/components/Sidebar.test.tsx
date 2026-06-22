// frontend/src/components/Sidebar.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Sidebar } from './Sidebar';
import type { BatchListItem, DirectorySummary } from '../lib/types';

const items: BatchListItem[] = [
  { id: '1', kind: 'different', label: 'a.png', left_path: '/l/a.png', right_path: '/r/a.png', difference_count: 3, match_strategy: 'file_name', message: null },
  { id: '2', kind: 'left_only', label: 'b.png', left_path: '/l/b.png', right_path: null, difference_count: 0, match_strategy: null, message: null },
];
const summary: DirectorySummary = {
  counts: { identical: 0, different: 1, left_only: 1, right_only: 0, error: 0 },
  items,
};

function renderBar(over: Partial<Parameters<typeof Sidebar>[0]> = {}) {
  const props = {
    summary, filteredItems: items, activeFilter: 'all' as const,
    searchQuery: '', sortKey: 'diff-desc' as const,
    selectedItemId: '1', isLoading: false, scanProgress: null,
    onFilter: vi.fn(), onSearch: vi.fn(), onSort: vi.fn(), onSelect: vi.fn(),
    onCancelScan: vi.fn(),
    ...over,
  };
  render(<Sidebar {...props} />);
  return props;
}

describe('Sidebar', () => {
  beforeEach(() => localStorage.clear());
  it('renders rows and marks the selected one', () => {
    renderBar();
    expect(screen.getByText('a.png').closest('.sidebar__row')).toHaveAttribute('data-selected', 'true');
    expect(screen.getByText('b.png')).toBeInTheDocument();
  });

  it('typing in search calls onSearch', () => {
    const p = renderBar();
    fireEvent.change(screen.getByPlaceholderText('搜索文件名…'), { target: { value: 'foo' } });
    expect(p.onSearch).toHaveBeenCalledWith('foo');
  });

  it('clicking a chip calls onFilter', () => {
    const p = renderBar();
    fireEvent.click(screen.getByRole('button', { name: /仅左 1/ }));
    expect(p.onFilter).toHaveBeenCalledWith('left_only');
  });

  it('clicking a row calls onSelect with the item', () => {
    const p = renderBar();
    fireEvent.click(screen.getByText('b.png'));
    expect(p.onSelect).toHaveBeenCalledWith(items[1]);
  });

  it('shows progress and cancel while loading', () => {
    const p = renderBar({ isLoading: true, scanProgress: { stage: 'comparing', done: 87, total: 600 } });
    expect(screen.getByText(/87 \/ 600/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '取消' }));
    expect(p.onCancelScan).toHaveBeenCalled();
  });

  it('empty filtered list shows clear-search hint when query active', () => {
    const p = renderBar({ filteredItems: [], searchQuery: 'zzz' });
    fireEvent.click(screen.getByRole('button', { name: '清空搜索' }));
    expect(p.onSearch).toHaveBeenCalledWith('');
  });

});
