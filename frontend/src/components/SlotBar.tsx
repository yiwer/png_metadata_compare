// frontend/src/components/SlotBar.tsx
import { Slot } from './Slot';

function basename(p: string): string {
  if (!p) return '';
  const m = p.match(/[^/\\]+$/);
  return m ? m[0] : p;
}

export function SlotBar({
  mode,
  leftValue,
  rightValue,
  collapsed,
  leftError,
  rightError,
  onPickLeft,
  onPickRight,
  onLeftChange,
  onRightChange,
  onToggleCollapsed,
}: {
  mode: 'single' | 'directory';
  leftValue: string;
  rightValue: string;
  collapsed: boolean;
  leftError?: string | null;
  rightError?: string | null;
  onPickLeft(): void;
  onPickRight(): void;
  onLeftChange(path: string): void;
  onRightChange(path: string): void;
  onToggleCollapsed(): void;
}) {
  if (collapsed) {
    return (
      <div className="slotbar slotbar--collapsed">
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
          左 · {basename(leftValue)}
        </span>
        <span style={{ color: 'var(--text-tertiary)' }}>⇄</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--fs-xs)', color: 'var(--text-secondary)' }}>
          右 · {basename(rightValue)}
        </span>
        <button
          type="button"
          aria-label="展开"
          className="slot__pick"
          onClick={onToggleCollapsed}
        >
          ▼
        </button>
      </div>
    );
  }

  return (
    <div className="slotbar">
      <Slot side="left"  mode={mode} value={leftValue}  errorMessage={leftError}  onPick={onPickLeft}  onChange={onLeftChange} />
      <Slot side="right" mode={mode} value={rightValue} errorMessage={rightError} onPick={onPickRight} onChange={onRightChange} />
    </div>
  );
}
