// frontend/src/App.tsx
import { useEffect, useMemo, useRef, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { SlotBar } from './components/SlotBar';
import { SoloTree } from './components/SoloTree';
import { MirrorTree } from './components/MirrorTree';
import { DirectoryList } from './components/DirectoryList';
import { useWorkbench } from './features/workbench/useWorkbench';
import { buildMirrorRows, hasDiffDeep } from './lib/treeModel';
import type { MirrorRow } from './lib/treeModel';
import type { DiffNode, DiffStatus, JsonValue, ScanProgress } from './lib/types';

const win = getCurrentWindow();

async function pickPath(directory: boolean): Promise<string> {
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: 'PNG', extensions: ['png'] }],
  });
  return typeof selected === 'string' ? selected : '';
}

export default function App() {
  const wb = useWorkbench();
  const isDir = wb.mode === 'directory';

  const handlePickLeft = async () => {
    const p = await pickPath(isDir);
    if (p) wb.setLeftInput(p);
  };
  const handlePickRight = async () => {
    const p = await pickPath(isDir);
    if (p) wb.setRightInput(p);
  };

  // Listen to keyboard-driven pick events from useWorkbench
  useEffect(() => {
    const onL = () => void handlePickLeft();
    const onR = () => void handlePickRight();
    document.addEventListener('wb:pickLeft', onL);
    document.addEventListener('wb:pickRight', onR);
    return () => {
      document.removeEventListener('wb:pickLeft', onL);
      document.removeEventListener('wb:pickRight', onR);
    };
  });

  // Auto-run on input change
  useEffect(() => {
    if (wb.leftInput || wb.rightInput) void wb.runAuto();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wb.leftInput, wb.rightInput, wb.mode]);

  const showModeToggle = wb.view === 'welcome' || wb.view === 'directory-overview' || !wb.inDirectorySubview;
  const showSlotBar = wb.view !== 'mirror' || !wb.inDirectorySubview;

  return (
    <div className="app-shell">
      <header className="topbar" data-tauri-drag-region>
        <div className="topbar-left">
          <img className="brand-icon" src="/app-icon.png" alt="" draggable={false} />
          <span className="brand">PNG ⌁ Compare</span>

          {showModeToggle && (
            <>
              <div className="topbar-vsep" />
              <div className="mode-toggle" role="group" aria-label="模式">
                <button
                  type="button"
                  className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
                  onClick={() => wb.setMode('single')}
                >单文件</button>
                <button
                  type="button"
                  className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
                  onClick={() => wb.setMode('directory')}
                >目录</button>
              </div>
            </>
          )}

          {wb.inDirectorySubview && wb.view !== 'directory-overview' && (
            <>
              <div className="topbar-vsep" />
              <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
                ← 返回目录
              </button>
            </>
          )}
        </div>

        <div className="topbar-center">
          {wb.view === 'mirror' && wb.pairResult && (
            <span>{wb.pairResult.left.file_name}</span>
          )}
          {wb.view === 'solo' && wb.soloResult && (
            <span>{wb.soloResult.file_name}</span>
          )}
          {wb.directoryContext && (
            <span>{wb.directoryContext.index} / {wb.directoryContext.totalDifferent} 处不一致</span>
          )}
        </div>

        <div className="topbar-right">
          <div className="win-controls">
            <button type="button" className="win-btn" onClick={() => void win.minimize()} aria-label="最小化">─</button>
            <button type="button" className="win-btn" onClick={() => void win.toggleMaximize()} aria-label="最大化">□</button>
            <button type="button" className="win-btn win-btn--close" onClick={() => void win.close()} aria-label="关闭">✕</button>
          </div>
        </div>
      </header>

      {showSlotBar && (
        <SlotBar
          mode={wb.mode}
          leftValue={wb.leftInput}
          rightValue={wb.rightInput}
          collapsed={wb.slotBarCollapsed}
          onPickLeft={() => void handlePickLeft()}
          onPickRight={() => void handlePickRight()}
          onLeftChange={(p) => wb.tryDropPath('left', p)}
          onRightChange={(p) => wb.tryDropPath('right', p)}
          onToggleCollapsed={wb.toggleSlotBarCollapsed}
        />
      )}

      <ControlBar wb={wb} />

      {wb.error && <div className="banner banner--error">{wb.error}</div>}

      {wb.isLoading && isDir && <ScanProgressBar progress={wb.scanProgress} />}

      <main style={{ overflow: 'hidden' }}>
        {wb.view === 'welcome' && <Welcome mode={wb.mode} />}

        {wb.view === 'solo' && wb.soloResult?.metadata && (
          <SoloTreeFrame name={wb.soloResult.file_name} side={wb.soloSide!}>
            <SoloTree value={wb.soloResult.metadata} />
          </SoloTreeFrame>
        )}
        {wb.view === 'solo' && wb.soloResult && !wb.soloResult.metadata && (
          <div className="banner banner--error">该文件不含嵌入式元数据。</div>
        )}

        {wb.view === 'mirror' && wb.pairResult && (
          <div className="mirror-view">
            <div className="mirror-view__main">
              {wb.viewMode === 'tree' && (
                <MirrorTree
                  left={wb.pairResult.left.metadata}
                  right={wb.pairResult.right.metadata}
                  diffRoot={wb.pairResult.diff_root}
                  highlight={wb.diffHighlight}
                  onlyDiff={wb.onlyDiff}
                  leftLabel={wb.pairResult.left.file_name}
                  rightLabel={wb.pairResult.right.file_name}
                />
              )}
              {wb.viewMode === 'json' && (
                <RawJsonSplit left={wb.pairResult.left.raw_json} right={wb.pairResult.right.raw_json} />
              )}
              {wb.viewMode === 'image' && (
                <ImageSplit
                  leftPath={wb.pairResult.left.file_path}
                  rightPath={wb.pairResult.right.file_path}
                  leftName={wb.pairResult.left.file_name}
                  rightName={wb.pairResult.right.file_name}
                />
              )}
            </div>
            <DiffReport
              left={wb.pairResult.left.metadata}
              right={wb.pairResult.right.metadata}
              diffRoot={wb.pairResult.diff_root}
            />
          </div>
        )}

        {wb.view === 'directory-overview' && wb.directorySummary && (
          <DirectoryList
            summary={wb.directorySummary}
            filteredItems={wb.filteredItems}
            activeFilter={wb.activeFilter}
            onFilter={wb.setActiveFilter}
            onSelect={(item) => void wb.navigateToPair(item)}
          />
        )}
      </main>

      {wb.toast && <div className="toast">{wb.toast}</div>}
    </div>
  );
}

