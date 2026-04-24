import { useWorkbench } from './features/workbench/useWorkbench';
import type { AnalysisTab, BatchListItemKind } from './lib/types';

const tabs: Array<{ id: AnalysisTab; label: string }> = [
  { id: 'diff', label: 'Diff' },
  { id: 'left-metadata', label: 'Left Metadata' },
  { id: 'right-metadata', label: 'Right Metadata' },
  { id: 'raw-json', label: 'Raw JSON' },
  { id: 'images', label: 'Images' },
];

const panelPlaceholders: Record<AnalysisTab, { title: string; body: string }> = {
  diff: {
    title: 'Diff View',
    body: 'State is now sourced from the workbench hook. Full tree rendering arrives in the next task.',
  },
  'left-metadata': {
    title: 'Left Metadata',
    body: 'Placeholder metadata browser for the selected left-side PNG payload.',
  },
  'right-metadata': {
    title: 'Right Metadata',
    body: 'Placeholder metadata browser for the selected right-side PNG payload.',
  },
  'raw-json': {
    title: 'Raw JSON',
    body: 'Placeholder raw payload viewer for comparing extracted JSON side by side.',
  },
  images: {
    title: 'Images',
    body: 'Placeholder full-size image review panel for expanded visual inspection.',
  },
};

function summarizeItem(kind: BatchListItemKind, differenceCount: number, message: string | null) {
  if (message) {
    return message;
  }

  switch (kind) {
    case 'different':
      return `${differenceCount} fields changed`;
    case 'identical':
      return 'pixel + metadata match';
    case 'left_only':
      return 'available on the left side only';
    case 'right_only':
      return 'available on the right side only';
    case 'error':
      return 'metadata payload unreadable';
  }
}

function pathLabel(path: string, emptyLabel: string) {
  if (!path) {
    return emptyLabel;
  }

  const segments = path.split(/[\\/]/);
  return segments[segments.length - 1] || path;
}

