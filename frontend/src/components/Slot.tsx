// frontend/src/components/Slot.tsx
import { useState } from 'react';

function basename(p: string): string {
  if (!p) return '';
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function Slot({
  side,
  mode,
  value,
  errorMessage,
  onPick,
  onChange,
}: {
  side: 'left' | 'right';
  mode: 'single' | 'directory';
  value: string;
  errorMessage?: string | null;
  onPick(): void;
  onChange(path: string): void;
}) {
  const [drag, setDrag] = useState(false);
  const filled = value.length > 0;
  const error = !!errorMessage;

  const cls = [
    'slot',
    filled && 'slot--full',
    error && 'slot--error',
    drag && 'slot--dragover',
  ].filter(Boolean).join(' ');

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDrag(true);
  };
  const onDragLeave = () => setDrag(false);
  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDrag(false);
    const file = e.dataTransfer.files?.[0];
    // Tauri injects `path` on dropped files
    const p = (file as unknown as { path?: string })?.path;
    if (p) onChange(p);
  };

  if (!filled) {
    return (
      <div className={cls} data-side={side} onDragOver={onDragOver} onDragLeave={onDragLeave} onDrop={onDrop}>
        <span className="slot__icon">{mode === 'single' ? '⬚' : '▢'}</span>
        <span>{mode === 'single' ? '拖入 PNG 或' : '拖入目录或'}</span>
        <button type="button" className="slot__pick" onClick={onPick}>浏览</button>
        {error && <span className="banner banner--error">{errorMessage}</span>}
      </div>
    );
  }

  return (
    <div className={cls} data-side={side} onDragOver={onDragOver} onDragLeave={onDragLeave} onDrop={onDrop}>
      <span className="slot__icon">{mode === 'single' ? '⬚' : '▢'}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div className="slot__name">{basename(value)}</div>
        <div className="slot__sub">{value}</div>
        {error && <span className="banner banner--error">{errorMessage}</span>}
      </div>
      <button type="button" className="slot__pick" onClick={onPick}>替换</button>
      <button type="button" className="slot__clear" aria-label="清除" onClick={() => onChange('')}>×</button>
    </div>
  );
}
