// frontend/src/App.tsx
import { useCallback, useEffect, useMemo, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { workbenchApi } from './lib/api';
import { ImageSplit, SoloImage } from './components/ImageViews';
import { Sidebar } from './components/Sidebar';
import { UnifiedTree } from './components/UnifiedTree';
import type { FocusRequest } from './components/UnifiedTree';
import { DiffRail } from './components/DiffRail';
import { WelcomePane } from './components/WelcomePane';
import { useWorkbench } from './features/workbench/useWorkbench';
import { SelectionBar } from './components/SelectionBar';
import { buildMirrorRows } from './lib/treeModel';
import { buildDiffEntries } from './lib/diffList';
import type { BatchListItem } from './lib/types';

const win = getCurrentWindow();
const RAIL_AUTO_COLLAPSE_WIDTH = 1000;

async function pickPath(directory: boolean): Promise<string> {
  if (directory) {
    const selected = await workbenchApi.pickFolder?.();
    return selected ?? '';
  }
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'PNG', extensions: ['png'] }],
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

  useEffect(() => {
    if (wb.leftInput || wb.rightInput) void wb.runAuto();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wb.leftInput, wb.rightInput, wb.mode]);

  // 窗口过窄自动收起差异栏；手动操作（按钮/快捷键）后本会话内尊重手动状态
  useEffect(() => {
    const onResize = () => {
      if (wb.railManualRef.current) return;
      wb.setRailCollapsed(window.innerWidth < RAIL_AUTO_COLLAPSE_WIDTH);
    };
    onResize();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, [wb.railManualRef, wb.setRailCollapsed]);

  // ===== 行模型单处计算，树与差异栏共享（设计 §5） =====
  const pairRows = useMemo(
    () => (wb.pairResult
      ? buildMirrorRows(wb.pairResult.left.metadata, wb.pairResult.right.metadata, wb.pairResult.diff_root)
      : null),
    [wb.pairResult],
  );
  const soloRows = useMemo(
    () => (wb.soloResult?.metadata !== undefined && wb.soloResult?.metadata !== null
      ? buildMirrorRows(
          wb.soloSide === 'right' ? null : wb.soloResult.metadata,
          wb.soloSide === 'right' ? wb.soloResult.metadata : null,
          null,
        )
      : null),
    [wb.soloResult, wb.soloSide],
  );
  const diffEntries = useMemo(() => (pairRows ? buildDiffEntries(pairRows) : []), [pairRows]);

  // ===== 差异跳转（n/p 与差异栏点击共用） =====
  const [focusRequest, setFocusRequest] = useState<FocusRequest | null>(null);
  // 切换文件对后清掉焦点请求，避免 n/p 以上一对的路径为锚点“幽灵推进”
  useEffect(() => { setFocusRequest(null); }, [wb.pairResult]);
  const jumpTo = (path: string) =>
    setFocusRequest((cur) => ({ path, seq: (cur?.seq ?? 0) + 1 }));
  useEffect(() => {
    const onJump = (e: Event) => {
      const dir = (e as CustomEvent<number>).detail;
      if (diffEntries.length === 0) return;
      setFocusRequest((cur) => {
        const curIdx = cur ? diffEntries.findIndex((d) => d.path === cur.path) : -1;
        // 无锚点（首次跳转/换对后）：n 从第一处开始，p 从最后一处开始
        const next = curIdx < 0
          ? (dir === 1 ? 0 : diffEntries.length - 1)
          : (curIdx + dir + diffEntries.length) % diffEntries.length;
        return { path: diffEntries[next].path, seq: (cur?.seq ?? 0) + 1 };
      });
    };
    document.addEventListener('wb:diffJump', onJump);
    return () => document.removeEventListener('wb:diffJump', onJump);
  }, [diffEntries]);

  const handleSelect = useCallback(
    (item: BatchListItem) => { void wb.selectItem(item); },
    [wb.selectItem],
  );

  const showSidebar = isDir && (wb.directorySummary !== null || wb.isLoading) && !wb.sidebarCollapsed;
  const showRail = wb.view === 'mirror' && wb.pairResult !== null;
  const showWelcome = wb.view === 'welcome' && !(isDir && wb.isLoading);

  return (
    <div className="app-shell">
      <header className="topbar" data-tauri-drag-region>
        <div className="topbar-left" data-tauri-drag-region>
          <img className="brand-icon" src="/app-icon.png" alt="" draggable={false} data-tauri-drag-region />
          <span className="brand" data-tauri-drag-region>PNG Compare</span>
          <div className="topbar-vsep" data-tauri-drag-region />
          <div className="mode-toggle" role="group" aria-label="模式">
            <button type="button" className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
              onClick={() => wb.setMode('single')}>单文件</button>
            <button type="button" className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
              onClick={() => wb.setMode('directory')}>目录</button>
          </div>
          {isDir && wb.directorySummary && (
            <button type="button" className="topbar-collapse" title="收起/展开侧栏 (Ctrl+B)"
              onClick={wb.toggleSidebarCollapsed}>{wb.sidebarCollapsed ? '⇤' : '⇥'}</button>
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

      <SelectionBar
        mode={wb.mode}
        leftInput={wb.leftInput}
        rightInput={wb.rightInput}
        onPickLeft={() => void handlePickLeft()}
        onPickRight={() => void handlePickRight()}
        onPastePath={(side, p) => wb.tryDropPath(side, p)}
        onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }}
        onClear={(side) => wb.clearSide(side)}
        onDrop={(side, p) => wb.tryDropPath(side, p)}
      />

      {wb.error && <div className="banner banner--error">{wb.error}</div>}

      <div className="shell-body">
        {showSidebar && (
          <Sidebar
            summary={wb.directorySummary} filteredItems={wb.filteredItems}
            activeFilter={wb.activeFilter} searchQuery={wb.searchQuery} sortKey={wb.sortKey}
            selectedItemId={wb.selectedItemId} isLoading={wb.isLoading} scanProgress={wb.scanProgress}
            onFilter={wb.setActiveFilter} onSearch={wb.setSearchQuery} onSort={wb.setSortKey}
            onSelect={handleSelect}
            onCancelScan={() => void wb.cancelScan()}
          />
        )}

        <main className="center">
          {showWelcome && (
            <WelcomePane mode={wb.mode}
              onApplyPair={(l, r) => { wb.setLeftInput(l); wb.setRightInput(r); }} />
          )}

          {((wb.view === 'mirror' && wb.pairResult !== null) || (wb.view === 'solo' && wb.soloResult !== null)) && (
            <DetailHeader wb={wb} diffCount={diffEntries.length} />
          )}

          {wb.view === 'solo' && wb.soloResult && (
            wb.viewMode === 'image' ? (
              // 图片视图不依赖元数据：无元数据的 PNG 也能看图
              <SoloImage path={wb.soloResult.file_path} name={wb.soloResult.file_name} />
            ) : wb.viewMode === 'json' ? (
              wb.soloResult.raw_json
                ? <RawJsonSplit left={wb.soloSide === 'left' ? wb.soloResult.raw_json : null}
                    right={wb.soloSide === 'right' ? wb.soloResult.raw_json : null} solo={wb.soloSide} />
                : <div className="banner banner--error">该文件不含嵌入式元数据。</div>
            ) : soloRows ? (
              <UnifiedTree rows={soloRows} solo={wb.soloSide} highlight={false} onlyDiff={false}
                leftLabel={wb.soloSide === 'left' ? wb.soloResult.file_name : ''}
                rightLabel={wb.soloSide === 'right' ? wb.soloResult.file_name : ''}
                focusRequest={null} />
            ) : (
              <div className="banner banner--error">该文件不含嵌入式元数据。</div>
            )
          )}

          {wb.view === 'mirror' && wb.pairResult && pairRows && (
            <>
              {wb.viewMode === 'tree' && (
                <UnifiedTree rows={pairRows} solo={null}
                  highlight={wb.diffHighlight} onlyDiff={wb.onlyDiff}
                  leftLabel={wb.pairResult.left.file_name} rightLabel={wb.pairResult.right.file_name}
                  focusRequest={focusRequest} />
              )}
              {wb.viewMode === 'json' && (
                <RawJsonSplit left={wb.pairResult.left.raw_json} right={wb.pairResult.right.raw_json} solo={null} />
              )}
              {wb.viewMode === 'image' && (
                <ImageSplit
                  leftPath={wb.pairResult.left.file_path} rightPath={wb.pairResult.right.file_path}
                  leftName={wb.pairResult.left.file_name} rightName={wb.pairResult.right.file_name} />
              )}
            </>
          )}

          {wb.view === 'error' && wb.errorItem && <ErrorCard item={wb.errorItem} />}
        </main>

        {showRail && (
          <DiffRail entries={diffEntries} onJump={jumpTo}
            collapsed={wb.railCollapsed}
            onToggle={wb.toggleRailCollapsed} />
        )}
      </div>

      {wb.toast && <div className="toast">{wb.toast}</div>}
    </div>
  );
}

