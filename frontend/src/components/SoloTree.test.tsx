import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { SoloTree } from './SoloTree';

const meta = {
  StopName: '翻身地铁站',
  RoadName: '创业一路',
  HasHints: false,
  Lines: [
    {
      LineName: 'B932',
      Direction: '终点A',
      RouteStops: [{ Name: '上川', Sequence: 2, BuildingType: null, RoadName: '福城路' }],
    },
  ],
};

describe('SoloTree', () => {
  it('renders Chinese labels for top-level fields', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('中文站名')).toBeTruthy();
    expect(screen.getByText('翻身地铁站')).toBeTruthy();
    expect(screen.getByText('所在道路')).toBeTruthy();
    expect(screen.getByText('创业一路')).toBeTruthy();
  });

  it('formats booleans contextually', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('含温馨提示')).toBeTruthy();
    expect(screen.getByText('否')).toBeTruthy();
  });

  it('renders array group with count and is collapsed by default', () => {
    render(<SoloTree value={meta as any} />);
    expect(screen.getByText('停靠线路')).toBeTruthy();
    expect(screen.getByText('(1 项)')).toBeTruthy();
    // collapsed by default → array-item label NOT visible
    expect(screen.queryByText(/线路 1 · B932/)).toBeNull();
  });

  it('expands array group when clicked', () => {
    render(<SoloTree value={meta as any} />);
    const arrToggle = screen.getAllByRole('button', { name: '展开' })
      .find((b) => b.textContent === '▶')!;
    fireEvent.click(arrToggle);
    expect(screen.getByText(/线路 1 · B932/)).toBeTruthy();
  });

  it('does not show diff backgrounds in solo mode', () => {
    const { container } = render(<SoloTree value={meta as any} />);
    expect(container.querySelector('.kv--mod')).toBeNull();
    expect(container.querySelector('.kv--add')).toBeNull();
    expect(container.querySelector('.kv--rem')).toBeNull();
  });
});
