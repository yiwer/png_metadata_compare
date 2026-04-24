export function PreviewStrip({
  leftLabel,
  rightLabel,
  onOpenImages,
}: {
  leftLabel: string;
  rightLabel: string;
  onOpenImages(): void;
}) {
  return (
    <section className="preview-strip panel" aria-label="Preview strip">
      <div className="panel-heading">
        <span>Preview Strip</span>
        <strong>Click a side to open the image view</strong>
      </div>
      <div className="preview-grid">
        <button type="button" className="preview-card preview-card--button" onClick={onOpenImages}>
          <span className="preview-label">LEFT PNG</span>
          <div className="preview-frame preview-frame--blue">{leftLabel}</div>
        </button>
        <button type="button" className="preview-card preview-card--button" onClick={onOpenImages}>
          <span className="preview-label">RIGHT PNG</span>
          <div className="preview-frame preview-frame--yellow">{rightLabel}</div>
        </button>
      </div>
    </section>
  );
}