function DetailHeader({ wb, diffCount }: { wb: ReturnType<typeof useWorkbench>; diffCount: number }) {
  const name = wb.view === 'mirror'
    ? wb.pairResult?.left.file_name
    : wb.soloResult?.file_name;
  return (
    <div className="detail-head">
      <div className="detail-head__seg" role="group" aria-label="视图模式">
        <button data-active={wb.viewMode === 'tree'} onClick={() => wb.setViewMode('tree')}>树</button>
        <button data-active={wb.viewMode === 'json'} onClick={() => wb.setViewMode('json')}>JSON</button>
        <button data-active={wb.viewMode === 'image'} onClick={() => wb.setViewMode('image')}>图片</button>
      </div>
      {wb.view === 'mirror' && (
        <>
          <button className="detail-head__btn" data-active={wb.onlyDiff} onClick={wb.toggleOnlyDiff}>仅看不同</button>
          <button className="detail-head__btn" data-active={wb.diffHighlight} onClick={wb.toggleDiffHighlight}>高亮</button>
        </>
      )}
      <span className="detail-head__name" title={name ?? ''}>
        {wb.view === 'solo' && `仅查看${wb.soloSide === 'left' ? '左' : '右'} · `}{name}
      </span>
      <span className="detail-head__spacer" />
      {wb.view === 'mirror' && (
        <span className="detail-head__hint">{diffCount > 0 ? `n/p 跳差异 · ${diffCount} 处` : '无差异'}</span>
      )}
    </div>
  );
}

