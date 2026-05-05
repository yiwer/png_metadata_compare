import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SlotBar } from './SlotBar';

describe('SlotBar', () => {
  const baseProps = {
    mode: 'single' as const,
    leftValue: '',
    rightValue: '',
    onPickLeft: vi.fn(),
    onPickRight: vi.fn(),
    onLeftChange: vi.fn(),
    onRightChange: vi.fn(),
    onToggleCollapsed: vi.fn(),
  };

  it('renders both slots when expanded', () => {
    const { container } = render(<SlotBar {...baseProps} collapsed={false} />);
    expect(container.querySelectorAll('.slot').length).toBe(2);
  });

  it('renders collapsed summary when collapsed', () => {
    render(
      <SlotBar
        {...baseProps}
        collapsed
        leftValue="C:/a/翻身.png"
        rightValue="C:/b/翻身.png"
      />,
    );
    expect(screen.getAllByText(/翻身\.png/).length).toBe(2);
    expect(screen.getByLabelText('展开')).toBeTruthy();
  });

  it('calls onToggleCollapsed when expand button clicked', () => {
    const onToggleCollapsed = vi.fn();
    render(
      <SlotBar
        {...baseProps}
        collapsed
        leftValue="x" rightValue="y"
        onToggleCollapsed={onToggleCollapsed}
      />,
    );
    fireEvent.click(screen.getByLabelText('展开'));
    expect(onToggleCollapsed).toHaveBeenCalled();
  });
});
