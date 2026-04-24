const resultRows = [
  { name: 'A001_front.png', status: 'different', delta: '12 fields changed' },
  { name: 'A002_back.png', status: 'identical', delta: 'pixel + metadata match' },
  { name: 'A003_plate.png', status: 'error', delta: 'metadata payload unreadable' },
];

const tabs = ['Diff', 'Left Metadata', 'Right Metadata', 'Raw JSON', 'Images'];

export default function App() {
  return (
    <div className="app-shell">
      <header className="topbar" aria-label="PNG Metadata Compare">
        <div className="brand-block">
          <p className="eyebrow">Desktop inspection workbench</p>
          <h1>PNG Metadata Compare</h1>
        </div>
        <div className="topbar-controls">
          <div className="mode-switch" role="group" aria-label="Mode switch">
            <button type="button" className="mode-button is-active">
              Single File
            </button>
            <button type="button" className="mode-button">
              Directory
            </button>
          </div>
          <div className="action-toolbar" role="toolbar" aria-label="Compare actions">
            <button type="button" className="toolbar-button">
              Compare
            </button>
            <button type="button" className="toolbar-button toolbar-button--muted">
              Swap
            </button>
            <span className="filter-pill">Only Differences</span>
          </div>
        </div>
      </header>

      <div className="workbench">
        <aside className="result-rail panel" aria-label="Result rail">
          <div className="panel-heading">
            <span>Result Rail</span>
            <strong>18 items</strong>
          </div>
          <div className="summary-strip">
            <span>12 different</span>
            <span>4 identical</span>
            <span>2 errors</span>
          </div>
          <div className="result-list">
            {resultRows.map((row, index) => (
              <button
                key={row.name}
                type="button"
                className={`result-row${index === 0 ? ' is-selected' : ''}`}
              >
                <span className={`status-dot status-dot--${row.status}`} aria-hidden="true" />
                <span className="result-copy">
                  <strong>{row.name}</strong>
                  <small>{row.delta}</small>
                </span>
              </button>
            ))}
          </div>
        </aside>

        <section className="workspace-column">
          <section className="preview-strip panel" aria-label="Preview strip">
            <div className="panel-heading">
              <span>Preview Strip</span>
              <strong>A001_front.png vs B001_front.png</strong>
            </div>
            <div className="preview-grid">
              <article className="preview-card">
                <span className="preview-label">LEFT PNG</span>
                <div className="preview-frame preview-frame--blue">1200 x 900</div>
              </article>
              <article className="preview-card">
                <span className="preview-label">RIGHT PNG</span>
                <div className="preview-frame preview-frame--yellow">1200 x 900</div>
              </article>
            </div>
          </section>

          <nav className="tab-strip panel" aria-label="Analysis views">
            <div className="tablist" role="tablist" aria-label="Analysis views">
              {tabs.map((tab, index) => (
                <button
                  key={tab}
                  type="button"
                  role="tab"
                  aria-selected={index === 0}
                  className={`tab-button${index === 0 ? ' is-active' : ''}`}
                >
                  {tab}
                </button>
              ))}
            </div>
          </nav>

          <div className="analysis-row">
            <main className="analysis-main panel" aria-label="Analysis workspace">
              <div className="panel-heading">
                <span>Diff View</span>
                <strong>12 changed fields</strong>
              </div>
              <div className="analysis-panels">
                <section className="analysis-panel">
                  <h2>Change Map</h2>
                  <ul>
                    <li>`metadata.stop_plate.serial` drift detected</li>
                    <li>`text_chunks[2].value` removed on the right side</li>
                    <li>`gamma` normalized during export</li>
                  </ul>
                </section>
                <section className="analysis-panel">
                  <h2>Workbench Notes</h2>
                  <p>
                    Static shell scaffold for the desktop workbench. Interactive data wiring
                    arrives in later tasks.
                  </p>
                </section>
              </div>
            </main>

            <aside className="inspector panel" aria-label="Detail inspector">
              <div className="panel-heading">
                <span>Inspector</span>
                <strong>Selected node</strong>
              </div>
              <dl className="inspector-list">
                <div>
                  <dt>Path</dt>
                  <dd>metadata.stop_plate.serial</dd>
                </div>
                <div>
                  <dt>Left</dt>
                  <dd>SP-22014</dd>
                </div>
                <div>
                  <dt>Right</dt>
                  <dd>SP-22014-REV-B</dd>
                </div>
                <div>
                  <dt>Status</dt>
                  <dd>Different</dd>
                </div>
              </dl>
            </aside>
          </div>
        </section>
      </div>
    </div>
  );
}
