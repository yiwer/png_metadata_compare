import type { WorkbenchMode } from '../lib/types';

export function Toolbar({
  mode,
  leftInput,
  rightInput,
  isLoading,
  onLeftInputChange,
  onRightInputChange,
  onPickLeft,
  onPickRight,
  onCompare,
}: {
  mode: WorkbenchMode;
  leftInput: string;
  rightInput: string;
  isLoading: boolean;
  onLeftInputChange(value: string): void;
  onRightInputChange(value: string): void;
  onPickLeft(): void;
  onPickRight(): void;
  onCompare(): void;
}) {
  const compareDisabled = !leftInput.trim() || !rightInput.trim() || isLoading;

  return (
    <div className="action-toolbar" role="toolbar" aria-label="Compare actions">
      <button type="button" className="toolbar-button toolbar-button--muted" onClick={onPickLeft}>
        {mode === 'single' ? 'Choose Left PNG' : 'Choose Left Folder'}
      </button>
      <input
        aria-label={mode === 'single' ? 'Left PNG path' : 'Left directory path'}
        className="toolbar-input"
        type="text"
        value={leftInput}
        onChange={(event) => onLeftInputChange(event.target.value)}
        placeholder={mode === 'single' ? 'Left PNG path' : 'Left directory path'}
      />
      <button type="button" className="toolbar-button toolbar-button--muted" onClick={onPickRight}>
        {mode === 'single' ? 'Choose Right PNG' : 'Choose Right Folder'}
      </button>
      <input
        aria-label={mode === 'single' ? 'Right PNG path' : 'Right directory path'}
        className="toolbar-input"
        type="text"
        value={rightInput}
        onChange={(event) => onRightInputChange(event.target.value)}
        placeholder={mode === 'single' ? 'Right PNG path' : 'Right directory path'}
      />
      <button type="button" className="toolbar-button" disabled={compareDisabled} onClick={onCompare}>
        {isLoading ? 'Loading...' : 'Compare'}
      </button>
    </div>
  );
}
