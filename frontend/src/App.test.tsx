// frontend/src/App.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import App from './App';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn(), onCloseRequested: vi.fn(),
  })),
}));
vi.mock('@tauri-apps/api/core', () => ({ convertFileSrc: vi.fn((p: string) => `asset://${p}`) }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openPath: vi.fn(), revealItemInDir: vi.fn() }));
vi.mock('./lib/api', () => ({
  workbenchApi: {
    compareSingle: vi.fn(),
    scanDirectory: vi.fn(),
    inspectSingle: vi.fn(),
  },
}));

describe('App (three-column shell)', () => {
  beforeEach(() => localStorage.clear());

  it('renders brand name', () => {
    render(<App />);
    expect(screen.getAllByText(/PNG Compare/i).length).toBeGreaterThan(0);
  });

  it('renders the welcome pane with pick slots on first load', () => {
    render(<App />);
    expect(screen.getByText(/拖入PNG 文件/)).toBeTruthy();
    expect(screen.getAllByRole('button', { name: '浏览' })).toHaveLength(2);
    // 空载时既无目录侧栏也无详情头
    expect(document.querySelector('.sidebar')).toBeNull();
    expect(document.querySelector('.detail-head')).toBeNull();
  });

  it('mode toggle switches the welcome noun between file and folder', () => {
    render(<App />);
    expect(screen.getByText('单文件')).toBeTruthy();
    fireEvent.click(screen.getByText('目录'));
    expect(screen.getByText(/拖入文件夹/)).toBeTruthy();
    fireEvent.click(screen.getByText('单文件'));
    expect(screen.getByText(/拖入PNG 文件/)).toBeTruthy();
  });
});
