import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { DirectoryList } from './DirectoryList';
import type { DirectorySummary } from '../lib/types';

const summary: DirectorySummary = {
  counts: { identical: 1, different: 2, left_only: 1, right_only: 0, error: 0 },
  items: [
    { id: '1', kind: 'different', label: 'a.png', left_path: '/L/a.png', right_path: '/R/a.png', difference_count: 5, match_strategy: 'file_name', message: null },
    { id: '2', kind: 'different', label: 'b.png', left_path: '/L/b.png', right_path: '/R/b.png', difference_count: 1, match_strategy: 'file_name', message: null },
    { id: '3', kind: 'left_only', label: 'c.png', left_path: '/L/c.png', right_path: null, difference_count: 0, match_strategy: 'file_name', message: null },
    { id: '4', kind: 'identical', label: 'd.png', left_path: '/L/d.png', right_path: '/R/d.png', difference_count: 0, match_strategy: 'file_name', message: null },
  ],
};

describe('DirectoryList', () => {
  it('renders stat counts', () => {
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={() => {}} onSelect={() => {}} />,
    );
    expect(screen.getByText('2')).toBeTruthy();
    expect(screen.getByText('不一致')).toBeTruthy();
    expect(screen.getByText('仅左')).toBeTruthy();
  });

  it('renders rows for each filtered item', () => {
    const filtered = summary.items.filter((i) => i.kind === 'different');
    render(
      <DirectoryList summary={summary} filteredItems={filtered}
        activeFilter="different" onFilter={() => {}} onSelect={() => {}} />,
    );
    expect(screen.getByText('a.png')).toBeTruthy();
    expect(screen.getByText('b.png')).toBeTruthy();
    expect(screen.queryByText('d.png')).toBeNull();
    expect(screen.getByText('5 处不同')).toBeTruthy();
  });

  it('clicking a row calls onSelect with that item', () => {
    const onSelect = vi.fn();
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={() => {}} onSelect={onSelect} />,
    );
    fireEvent.click(screen.getByText('a.png').closest('.dirlist__row')!);
    expect(onSelect).toHaveBeenCalledWith(summary.items[0]);
  });

  it('clicking a chip calls onFilter', () => {
    const onFilter = vi.fn();
    render(
      <DirectoryList summary={summary} filteredItems={summary.items}
        activeFilter="all" onFilter={onFilter} onSelect={() => {}} />,
    );
    fireEvent.click(screen.getByText(/^不一致 2$/));
    expect(onFilter).toHaveBeenCalledWith('different');
  });
});