function ErrorCard({ item }: { item: import('./lib/types').BatchListItem }) {
  const target = item.left_path ?? item.right_path ?? item.label;
  return (
    <div className="error-card">
      <div className="error-card__title">无法解析此文件</div>
      <div className="error-card__path">{item.label}</div>
      {item.message && <div className="error-card__msg">{item.message}</div>}
      <button type="button" className="error-card__open"
        onClick={() => void openPath(target).catch(() => { /* 文件可能已被移动/删除：静默 */ })}>打开文件 ↗</button>
    </div>
  );
}

function RawJsonSplit({ left, right, solo }: { left: string | null; right: string | null; solo: 'left' | 'right' | null }) {
  const leftText = useMemo(() => format(left), [left]);
  const rightText = useMemo(() => format(right), [right]);
  if (solo === 'left') return <pre className="raw-json raw-json--solo">{leftText}</pre>;
  if (solo === 'right') return <pre className="raw-json raw-json--solo">{rightText}</pre>;
  return (
    <div className="mirror-grid">
      <pre className="raw-json">{leftText}</pre>
      <pre className="raw-json">{rightText}</pre>
    </div>
  );
}

function format(raw: string | null): string {
  if (!raw) return '— 无 JSON —';
  try { return JSON.stringify(JSON.parse(raw), null, 2); } catch { return raw; }
}

