import { useState } from 'react';
import { loadRecent, removeRecent } from '../lib/recentDirs';
import type { RecentKind, RecentPair } from '../lib/recentDirs';
import type { Side, WorkbenchMode } from '../lib/types';

function basename(p: string): string {
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

function relTime(ts: number): string {
  const mins = Math.round((Date.now() - ts) / 60000);
  if (mins < 1) return '刚刚';
  if (mins < 60) return `${mins} 分钟前`;
  const hours = Math.round(mins / 60);
  if (hours < 24) return `${hours} 小时前`;
  return `${Math.round(hours / 24)} 天前`;
}

export function WelcomePane({
  mode, onApplyPair, onDrop, onPickLeft, onPickRight,
}: {
  mode: WorkbenchMode;
  onApplyPair(left: string, right: string): void;
  onDrop(side: Side, path: string): void;
  onPickLeft(): void;
  onPickRight(): void;
}) {
  const kind: RecentKind = mode === 'directory' ? 'dir' : 'file';
  const [recent, setRecent] = useState<RecentPair[]>(() => loadRecent(kind));
  const noun = mode === 'directory' ? '文件夹' : 'PNG 文件';

  const dropHandler = (side: Side) => (e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    const p = (file as unknown as { path?: string })?.path;
    if (p) onDrop(side, p);
  };

  return (
    <div className="welcome2">
      <div className="welcome2__title">PNG Compare</div>
      <p className="welcome2__noun">拖入{noun}，或点击浏览</p>
      <div className="welcome2__slots">
        {(['left', 'right'] as const).map((side) => (
          <div key={side} className="welcome2__slot"
            onDragOver={(e) => e.preventDefault()} onDrop={dropHandler(side)}>
            <span>{side === 'left' ? '左侧' : '右侧'}</span>
            <button type="button" onClick={side === 'left' ? onPickLeft : onPickRight}>浏览</button>
          </div>
        ))}
      </div>

      {recent.length > 0 && (
        <div className="welcome2__recent">
          <div className="welcome2__recent-title">最近使用</div>
          {recent.map((p) => (
            <div key={`${p.left}|${p.right}`} className="welcome2__recent-row">
              <button type="button" className="welcome2__recent-main"
                title={`${p.left}\n${p.right}`}
                onClick={() => onApplyPair(p.left, p.right)}>
                <span className="welcome2__recent-name">{basename(p.left)} ⇄ {basename(p.right)}</span>
                <span className="welcome2__recent-time">{relTime(p.lastUsed)}</span>
              </button>
              <button type="button" className="welcome2__recent-del" aria-label="删除该记录"
                onClick={() => { removeRecent(kind, p.left, p.right); setRecent(loadRecent(kind)); }}>×</button>
            </div>
          ))}
        </div>
      )}

      <div className="welcome2__hint">
        <kbd>Ctrl+O</kbd> 选左 · <kbd>Ctrl+Shift+O</kbd> 选右 · <kbd>↑</kbd>/<kbd>↓</kbd> 列表穿梭 · <kbd>n</kbd>/<kbd>p</kbd> 跳差异 · <kbd>1/2/3</kbd> 视图 · <kbd>F</kbd> 仅看不同 · <kbd>D</kbd> 高亮
      </div>
    </div>
  );
}