function ScanProgressBar({ progress }: { progress: ScanProgress | null }) {
  const comparing = progress?.stage === 'comparing' && progress.total > 0;
  const percent = comparing ? Math.round((progress.done / progress.total) * 100) : 0;
  return (
    <div className="scan-progress" role="status" aria-live="polite">
      <span className="scan-progress__label">
        {comparing
          ? `正在比对 ${progress.done} / ${progress.total} 对文件 (${percent}%)`
          : '正在扫描目录…'}
      </span>
      <div className="scan-progress__track">
        <div
          className={`scan-progress__fill${comparing ? '' : ' scan-progress__fill--indeterminate'}`}
          style={comparing ? { width: `${percent}%` } : undefined}
        />
      </div>
    </div>
  );
}

function Welcome({ mode }: { mode: 'single' | 'directory' }) {
  return (
    <div className="welcome">
      <div className="welcome__title">PNG ⌁ Compare</div>
      <div className="welcome__hint">
        拖入 {mode === 'single' ? 'PNG 文件' : '文件夹'}（左右各一个），或按
        <kbd>Ctrl+O</kbd> / <kbd>Ctrl+Shift+O</kbd> 选择
      </div>
      <div className="welcome__hint">
        快捷键：<kbd>Ctrl+Enter</kbd> 重新分析 · <kbd>1</kbd>/<kbd>2</kbd>/<kbd>3</kbd> 切换视图 · <kbd>D</kbd> 切换差异高亮
      </div>
    </div>
  );
}

function ControlBar({ wb }: { wb: ReturnType<typeof useWorkbench> }) {
  if (wb.view === 'welcome' || wb.view === 'directory-overview') return null;
  const total = wb.pairResult ? wb.pairResult.diff_summary : null;
  return (
    <div className="controlbar">
      {wb.view === 'mirror' && (
        <div className="controlbar__seg" role="group" aria-label="视图模式">
          <button data-active={wb.viewMode === 'tree'} onClick={() => wb.setViewMode('tree')}>树</button>
          <button data-active={wb.viewMode === 'json'} onClick={() => wb.setViewMode('json')}>JSON</button>
          <button data-active={wb.viewMode === 'image'} onClick={() => wb.setViewMode('image')}>图片</button>
        </div>
      )}
      {wb.view === 'mirror' && (
        <>
          <button className="controlbar__btn" data-active={wb.diffHighlight} onClick={wb.toggleDiffHighlight}>高亮差异</button>
          <button className="controlbar__btn" data-active={wb.onlyDiff} onClick={wb.toggleOnlyDiff}>仅看不同</button>
          <span className="controlbar__spacer" />
          {total && (
            <span className="controlbar__summary">
              {total.modified} 处不同 · {total.added} 仅右 · {total.removed} 仅左 · {total.reordered} 顺序不同
            </span>
          )}
        </>
      )}
    </div>
  );
}

