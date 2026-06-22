import { renderHook, act } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { useImageTransform } from './useImageTransform';

describe('useImageTransform', () => {
  it('starts at zoom 1 with an identity translate/scale', () => {
    const { result } = renderHook(() => useImageTransform());
    expect(result.current.zoom).toBe(1);
    expect(result.current.transform).toBe('translate(0px, 0px) scale(1)');
  });

  it('zoomIn multiplies by 1.25, reset returns to 1', () => {
    const { result } = renderHook(() => useImageTransform());
    act(() => { result.current.zoomIn(); });
    expect(result.current.zoom).toBeCloseTo(1.25);
    act(() => { result.current.reset(); });
    expect(result.current.zoom).toBe(1);
  });

  it('drag updates the translate offset', () => {
    const { result } = renderHook(() => useImageTransform());
    act(() => { result.current.onMouseDown({ button: 0, clientX: 10, clientY: 10 } as unknown as React.MouseEvent); });
    act(() => { result.current.onMouseMove({ clientX: 30, clientY: 25 } as unknown as React.MouseEvent); });
    expect(result.current.transform).toBe('translate(20px, 15px) scale(1)');
    act(() => { result.current.endDrag(); });
  });
});
