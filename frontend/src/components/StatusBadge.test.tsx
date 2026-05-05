import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { StatusBadge } from './StatusBadge';

describe('StatusBadge', () => {
  it('renders label with the right kind class', () => {
    render(<StatusBadge kind="modified">12 处不同</StatusBadge>);
    const badge = screen.getByText('12 处不同');
    expect(badge.className).toContain('badge--mod');
  });

  it('handles all kinds', () => {
    const { container } = render(
      <>
        <StatusBadge kind="modified">m</StatusBadge>
        <StatusBadge kind="added">a</StatusBadge>
        <StatusBadge kind="removed">r</StatusBadge>
        <StatusBadge kind="error">e</StatusBadge>
        <StatusBadge kind="unchanged">u</StatusBadge>
        <StatusBadge kind="reordered">o</StatusBadge>
      </>,
    );
    const html = container.innerHTML;
    expect(html).toContain('badge--mod');
    expect(html).toContain('badge--add');
    expect(html).toContain('badge--rem');
    expect(html).toContain('badge--err');
    expect(html).toContain('badge--neu');
  });
});
