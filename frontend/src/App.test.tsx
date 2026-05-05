// frontend/src/App.test.tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import App from './App';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(), onCloseRequested: vi.fn(),
  })),
}));
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
    expect(screen.getAllByText(/PNG.*Compare/i).length).toBeGreaterThan(0);
  });

  it('renders welcome screen on first load', () => {
    render(<App />);
    expect(screen.getByText(/PNG 文件/)).toBeTruthy();
  });

  it('renders mode toggle buttons', () => {
    render(<App />);
    expect(screen.getByText('单文件')).toBeTruthy();
    expect(screen.getByText('目录')).toBeTruthy();
  });
});
