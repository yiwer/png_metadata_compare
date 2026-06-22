import { useRef, useState } from 'react';
import type * as React from 'react';

export interface ImageTransform {
  zoom: number;
  transform: string;
  dragging: boolean;
  onWheel(e: React.WheelEvent): void;
  onMouseDown(e: React.MouseEvent): void;
  onMouseMove(e: React.MouseEvent): void;
  endDrag(): void;
  zoomIn(): void;
  zoomOut(): void;
  reset(): void;
}

export function useImageTransform(): ImageTransform {
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const dragRef = useRef<{ startX: number; startY: number; baseX: number; baseY: number } | null>(null);

  const onWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
    setZoom((z) => Math.min(20, Math.max(0.1, z * factor)));
  };
  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY, baseX: offset.x, baseY: offset.y };
  };
  const onMouseMove = (e: React.MouseEvent) => {
    if (!dragRef.current) return;
    setOffset({
      x: dragRef.current.baseX + (e.clientX - dragRef.current.startX),
      y: dragRef.current.baseY + (e.clientY - dragRef.current.startY),
    });
  };
  const endDrag = () => { dragRef.current = null; };
  const zoomIn = () => setZoom((z) => Math.min(20, z * 1.25));
  const zoomOut = () => setZoom((z) => Math.max(0.1, z / 1.25));
  const reset = () => { setZoom(1); setOffset({ x: 0, y: 0 }); };

  const transform = `translate(${offset.x}px, ${offset.y}px) scale(${zoom})`;
  const dragging = dragRef.current !== null;
  return { zoom, transform, dragging, onWheel, onMouseDown, onMouseMove, endDrag, zoomIn, zoomOut, reset };
}
