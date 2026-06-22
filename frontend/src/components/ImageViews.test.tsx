import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SoloImage, ImageSplit } from './ImageViews';

vi.mock('@tauri-apps/api/core', () => ({ convertFileSrc: (p: string) => `asset://${p}` }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openPath: vi.fn() }));

describe('image views', () => {
  it('SoloImage renders one pane, a zoom toolbar, and the single-column class', () => {
    const { container } = render(<SoloImage path="/x.png" name="x.png" />);
    expect(container.querySelectorAll('.image-pane')).toHaveLength(1);
    expect(container.querySelector('.image-split__panes--solo')).not.toBeNull();
    expect(screen.getByRole('button', { name: '重置' })).toBeInTheDocument();
  });

  it('SoloImage points the <img> at the converted asset url', () => {
    const { container } = render(<SoloImage path="/pics/a.png" name="a.png" />);
    expect(container.querySelector('img.image-pane__img')).toHaveAttribute('src', 'asset:///pics/a.png');
  });

  it('ImageSplit renders two panes', () => {
    const { container } = render(
      <ImageSplit leftPath="/l.png" rightPath="/r.png" leftName="l" rightName="r" />);
    expect(container.querySelectorAll('.image-pane')).toHaveLength(2);
  });
});
