import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { GroupHead } from './GroupHead';

describe('GroupHead', () => {
  it('renders label and count', () => {
    render(<GroupHead label="停靠线路" count={5} />);
    expect(screen.getByText('停靠线路')).toBeTruthy();
    expect(screen.getByText('(5 项)')).toBeTruthy();
  });

  it('shows ▼ when open and ▶ when closed and toggles via onToggle', () => {
    const onToggle = vi.fn();
    const { rerender } = render(<GroupHead label="x" open onToggle={onToggle} />);
    expect(screen.getByRole('button')).toHaveTextContent('▼');
    fireEvent.click(screen.getByRole('button'));
    expect(onToggle).toHaveBeenCalled();
    rerender(<GroupHead label="x" open={false} onToggle={onToggle} />);
    expect(screen.getByRole('button')).toHaveTextContent('▶');
  });

  it('emits data-level capped at 3 and drops the legacy nested class', () => {
    const { container, rerender } = render(<GroupHead label="x" />);
    expect(container.firstChild).toHaveAttribute('data-level', '0');
    rerender(<GroupHead label="x" level={1} />);
    expect(container.firstChild).toHaveAttribute('data-level', '1');
    rerender(<GroupHead label="x" level={2} />);
    expect(container.firstChild).toHaveAttribute('data-level', '2');
    rerender(<GroupHead label="x" level={5} />);
    expect(container.firstChild).toHaveAttribute('data-level', '3');
    expect(container.firstChild).not.toHaveClass('group-head--nested');
  });
});
