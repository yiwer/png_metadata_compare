import type { AnalysisTab } from '../lib/types';

export const analysisTabs: Array<{ id: AnalysisTab; label: string }> = [
  { id: 'diff', label: 'Diff' },
  { id: 'left_metadata', label: 'Left Metadata' },
  { id: 'right_metadata', label: 'Right Metadata' },
  { id: 'raw_json', label: 'Raw JSON' },
  { id: 'images', label: 'Images' },
];

export function TabBar({
  activeTab,
  onSelect,
}: {
  activeTab: AnalysisTab;
  onSelect(tab: AnalysisTab): void;
}) {
  return (
    <nav className="tab-strip panel" aria-label="Analysis views">
      <div className="tablist" role="tablist" aria-label="Analysis views">
        {analysisTabs.map((tab) => {
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
              onClick={() => onSelect(tab.id)}
            >
              {tab.label}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