function SoloTreeFrame({ side, name, children }: { side: 'left' | 'right'; name: string; children: React.ReactNode }) {
  return (
    <>
      <div className="solo-status">仅查看 {side === 'left' ? '左' : '右'} · {name}</div>
      {children}
    </>
  );
}

function RawJsonSplit({ left, right }: { left: string | null; right: string | null }) {
  return (
    <div className="mirror-grid">
      <pre className="raw-json">{format(left)}</pre>
      <pre className="raw-json">{format(right)}</pre>
    </div>
  );
}

function format(raw: string | null): string {
  if (!raw) return '— 无 JSON —';
  try { return JSON.stringify(JSON.parse(raw), null, 2); } catch { return raw; }
}

function ImageSplit({ leftPath, rightPath, leftName, rightName }: { leftPath: string; rightPath: string; leftName: string; rightName: string; }) {
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
  const reset = () => { setZoom(1); setOffset({ x: 0, y: 0 }); };

  const transform = `translate(${offset.x}px, ${offset.y}px) scale(${zoom})`;
  const dragging = dragRef.current !== null;

  return (
    <div className="image-split">
      <div className="image-split__toolbar">
        <button type="button" className="controlbar__btn" onClick={() => setZoom((z) => Math.max(0.1, z / 1.25))}>−</button>
        <span className="controlbar__summary" style={{ minWidth: 56, textAlign: 'center' }}>{Math.round(zoom * 100)}%</span>
        <button type="button" className="controlbar__btn" onClick={() => setZoom((z) => Math.min(20, z * 1.25))}>＋</button>
        <button type="button" className="controlbar__btn" onClick={reset}>重置</button>
        <span className="controlbar__spacer" />
        <span className="controlbar__summary">滚轮缩放 · 拖拽平移（左右同步）</span>
      </div>
      <div
        className={`image-split__panes${dragging ? ' image-split__panes--dragging' : ''}`}
        onWheel={onWheel}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={endDrag}
        onMouseLeave={endDrag}
      >
        <ImagePane path={leftPath} name={leftName} transform={transform} />
        <ImagePane path={rightPath} name={rightName} transform={transform} />
      </div>
    </div>
  );
}

function ImagePane({ path, name, transform }: { path: string; name: string; transform: string }) {
  const url = convertFileSrc(path);
  return (
    <div className="image-pane">
      <div className="image-pane__viewport">
        <img
          className="image-pane__img"
          src={url}
          alt={name}
          draggable={false}
          style={{ transform, transformOrigin: 'center center' }}
          onError={(e) => { (e.currentTarget as HTMLImageElement).dataset.broken = 'true'; }}
        />
      </div>
      <div className="image-pane__bar">
        <span className="image-pane__name">{name}</span>
        <button type="button" className="controlbar__btn" onClick={() => void openPath(path)}>打开原文件 ↗</button>
      </div>
    </div>
  );
}

// ============================================================
// Diff report (bottom strip on mirror view)
// ============================================================

const STATUS_LABEL: Record<DiffStatus, string> = {
  unchanged: '未变',
  modified: '改',
  added: '仅右',
  removed: '仅左',
  reordered: '顺序',
  error: '错',
};

const STATUS_TO_BADGE: Record<DiffStatus, string> = {
  unchanged: 'badge--neu',
  modified: 'badge--mod',
  added: 'badge--add',
  removed: 'badge--rem',
  reordered: 'badge--neu',
  error: 'badge--err',
};

interface DiffCounts {
  modified: number;
  added: number;
  removed: number;
  total: number;
}

function tallyDiff(rows: MirrorRow[]): DiffCounts {
  const counts: DiffCounts = { modified: 0, added: 0, removed: 0, total: 0 };
  function walk(rs: MirrorRow[]) {
    for (const r of rs) {
      if (r.kind === 'leaf') {
        if (r.status === 'modified') { counts.modified++; counts.total++; }
        else if (r.status === 'added') { counts.added++; counts.total++; }
        else if (r.status === 'removed') { counts.removed++; counts.total++; }
        continue;
      }
      // Array-item Added/Removed counts as a single unit (no nested descent).
      const isItemAddRem =
        r.variant === 'array-item' && (r.status === 'added' || r.status === 'removed');
      if (isItemAddRem) {
        if (r.status === 'added') counts.added++;
        else counts.removed++;
        counts.total++;
        continue;
      }
      if (r.children) walk(r.children);
    }
  }
  walk(rows);
  return counts;
}

