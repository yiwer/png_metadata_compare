import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Slot } from './Slot';

describe('Slot', () => {
  it('renders empty state with mode hint', () => {
    render(
      <Slot side="left" mode="single" value="" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText(/拖入 PNG/)).toBeTruthy();
  });

  it('renders directory mode hint', () => {
    render(
      <Slot side="right" mode="directory" value="" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText(/拖入目录/)).toBeTruthy();
  });

  it('renders filename when filled', () => {
    render(
      <Slot side="left" mode="single" value="C:/x/翻身.png" onPick={() => {}} onChange={() => {}} />,
    );
    expect(screen.getByText('翻身.png')).toBeTruthy();
  });

  it('calls onPick when 浏览 button clicked', () => {
    const onPick = vi.fn();
    render(
      <Slot side="left" mode="single" value="" onPick={onPick} onChange={() => {}} />,
    );
    fireEvent.click(screen.getByText(/浏览/));
    expect(onPick).toHaveBeenCalled();
  });

  it('calls onChange("") when clear clicked', () => {
    const onChange = vi.fn();
    render(
      <Slot side="left" mode="single" value="C:/x/y.png" onPick={() => {}} onChange={onChange} />,
    );
    fireEvent.click(screen.getByLabelText('清除'));
    expect(onChange).toHaveBeenCalledWith('');
  });

  it('shows error styling and message', () => {
    const { container } = render(
      <Slot side="left" mode="single" value="C:/x.png" errorMessage="无元数据"
            onPick={() => {}} onChange={() => {}} />,
    );
    expect(container.querySelector('.slot--error')).toBeTruthy();
    expect(screen.getByText('无元数据')).toBeTruthy();
  });
});
