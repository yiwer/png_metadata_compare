import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { DiffTree } from './components/DiffTree';
import { EmptyState } from './components/EmptyState';
import { ImagePane } from './components/ImagePane';
import { InspectorPanel } from './components/InspectorPanel';
import { MetadataTree } from './components/MetadataTree';
import { PreviewStrip } from './components/PreviewStrip';
import { RawJsonPanel } from './components/RawJsonPanel';
import { ResultRail } from './components/ResultRail';
import { StatusBanner } from './components/StatusBanner';
import { TabBar, analysisTabs } from './components/TabBar';
import { Toolbar } from './components/Toolbar';
import { useWorkbench } from './features/workbench/useWorkbench';
import type { AnalysisTab, JsonValue } from './lib/types';

const tabMeta: Record<AnalysisTab, { title: string; empty: string }> = {
  diff: {
    title: 'Diff View',
    empty: 'Pair results render an interactive diff tree here.',
  },
  left_metadata: {
    title: 'Left Metadata',
    empty: 'Left-side metadata appears here when available.',
  },
  right_metadata: {
    title: 'Right Metadata',
    empty: 'Right-side metadata appears here when available.',
  },
  raw_json: {
    title: 'Raw JSON',
    empty: 'Raw extracted payloads render side by side here.',
  },
  images: {
    title: 'Images',
    empty: 'Expanded image review appears here after a result is loaded.',
  },
};

