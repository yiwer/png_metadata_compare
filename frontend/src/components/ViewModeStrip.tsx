// frontend/src/components/ViewModeStrip.tsx
import type { ViewMode } from '../features/workbench/useWorkbench';

const MODES: { id: ViewMode; label: string }[] = [
  { id: 'tree', label: 'Tree' },
  { id: 'json', label: 'JSON' },
  { id: 'image', label: 'Image' },
];

export function ViewModeStrip({
  viewMode,
  diffHighlight,
  changeCount,
  onViewMode,
  onToggleDiff,
}: {
  viewMode: ViewMode;
  diffHighlight: boolean;
  changeCount: number;
  onViewMode(mode: ViewMode): void;
  onToggleDiff(): void;
}) {
  return (
    <div className="view-strip">
      <span className="view-strip__label">View</span>
      <div className="seg-group" role="group" aria-label="View mode">
        {MODES.map((m) => (
          <button
            key={m.id}
            type="button"
            className={`seg${viewMode === m.id ? ' seg--active' : ''}`}
            aria-pressed={viewMode === m.id}
            onClick={() => onViewMode(m.id)}
          >
            {m.label}
          </button>
        ))}
      </div>
      <div className="view-strip__right">
        <span className="view-strip__diff-label">Highlight Diffs</span>
        <button
          type="button"
          className={`diff-toggle${diffHighlight ? ' diff-toggle--on' : ''}`}
          aria-label={diffHighlight ? 'Disable diff highlight' : 'Enable diff highlight'}
          onClick={onToggleDiff}
        >
          <span className="diff-toggle__knob" />
        </button>
        <span className={`change-badge${changeCount === 0 ? ' change-badge--zero' : ''}`}>
          {changeCount} {changeCount === 1 ? 'change' : 'changes'}
        </span>
      </div>
    </div>
  );
}
