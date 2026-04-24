import { convertFileSrc } from '@tauri-apps/api/core';

function toImageSrc(path?: string) {
  if (!path) {
    return '';
  }

  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
    ? convertFileSrc(path)
    : path;
}

export function ImagePane({
  leftPath,
  rightPath,
  onOpenLeft,
  onOpenRight,
}: {
  leftPath?: string;
  rightPath?: string;
  onOpenLeft?(path?: string): void;
  onOpenRight?(path?: string): void;
}) {
  return (
    <div className="raw-grid">
      <section className="analysis-panel">
        <h2>Left Image</h2>
        {leftPath ? (
          <button
            type="button"
            className="toolbar-button toolbar-button--muted image-action"
            onClick={() => onOpenLeft?.(leftPath)}
          >
            Open Original
          </button>
        ) : null}
        <div className="image-frame">
          {leftPath ? <img alt="Left PNG" src={toImageSrc(leftPath)} /> : 'No left image available.'}
        </div>
      </section>
      <section className="analysis-panel">
        <h2>Right Image</h2>
        {rightPath ? (
          <button
            type="button"
            className="toolbar-button toolbar-button--muted image-action"
            onClick={() => onOpenRight?.(rightPath)}
          >
            Open Original
          </button>
        ) : null}
        <div className="image-frame">
          {rightPath ? (
            <img alt="Right PNG" src={toImageSrc(rightPath)} />
          ) : (
            'No right image available.'
          )}
        </div>
      </section>
    </div>
  );
}