function DiffReport({
  left, right, diffRoot,
}: {
  left: JsonValue | null;
  right: JsonValue | null;
  diffRoot: DiffNode | null;
}) {
  const rows = useMemo(() => buildMirrorRows(left, right, diffRoot), [left, right, diffRoot]);
  const counts = useMemo(() => tallyDiff(rows), [rows]);
  const [collapsed, setCollapsed] = useState(false);

  return (
    <div className={`diff-report${collapsed ? ' diff-report--collapsed' : ''}`}>
      <button
        type="button"
        className="diff-report__head"
        onClick={() => setCollapsed((c) => !c)}
        aria-expanded={!collapsed}
      >
        <span className="diff-report__title">
          <span className="diff-report__caret">{collapsed ? '▶' : '▼'}</span>
          差异汇总
        </span>
        <span className="diff-report__counts">
          {counts.total === 0 ? (
            <span className="controlbar__summary">两份元数据完全一致</span>
          ) : (
            <>
              {counts.modified > 0 && <span className="badge badge--mod">{counts.modified} 改</span>}
              {counts.removed > 0 && <span className="badge badge--rem">{counts.removed} 仅左</span>}
              {counts.added > 0 && <span className="badge badge--add">{counts.added} 仅右</span>}
              <span className="diff-report__total">共 {counts.total} 项</span>
            </>
          )}
        </span>
      </button>
      {!collapsed && counts.total > 0 && (
        <div className="diff-report__body">
          <DiffTree rows={rows} />
        </div>
      )}
    </div>
  );
}

function DiffTree({ rows }: { rows: MirrorRow[] }) {
  return (
    <>
      {rows.map((row) => (
        <DiffTreeNode key={row.path || 'root'} row={row} level={0} />
      ))}
    </>
  );
}

function DiffTreeNode({ row, level }: { row: MirrorRow; level: number }) {
  if (row.status === 'unchanged' && !hasDiffDeep(row)) return null;

  // root group: render children only, no header
  if (row.variant === 'object-root') {
    return (
      <>
        {row.children?.map((c) => (
          <DiffTreeNode key={c.path} row={c} level={level} />
        ))}
      </>
    );
  }

  if (row.kind === 'leaf') {
    if (row.status === 'unchanged') return null;
    return <DiffLeaf row={row} level={level} />;
  }

  // Array-item Added/Removed: render the colored header without descending into fields.
  const isItemAddRem =
    row.variant === 'array-item' && (row.status === 'added' || row.status === 'removed');

  if (isItemAddRem) {
    return <DiffGroupHead row={row} level={level} mode="leaf" />;
  }

  const changedChildren = row.children?.filter((c) => c.status !== 'unchanged' || hasDiffDeep(c)) ?? [];
  if (changedChildren.length === 0) return null;
  return (
    <>
      <DiffGroupHead row={row} level={level} mode="branch" />
      {changedChildren.map((c) => (
        <DiffTreeNode key={c.path} row={c} level={level + 1} />
      ))}
    </>
  );
}

function DiffGroupHead({
  row, level, mode,
}: { row: MirrorRow; level: number; mode: 'branch' | 'leaf' }) {
  const isAddRem = row.status === 'added' || row.status === 'removed';
  // For an array-item that exists exclusively on one side, color it green (added).
  const colorStatus =
    row.variant === 'array-item' && isAddRem ? 'added' : row.status;
  const label =
    row.status === 'added' ? '仅右'
    : row.status === 'removed' ? '仅左'
    : row.status === 'modified' ? '改'
    : row.status === 'error' ? '错'
    : '';
  return (
    <div
      className={`diff-tree__group diff-tree__group--${colorStatus} diff-tree__group--${mode}`}
      style={{ paddingLeft: 8 + level * 14 }}
    >
      {label && <span className={`badge ${STATUS_TO_BADGE[colorStatus]}`}>{label}</span>}
      <span className="diff-tree__label">{row.label}</span>
      {row.variant === 'array' && typeof row.count === 'number' && (
        <span className="diff-tree__count">({row.count} 项)</span>
      )}
    </div>
  );
}

function DiffLeaf({ row, level }: { row: MirrorRow; level: number }) {
  return (
    <div
      className={`diff-tree__leaf diff-tree__leaf--${row.status}`}
      style={{ paddingLeft: 8 + level * 14 }}
    >
      <span className={`badge ${STATUS_TO_BADGE[row.status]}`}>{STATUS_LABEL[row.status]}</span>
      <span className="diff-tree__label">{row.label}</span>
      <span className="diff-tree__values">
        {row.status === 'modified' && (
          <>
            <span className="diff-tree__old">{row.leftValue}</span>
            <span className="diff-tree__arrow">→</span>
            <span className="diff-tree__new">{row.rightValue}</span>
          </>
        )}
        {row.status === 'removed' && <span className="diff-tree__old">{row.leftValue}</span>}
        {row.status === 'added' && <span className="diff-tree__new">{row.rightValue}</span>}
      </span>
    </div>
  );
}
