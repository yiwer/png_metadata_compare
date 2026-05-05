// frontend/src/App.tsx
import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { SlotBar } from './components/SlotBar';
import { SoloTree } from './components/SoloTree';
import { MirrorTree } from './components/MirrorTree';
import { DirectoryList } from './components/DirectoryList';
import { useWorkbench } from './features/workbench/useWorkbench';

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

  const showModeToggle = wb.view === 'welcome' || wb.view === 'directory-overview' || wb.directoryContext === null;
  const showSlotBar = wb.view !== 'mirror' || !wb.directoryContext;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="topbar-left" data-tauri-drag-region>
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

          {wb.directoryContext && wb.view !== 'directory-overview' && (
            <>
              <div className="topbar-vsep" />
              <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
                ← 返回目录
              </button>
            </>
          )}
        </div>

        <div className="topbar-center" data-tauri-drag-region>
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

        <div className="topbar-right" data-tauri-drag-region>
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

        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'tree' && (
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
        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'json' && (
          <RawJsonSplit left={wb.pairResult.left.raw_json} right={wb.pairResult.right.raw_json} />
        )}
        {wb.view === 'mirror' && wb.pairResult && wb.viewMode === 'image' && (
          <ImageSplit
            leftPath={wb.pairResult.left.file_path}
            rightPath={wb.pairResult.right.file_path}
            leftName={wb.pairResult.left.file_name}
            rightName={wb.pairResult.right.file_name}
          />
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
      <div className="controlbar__seg" role="group" aria-label="视图模式">
        <button data-active={wb.viewMode === 'tree'} onClick={() => wb.setViewMode('tree')}>树</button>
        <button data-active={wb.viewMode === 'json'} onClick={() => wb.setViewMode('json')}>JSON</button>
        <button data-active={wb.viewMode === 'image'} onClick={() => wb.setViewMode('image')}>图片</button>
      </div>
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
      {wb.view === 'solo' && (
        <span className="controlbar__summary">仅查看 {wb.soloSide === 'left' ? '左' : '右'} · {wb.soloResult?.file_name}</span>
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
  return (
    <div className="mirror-grid">
      <ImagePane path={leftPath} name={leftName} />
      <ImagePane path={rightPath} name={rightName} />
    </div>
  );
}

function ImagePane({ path, name }: { path: string; name: string }) {
  const url = `asset://localhost/${path.replace(/\\/g, '/').split('/').map(encodeURIComponent).join('/')}`;
  return (
    <div style={{ padding: 12, display: 'flex', flexDirection: 'column', gap: 8 }}>
      <img src={url} alt={name} style={{ maxWidth: '100%', maxHeight: 'calc(100vh - 200px)', objectFit: 'contain' }}
           onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
      <button type="button" className="controlbar__btn" onClick={() => void openPath(path)}>打开原文件 ↗</button>
    </div>
  );
}
