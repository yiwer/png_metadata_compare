// frontend/src/App.tsx
import { open } from '@tauri-apps/plugin-dialog';
import { DirectoryOverview } from './components/DirectoryOverview';
import { PairComparison } from './components/PairComparison';
import { useWorkbench } from './features/workbench/useWorkbench';

async function pickPath(directory: boolean): Promise<string> {
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: 'PNG', extensions: ['png'] }],
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

  const progressLabel =
    wb.directoryContext
      ? `${wb.directoryContext.index} / ${wb.directoryContext.totalDifferent} different`
      : null;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="topbar-left">
          {wb.view === 'pair-comparison' && wb.directoryContext && (
            <button type="button" className="back-btn" onClick={wb.goBackToDirectory}>
              ← Directory
            </button>
          )}
          <span className="brand">PNG ⌁ Compare</span>
        </div>

        <div className="topbar-center">
          {wb.view === 'pair-comparison' && wb.pairResult && (
            <span className="topbar-filename">{wb.pairResult.left.file_name}</span>
          )}
        </div>

        <div className="topbar-right">
          {wb.view === 'directory-overview' || wb.directoryContext === null ? (
            <div className="mode-toggle" role="group" aria-label="Mode">
              <button
                type="button"
                className={`mode-btn${wb.mode === 'single' ? ' mode-btn--active' : ''}`}
                onClick={() => wb.setMode('single')}
              >
                Single File
              </button>
              <button
                type="button"
                className={`mode-btn${wb.mode === 'directory' ? ' mode-btn--active' : ''}`}
                onClick={() => wb.setMode('directory')}
              >
                Directory
              </button>
            </div>
          ) : (
            progressLabel && <span className="topbar-progress">{progressLabel}</span>
          )}
        </div>
      </header>

      {wb.view === 'directory-overview' ? (
        <DirectoryOverview
          leftInput={wb.leftInput}
          rightInput={wb.rightInput}
          directorySummary={wb.directorySummary}
          filteredItems={wb.filteredItems}
          activeFilter={wb.activeFilter}
          isLoading={wb.isLoading}
          error={wb.error}
          onLeftInput={wb.setLeftInput}
          onRightInput={wb.setRightInput}
          onScan={() => { void wb.runCompare(); }}
          onPickLeft={() => { void handlePickLeft(); }}
          onPickRight={() => { void handlePickRight(); }}
          onFilter={wb.setActiveFilter}
          onSelectItem={(item) => { void wb.navigateToPair(item); }}
        />
      ) : (
        <PairComparison
          mode={wb.mode}
          leftInput={wb.leftInput}
          rightInput={wb.rightInput}
          pairResult={wb.pairResult}
          viewMode={wb.viewMode}
          diffHighlight={wb.diffHighlight}
          isLoading={wb.isLoading}
          error={wb.error}
          onLeftInput={wb.setLeftInput}
          onRightInput={wb.setRightInput}
          onCompare={() => { void wb.runCompare(); }}
          onPickLeft={() => { void handlePickLeft(); }}
          onPickRight={() => { void handlePickRight(); }}
          onViewMode={wb.setViewMode}
          onToggleDiff={wb.toggleDiffHighlight}
        />
      )}
    </div>
  );
}
