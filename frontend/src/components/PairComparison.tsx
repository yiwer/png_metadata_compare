// frontend/src/components/PairComparison.tsx
import { openPath } from '@tauri-apps/plugin-opener';
import { DiffStrip } from './DiffStrip';
import { EmptyState } from './EmptyState';
import { MetadataTree } from './MetadataTree';
import { ViewModeStrip } from './ViewModeStrip';
import { buildDiffPathMap, totalDiffCount } from '../lib/diffUtils';
import type { DiffStatus, PairInspection } from '../lib/types';
import type { ViewMode } from '../features/workbench/useWorkbench';

export function PairComparison({
  mode,
  leftInput,
  rightInput,
  pairResult,
  viewMode,
  diffHighlight,
  isLoading,
  error,
  onLeftInput,
  onRightInput,
  onCompare,
  onPickLeft,
  onPickRight,
  onViewMode,
  onToggleDiff,
}: {
  mode: 'single' | 'directory';
  leftInput: string;
  rightInput: string;
  pairResult: PairInspection | null;
  viewMode: ViewMode;
  diffHighlight: boolean;
  isLoading: boolean;
  error: string | null;
  onLeftInput(v: string): void;
  onRightInput(v: string): void;
  onCompare(): void;
  onPickLeft(): void;
  onPickRight(): void;
  onViewMode(mode: ViewMode): void;
  onToggleDiff(): void;
}) {
  const diffPathMap = pairResult ? buildDiffPathMap(pairResult.diff_root) : undefined;
  const changeCount = pairResult ? totalDiffCount(pairResult.diff_summary) : 0;
  const leftLabel = pairResult?.left.file_name ?? 'Left';
  const rightLabel = pairResult?.right.file_name ?? 'Right';

  return (
    <>
      <div className="toolbar">
        <div className="path-group">
          <span className="path-label">{mode === 'directory' ? 'Left File' : 'Left PNG'}</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={leftInput}
              onChange={(e) => onLeftInput(e.target.value)}
              placeholder="Path to left PNG…"
            />
            <button type="button" className="choose-btn" onClick={onPickLeft}>Choose</button>
          </div>
        </div>
        <span className="vs-divider">vs</span>
        <div className="path-group">
          <span className="path-label">{mode === 'directory' ? 'Right File' : 'Right PNG'}</span>
          <div className="path-input-row">
            <input
              className="path-input"
              value={rightInput}
              onChange={(e) => onRightInput(e.target.value)}
              placeholder="Path to right PNG…"
            />
            <button type="button" className="choose-btn" onClick={onPickRight}>Choose</button>
          </div>
        </div>
        <div className="cta-wrap">
          <div className="cta-outer">
            <button
              type="button"
              className="cta-btn"
              disabled={isLoading || !leftInput || !rightInput}
              onClick={onCompare}
            >
              {isLoading ? 'Comparing…' : 'Compare'}
            </button>
          </div>
        </div>
      </div>

      {error && (
        <div className="status-banner status-banner--error">{error}</div>
      )}

      <ViewModeStrip
        viewMode={viewMode}
        diffHighlight={diffHighlight}
        changeCount={changeCount}
        onViewMode={onViewMode}
        onToggleDiff={onToggleDiff}
      />

      <div className="split-body">
        <div className="split-panel split-panel--left">
          <div className="panel-header">{leftLabel}</div>
          <SplitPanelContent
            side="left"
            pairResult={pairResult}
            viewMode={viewMode}
            diffPathMap={diffHighlight ? diffPathMap : undefined}
          />
        </div>

        <DiffStrip root={pairResult?.diff_root ?? null} />

        <div className="split-panel split-panel--right">
          <div className="panel-header">{rightLabel}</div>
          <SplitPanelContent
            side="right"
            pairResult={pairResult}
            viewMode={viewMode}
            diffPathMap={diffHighlight ? diffPathMap : undefined}
          />
        </div>
      </div>
    </>
  );
}

function SplitPanelContent({
  side,
  pairResult,
  viewMode,
  diffPathMap,
}: {
  side: 'left' | 'right';
  pairResult: PairInspection | null;
  viewMode: ViewMode;
  diffPathMap?: Map<string, DiffStatus>;
}) {
  if (!pairResult) {
    return (
      <EmptyState
        title="Choose inputs and compare"
        body="Results appear here after you run a comparison."
      />
    );
  }

  const sideData = side === 'left' ? pairResult.left : pairResult.right;

  if (viewMode === 'tree') {
    if (!sideData.metadata) {
      return <EmptyState title="No metadata" body="This file has no embedded metadata." />;
    }
    return (
      <MetadataTree
        value={sideData.metadata}
        diffPathMap={diffPathMap}
        highlight={!!diffPathMap}
      />
    );
  }

  if (viewMode === 'json') {
    const raw = sideData.raw_json;
    if (!raw) return <EmptyState title="No JSON" body="No raw JSON payload found." />;
    return <pre className="json-block">{formatJson(raw)}</pre>;
  }

  if (viewMode === 'image') {
    const filePath = sideData.file_path;
    return (
      <div className="image-panel">
        <div className="image-frame">
          <img
            src={`asset://localhost/${filePath.replace(/\\/g, '/').split('/').map(encodeURIComponent).join('/')}`}
            alt={sideData.file_name}
            className="image-preview"
            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
          />
        </div>
        <button
          type="button"
          className="open-btn"
          onClick={() => void openPath(filePath)}
        >
          Open Original ↗
        </button>
      </div>
    );
  }

  const _exhaustive: never = viewMode;
  return _exhaustive;
}

function formatJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}
