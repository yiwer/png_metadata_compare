// frontend/src/App.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import App from './App';

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openPath: vi.fn() }));
vi.mock('./lib/api', () => ({
  workbenchApi: {
    compareSingle: vi.fn(),
    scanDirectory: vi.fn(),
    inspectSingle: vi.fn(),
  },
}));

describe('App', () => {
  it('renders brand name', () => {
    render(<App />);
    expect(screen.getByText(/PNG.*Compare/i)).toBeTruthy();
  });

  it('renders mode toggle buttons', () => {
    render(<App />);
    expect(screen.getByText('Single File')).toBeTruthy();
    expect(screen.getByText('Directory')).toBeTruthy();
  });
});
