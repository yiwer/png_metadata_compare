import { useEffect, useState } from 'react';
import { loadRecent } from '../lib/recentDirs';
import type { Side, WorkbenchMode } from '../lib/types';

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function SelectionBar({
  mode, leftInput, rightInput,
  onPickLeft, onPickRight, onPastePath, onApplyPair, onClear, onDrop,
}: {
  mode: WorkbenchMode;
  leftInput: string;
  rightInput: string;
  onPickLeft(): void;
  onPickRight(): void;
  onPastePath(side: Side, path: string): void;
  onApplyPair(left: string, right: string): void;
  onClear(side: Side): void;
  onDrop(side: Side, path: string): void;
}) {
  const [openSide, setOpenSide] = useState<Side | null>(null);

  useEffect(() => {
    if (!openSide) return;
    const close = (e: MouseEvent) => {
      const inside = (e.target as HTMLElement).closest('.selbar__menu, .selbar__slot-main');
      if (!inside) setOpenSide(null);
    };
    window.addEventListener('mousedown', close);
    return () => window.removeEventListener('mousedown', close);
  }, [openSide]);

  const kind = mode === 'directory' ? 'dir' : 'file';
  const noun = mode === 'directory' ? '文件夹' : 'PNG 文件';

  const renderSlot = (side: Side, value: string) => {
    const onPick = side === 'left' ? onPickLeft : onPickRight;
    const sideLabel = side === 'left' ? '左' : '右';
    const dropHandler = (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      const p = (file as unknown as { path?: string })?.path;
      if (p) { onDrop(side, p); setOpenSide(null); }
    };
    return (
      <div className={`selbar__slot${value ? ' selbar__slot--filled' : ' selbar__slot--empty'}`}
        data-side={side} onDragOver={(e) => e.preventDefault()} onDrop={dropHandler}>
        <span className="selbar__side">{sideLabel}</span>
        <button type="button" className="selbar__slot-main" title={value || '未选择'}
          onClick={() => setOpenSide(openSide === side ? null : side)}>
          {value ? basename(value) : `点击选择${noun} / 拖入…`}
        </button>
        {value && (
          <button type="button" className="selbar__clear" aria-label={`清除${sideLabel}侧`}
            onClick={() => { onClear(side); setOpenSide(null); }}>✕</button>
        )}
        {openSide === side && (
          <div className="selbar__menu">
            <button type="button" onClick={() => { onPick(); setOpenSide(null); }}>浏览…</button>
            <input type="text" placeholder="粘贴路径后回车"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const v = (e.target as HTMLInputElement).value.trim();
                  if (v) { onPastePath(side, v); setOpenSide(null); }
                } else if (e.key === 'Escape') setOpenSide(null);
              }} />
            {loadRecent(kind).map((p) => (
              <button key={`${p.left}|${p.right}`} type="button" title={`${p.left}\n${p.right}`}
                onClick={() => { onApplyPair(p.left, p.right); setOpenSide(null); }}>
                {basename(p.left)} ⇄ {basename(p.right)}
              </button>
            ))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="selbar">
      {renderSlot('left', leftInput)}
      {renderSlot('right', rightInput)}
    </div>
  );
}