export default function App() {
  const {
    mode,
    leftInput,
    rightInput,
    directorySummary,
    activeResultItem,
    activeInspection,
    activeSingleSideInspection,
    activeTab,
    activeNodePath,
    isLoading,
    errorBanner,
    setMode,
    setActiveTab,
    runCompare,
    selectResultItem,
  } = useWorkbench();

  const counts = directorySummary?.counts ?? {
    identical: 0,
    different: 0,
    left_only: 0,
    right_only: 0,
    error: 0,
  };
  const resultRows = directorySummary?.items ?? [];
  const activePairTitle = activeInspection
    ? `${activeInspection.left.file_name} vs ${activeInspection.right.file_name}`
    : activeSingleSideInspection
      ? `${activeSingleSideInspection.file_name} only`
      : 'Select inputs and run compare';
  const inspectorValues = activeInspection
    ? {
        left: activeInspection.left.raw_json ?? 'No left payload',
        right: activeInspection.right.raw_json ?? 'No right payload',
        status:
          activeInspection.diff_summary.error > 0
            ? 'Error'
            : activeInspection.diff_summary.modified > 0 ||
                activeInspection.diff_summary.added > 0 ||
                activeInspection.diff_summary.removed > 0 ||
                activeInspection.diff_summary.reordered > 0
              ? 'Different'
              : 'Identical',
      }
    : activeSingleSideInspection
      ? {
          left:
            activeSingleSideInspection.side === 'left'
              ? activeSingleSideInspection.raw_json ?? 'No payload'
              : 'Missing',
          right:
            activeSingleSideInspection.side === 'right'
              ? activeSingleSideInspection.raw_json ?? 'No payload'
              : 'Missing',
          status: activeSingleSideInspection.error ? 'Error' : 'Single-sided',
        }
      : {
          left: 'Run compare to inspect left-side details.',
          right: 'Run compare to inspect right-side details.',
          status: 'Idle',
        };

  return (
    <div className="app-shell">
      <header className="topbar" aria-label="PNG Metadata Compare">
        <div className="brand-block">
          <p className="eyebrow">Desktop inspection workbench</p>
          <h1>PNG Metadata Compare</h1>
        </div>
        <div className="topbar-controls">
          <div className="mode-switch" role="group" aria-label="Mode switch">
            <button
              type="button"
              className={`mode-button${mode === 'single' ? ' is-active' : ''}`}
              onClick={() => setMode('single')}
            >
              Single File
            </button>
            <button
              type="button"
              className={`mode-button${mode === 'directory' ? ' is-active' : ''}`}
              onClick={() => setMode('directory')}
            >
              Directory
            </button>
          </div>
          <div className="action-toolbar" role="toolbar" aria-label="Compare actions">
            <button type="button" className="toolbar-button toolbar-button--muted">
              {pathLabel(leftInput, mode === 'single' ? 'Choose Left' : 'Choose Left Directory')}
            </button>
            <button type="button" className="toolbar-button toolbar-button--muted">
              {pathLabel(rightInput, mode === 'single' ? 'Choose Right' : 'Choose Right Directory')}
            </button>
            <button
              type="button"
              className="toolbar-button"
              onClick={() => {
                void runCompare();
              }}
            >
              {isLoading ? 'Loading...' : 'Compare'}
            </button>
          </div>
        </div>
      </header>

      {errorBanner ? (
        <div className="panel" role="alert" aria-live="polite">
          <div className="panel-heading">
            <span>Error</span>
            <strong>Workbench banner</strong>
          </div>
          <p>{errorBanner}</p>
        </div>
      ) : null}

      <div className="workbench">
        <aside className="result-rail panel" aria-label="Result rail">
          <div className="panel-heading">
            <span>Result Rail</span>
            <strong>{resultRows.length} items</strong>
          </div>
          <div className="summary-strip">
            <span>{counts.different} different</span>
            <span>{counts.identical} identical</span>
            <span>{counts.error} errors</span>
          </div>
          <div className="result-list">
            {resultRows.length > 0 ? (
              resultRows.map((row) => (
                <button
                  key={row.id}
                  type="button"
                  className={`result-row${activeResultItem?.id === row.id ? ' is-selected' : ''}`}
                  onClick={() => {
                    void selectResultItem(row.id);
                  }}
                >
                  <span className={`status-dot status-dot--${row.kind}`} aria-hidden="true" />
                  <span className="result-copy">
                    <strong>{row.label}</strong>
                    <small>{summarizeItem(row.kind, row.difference_count, row.message)}</small>
                  </span>
                </button>
              ))
            ) : (
              <div className="analysis-panel">
                <h2>No results yet</h2>
                <p>Run a compare to populate the result rail.</p>
              </div>
            )}
          </div>
        </aside>

        <section className="workspace-column">
          <section className="preview-strip panel" aria-label="Preview strip">
            <div className="panel-heading">
              <span>Preview Strip</span>
              <strong>{activePairTitle}</strong>
            </div>
            <div className="preview-grid">
              <article className="preview-card">
                <span className="preview-label">LEFT PNG</span>
                <div className="preview-frame preview-frame--blue">
                  {activeInspection
                    ? activeInspection.left.file_name
                    : activeSingleSideInspection?.side === 'left'
                      ? activeSingleSideInspection.file_name
                      : 'Awaiting selection'}
                </div>
              </article>
              <article className="preview-card">
                <span className="preview-label">RIGHT PNG</span>
                <div className="preview-frame preview-frame--yellow">
                  {activeInspection
                    ? activeInspection.right.file_name
                    : activeSingleSideInspection?.side === 'right'
                      ? activeSingleSideInspection.file_name
                      : 'Awaiting selection'}
                </div>
              </article>
            </div>
          </section>

          <nav className="tab-strip panel" aria-label="Analysis views">
            <div className="tablist" role="tablist" aria-label="Analysis views">
              {tabs.map((tab) => {
                const tabId = `analysis-tab-${tab.id}`;
                const panelId = `analysis-panel-${tab.id}`;
                const isActive = tab.id === activeTab;

                return (
                  <button
                    key={tab.id}
                    id={tabId}
                    aria-controls={panelId}
                    tabIndex={isActive ? 0 : -1}
                    type="button"
                    role="tab"
                    aria-selected={isActive}
                    className={`tab-button${isActive ? ' is-active' : ''}`}
                    onClick={() => setActiveTab(tab.id)}
                  >
                    {tab.label}
                  </button>
                );
              })}
            </div>
          </nav>

          <div className="analysis-row">
            <main className="analysis-main panel" aria-label="Analysis workspace">
              {tabs.map((tab) => {
                const panelId = `analysis-panel-${tab.id}`;
                const tabId = `analysis-tab-${tab.id}`;
                const isActive = tab.id === activeTab;
                const placeholder = panelPlaceholders[tab.id];

                return (
                  <section
                    key={tab.id}
                    id={panelId}
                    role="tabpanel"
                    aria-labelledby={tabId}
                    className="analysis-tabpanel"
                    hidden={!isActive}
                  >
                    {isActive ? (
                      <>
                        <div className="panel-heading">
                          <span>{placeholder.title}</span>
                          <strong>
                            {activeInspection
                              ? `${activeInspection.diff_summary.modified} modified fields`
                              : activeSingleSideInspection
                                ? 'Single-side inspection'
                                : 'Scaffold placeholder'}
                          </strong>
                        </div>
                        <div className="analysis-panels">
                          <section className="analysis-panel">
                            <h2>{placeholder.title}</h2>
                            <ul>
                              <li>{activeNodePath ?? 'No active node selected.'}</li>
                              <li>
                                {activeResultItem
                                  ? summarizeItem(
                                      activeResultItem.kind,
                                      activeResultItem.difference_count,
                                      activeResultItem.message,
                                    )
                                  : 'Run compare to populate synchronized workbench state.'}
                              </li>
                              <li>
                                {activeInspection
                                  ? activeInspection.diff_root.summary
                                  : activeSingleSideInspection?.error?.message ??
                                    'Payload wiring is active; full tree rendering lands next.'}
                              </li>
                            </ul>
                          </section>
                          <section className="analysis-panel">
                            <h2>Workbench Notes</h2>
                            <p>{placeholder.body}</p>
                          </section>
                        </div>
                      </>
                    ) : (
                      <>
                        <div className="panel-heading">
                          <span>{placeholder.title}</span>
                          <strong>Scaffold placeholder</strong>
                        </div>
                        <section className="analysis-panel">
                          <h2>{placeholder.title}</h2>
                          <p>{placeholder.body}</p>
                        </section>
                      </>
                    )}
                  </section>
                );
              })}
            </main>

            <aside className="inspector panel" aria-label="Detail inspector">
              <div className="panel-heading">
                <span>Inspector</span>
                <strong>Selected node</strong>
              </div>
              <dl className="inspector-list">
                <div>
                  <dt>Path</dt>
                  <dd>{activeNodePath ?? 'No active node selected'}</dd>
                </div>
                <div>
                  <dt>Left</dt>
                  <dd>{inspectorValues.left}</dd>
                </div>
                <div>
                  <dt>Right</dt>
                  <dd>{inspectorValues.right}</dd>
                </div>
                <div>
                  <dt>Status</dt>
                  <dd>{inspectorValues.status}</dd>
                </div>
              </dl>
            </aside>
          </div>
        </section>
      </div>
    </div>
  );
}
