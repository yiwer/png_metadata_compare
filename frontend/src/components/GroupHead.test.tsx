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

  it('applies nested class when level > 0', () => {
    const { container } = render(<GroupHead label="x" level={1} />);
    expect(container.firstChild).toHaveClass('group-head--nested');
  });
});