async function pickPath(directory: boolean) {
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: 'PNG', extensions: ['png'] }],
  });

  return typeof selected === 'string' ? selected : '';
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
    loadingMessage,
    errorBanner,
    setMode,
    setLeftInput,
    setRightInput,
    setActiveTab,
    setActiveNodePath,
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
      : 'Preview placeholder';
  const diffCount = activeInspection
    ? activeInspection.diff_summary.modified +
      activeInspection.diff_summary.added +
      activeInspection.diff_summary.removed +
      activeInspection.diff_summary.reordered +
      activeInspection.diff_summary.error
    : 0;
  const leftMetadata =
    activeInspection?.left.metadata ??
    (activeSingleSideInspection?.side === 'left' ? activeSingleSideInspection.metadata : null);
  const rightMetadata =
    activeInspection?.right.metadata ??
    (activeSingleSideInspection?.side === 'right' ? activeSingleSideInspection.metadata : null);
  const leftRaw =
    activeInspection?.left.raw_json ??
    (activeSingleSideInspection?.side === 'left' ? activeSingleSideInspection.raw_json : null);
  const rightRaw =
    activeInspection?.right.raw_json ??
    (activeSingleSideInspection?.side === 'right' ? activeSingleSideInspection.raw_json : null);
  const leftImagePath =
    activeInspection?.left.file_path ??
    (activeSingleSideInspection?.side === 'left' ? activeSingleSideInspection.file_path : undefined);
  const rightImagePath =
    activeInspection?.right.file_path ??
    (activeSingleSideInspection?.side === 'right' ? activeSingleSideInspection.file_path : undefined);

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
          <Toolbar
            mode={mode}
            leftInput={leftInput}
            rightInput={rightInput}
            isLoading={isLoading}
            onLeftInputChange={setLeftInput}
            onRightInputChange={setRightInput}
            onPickLeft={async () => {
              const picked = await pickPath(mode === 'directory');
              if (picked) {
                setLeftInput(picked);
              }
            }}
            onPickRight={async () => {
              const picked = await pickPath(mode === 'directory');
              if (picked) {
                setRightInput(picked);
              }
            }}
            onCompare={() => {
              void runCompare();
            }}
          />
        </div>
      </header>

      <StatusBanner loading={loadingMessage} error={errorBanner} />

      <div className="workbench">
        <ResultRail
          counts={directorySummary ? counts : null}
          items={resultRows}
          activeId={activeResultItem?.id ?? null}
          onSelect={(item) => {
            void selectResultItem(item.id);
          }}
        />

        <section className="workspace-column">
          <PreviewStrip
            leftLabel={
              activeInspection?.left.file_name ??
              (activeSingleSideInspection?.side === 'left'
                ? activeSingleSideInspection.file_name
                : 'No left PNG selected')
            }
            rightLabel={
              activeInspection?.right.file_name ??
              (activeSingleSideInspection?.side === 'right'
                ? activeSingleSideInspection.file_name
                : 'No right PNG selected')
            }
            onOpenImages={() => setActiveTab('images')}
          />

          <TabBar activeTab={activeTab} onSelect={setActiveTab} />

          <div className="analysis-row">
            <main className="analysis-main panel" aria-label="Analysis workspace">
              {analysisTabs.map((tab) => {
                const panelId = `analysis-panel-${tab.id}`;
                const tabId = `analysis-tab-${tab.id}`;
                const isActive = tab.id === activeTab;
                const meta = tabMeta[tab.id];

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
                          <span>{meta.title}</span>
                          <strong>
                            {isLoading
                              ? 'Loading workspace'
                              : errorBanner
                                ? 'Error banner active'
                                : activeInspection
                                  ? `${diffCount} changed fields`
                                  : activeSingleSideInspection
                                    ? 'Single-side inspection'
                                    : activePairTitle}
                          </strong>
                        </div>
                        {renderPanel({
                          activeTab: tab.id,
                          activeInspection,
                          activeNodePath,
                          activeSingleSideInspection,
                          errorBanner,
                          leftImagePath,
                          leftMetadata,
                          leftRaw,
                          rightImagePath,
                          rightMetadata,
                          rightRaw,
                          setActiveNodePath,
                          openImagePath: (path?: string) => {
                            if (path) {
                              void openPath(path);
                            }
                          },
                        })}
                      </>
                    ) : (
                      <>
                        <div className="panel-heading">
                          <span>{meta.title}</span>
                          <strong>Scaffold placeholder</strong>
                        </div>
                        <section className="analysis-panel">
                          <h2>{meta.title}</h2>
                          <p>{meta.empty}</p>
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
                <strong>{tabMeta[activeTab].title}</strong>
              </div>
              <InspectorPanel
                inspection={activeInspection}
                singleSideInspection={activeSingleSideInspection}
                activePath={activeNodePath}
                activeTab={activeTab}
              />
            </aside>
          </div>
        </section>
      </div>
    </div>
  );
}

function renderPanel({
  activeTab,
  activeInspection,
  activeNodePath,
  activeSingleSideInspection,
  errorBanner,
  leftImagePath,
  leftMetadata,
  leftRaw,
  rightImagePath,
  rightMetadata,
  rightRaw,
  setActiveNodePath,
  openImagePath,
}: {
  activeTab: AnalysisTab;
  activeInspection: ReturnType<typeof useWorkbench>['activeInspection'];
  activeNodePath: string | null;
  activeSingleSideInspection: ReturnType<typeof useWorkbench>['activeSingleSideInspection'];
  errorBanner: string | null;
  leftImagePath?: string;
  leftMetadata: JsonValue | null;
  leftRaw: string | null;
  rightImagePath?: string;
  rightMetadata: JsonValue | null;
  rightRaw: string | null;
  setActiveNodePath(path: string | null): void;
  openImagePath(path?: string): void;
}) {
  if (errorBanner && !activeInspection && !activeSingleSideInspection) {
    return (
      <section className="analysis-panel">
        <h2>Compare Error</h2>
        <p>{errorBanner}</p>
      </section>
    );
  }

  switch (activeTab) {
    case 'diff':
      return activeInspection ? (
        <DiffTree
          root={activeInspection.diff_root}
          activePath={activeNodePath}
          onSelect={setActiveNodePath}
        />
      ) : (
        <EmptyState
          title="Choose inputs and run compare"
          body="The workbench will show result navigation, image previews, metadata, and raw JSON here."
        />
      );
    case 'left_metadata':
      return leftMetadata ? (
        <MetadataTree value={leftMetadata} activePath={activeNodePath} onSelect={setActiveNodePath} />
      ) : (
        <EmptyState
          title="No left-side metadata yet"
          body="Load a PNG pair or a left-only result to inspect the extracted metadata tree."
        />
      );
    case 'right_metadata':
      return rightMetadata ? (
        <MetadataTree
          value={rightMetadata}
          activePath={activeNodePath}
          onSelect={setActiveNodePath}
        />
      ) : (
        <EmptyState
          title="No right-side metadata yet"
          body="Load a PNG pair or a right-only result to inspect the extracted metadata tree."
        />
      );
    case 'raw_json':
      return <RawJsonPanel leftRaw={leftRaw} rightRaw={rightRaw} />;
    case 'images':
      return (
        <ImagePane
          leftPath={leftImagePath}
          rightPath={rightImagePath}
          onOpenLeft={openImagePath}
          onOpenRight={openImagePath}
        />
      );
  }
}
