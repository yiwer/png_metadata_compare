export function RawJsonPanel({
  leftRaw,
  rightRaw,
}: {
  leftRaw: string | null | undefined;
  rightRaw: string | null | undefined;
}) {
  return (
    <div className="raw-grid">
      <section className="analysis-panel">
        <h2>Left JSON</h2>
        <pre className="code-block">{leftRaw ?? 'No left JSON available.'}</pre>
      </section>
      <section className="analysis-panel">
        <h2>Right JSON</h2>
        <pre className="code-block">{rightRaw ?? 'No right JSON available.'}</pre>
      </section>
    </div>
  );
}
