import { render, screen } from '@testing-library/react';
import App from './App';

describe('App shell', () => {
  it('renders the planned scaffold labels', () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: /png metadata compare/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Single File' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Directory' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Choose Left' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Choose Right' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Compare' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Diff' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Left Metadata' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Right Metadata' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Raw JSON' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Images' })).toBeInTheDocument();
  });

  it('wires the active tab to a tabpanel with roving tab focus', () => {
    render(<App />);

    const tabs = screen.getAllByRole('tab');
    const diffTab = screen.getByRole('tab', { name: 'Diff' });
    const leftMetadataTab = screen.getByRole('tab', { name: 'Left Metadata' });
    const tabPanel = screen.getByRole('tabpanel', { name: 'Diff' });

    for (const tab of tabs) {
      const controlsId = tab.getAttribute('aria-controls');
      expect(controlsId).toBeTruthy();

      const controlledPanel = document.getElementById(controlsId!);
      expect(controlledPanel).not.toBeNull();
      expect(controlledPanel).toHaveAttribute('role', 'tabpanel');
      expect(controlledPanel).toHaveAttribute('aria-labelledby', tab.id);
    }

    expect(diffTab).toHaveAttribute('id');
    expect(diffTab).toHaveAttribute('aria-controls', tabPanel.id);
    expect(diffTab).toHaveAttribute('tabindex', '0');
    expect(diffTab).toHaveAttribute('aria-selected', 'true');

    expect(leftMetadataTab).toHaveAttribute('id');
    expect(leftMetadataTab).toHaveAttribute('aria-controls');
    expect(leftMetadataTab).toHaveAttribute('tabindex', '-1');
    expect(leftMetadataTab).toHaveAttribute('aria-selected', 'false');

    expect(tabPanel).toHaveAttribute('aria-labelledby', diffTab.id);
  });
});
