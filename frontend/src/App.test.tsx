// frontend/src/App.test.tsx
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import App from './App';
import { workbenchApi } from './lib/api';
import { open } from '@tauri-apps/plugin-dialog';
import type { BatchListItem, DirectorySummary, PairInspection } from './lib/types';

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
    compareSingle: vi.fn().mockResolvedValue({
      left: { side: 'left', file_path: '/l/x.png', file_name: 'x.png', raw_json: null, metadata: null, error: null },
      right: { side: 'right', file_path: '/r/y.png', file_name: 'y.png', raw_json: null, metadata: null, error: null },
      diff_root: { path: '', status: 'unchanged', left_value: null, right_value: null, summary: 'root', children: [] },
      diff_summary: { modified: 0, added: 0, removed: 0, reordered: 0, error: 0 },
      default_selected_path: null,
    }),
    scanDirectory: vi.fn().mockResolvedValue({
      counts: { identical: 0, different: 0, left_only: 0, right_only: 0, error: 0 },
      items: [],
    }),
    inspectSingle: vi.fn().mockResolvedValue({
      side: 'left', file_path: '/x.png', file_name: 'x.png', raw_json: null, metadata: null, error: null,
    }),
  },
}));

const PAIR: PairInspection = {
  left: { side: 'left', file_path: '/l/x.png', file_name: 'x.png', raw_json: '{"Name":"a"}', metadata: { Name: 'a' }, error: null },
  right: { side: 'right', file_path: '/r/y.png', file_name: 'y.png', raw_json: '{"Name":"b"}', metadata: { Name: 'b' }, error: null },
  diff_root: {
    path: '', status: 'modified', left_value: null, right_value: null, summary: '1 处差异',
    children: [{ path: 'Name', status: 'modified', left_value: 'a', right_value: 'b', summary: 'a → b', children: [] }],
  },
  diff_summary: { modified: 1, added: 0, removed: 0, reordered: 0, error: 0 },
  default_selected_path: 'Name',
};

function diffItem(id: string, label: string, count: number): BatchListItem {
  return {
    id, kind: 'different', label,
    left_path: `/l/${label}`, right_path: `/r/${label}`,
    difference_count: count, match_strategy: 'file_name', message: null,
  };
}

function summaryOf(items: BatchListItem[]): DirectorySummary {
  return {
    counts: {
      identical: 0, left_only: 0, right_only: 0, error: 0,
      different: items.filter((i) => i.kind === 'different').length,
    },
    items,
  };
}

/** 切到目录模式并通过两次「浏览」把 /L、/R 填入，触发自动扫描。 */
async function setupDirectoryScan() {
  vi.mocked(open).mockResolvedValueOnce('/L').mockResolvedValueOnce('/R');
  render(<App />);
  fireEvent.click(screen.getByText('目录'));
  // WelcomePane 在加载期间会卸载重挂，所以每次点击前重新查询按钮
  await act(async () => { fireEvent.click(screen.getAllByRole('button', { name: '浏览' })[0]); });
  await act(async () => { fireEvent.click(screen.getAllByRole('button', { name: '浏览' })[1]); });
}

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

  it('directory scan populates the sidebar, auto-selects into mirror and shows the diff rail', async () => {
    vi.mocked(workbenchApi.scanDirectory).mockResolvedValueOnce(summaryOf([diffItem('1', 'a.png', 1)]));
    vi.mocked(workbenchApi.compareSingle).mockResolvedValueOnce(PAIR);
    await setupDirectoryScan();

    expect(workbenchApi.scanDirectory).toHaveBeenCalledWith('/L', '/R', expect.any(Function));
    // 侧栏出现（搜索框在）
    expect(await screen.findByPlaceholderText('搜索文件名…')).toBeInTheDocument();
    // 自动选中进入 mirror（详情头出现）
    await waitFor(() => expect(document.querySelector('.detail-head')).not.toBeNull());
    expect(workbenchApi.compareSingle).toHaveBeenCalledWith('/l/a.png', '/r/a.png');
    // 差异栏出现
    await waitFor(() => expect(document.querySelector('.rail')).not.toBeNull());
  });

  it('clicking another row loads it without flashing the directory-scan progress text', async () => {
    vi.mocked(workbenchApi.scanDirectory).mockResolvedValueOnce(
      summaryOf([diffItem('1', 'a.png', 1), diffItem('2', 'b.png', 2)]),
    );
    let resolveSecond!: (v: PairInspection) => void;
    vi.mocked(workbenchApi.compareSingle)
      .mockResolvedValueOnce(PAIR) // 自动选中 b.png（差异数更多，diff-desc 排序在前）
      .mockImplementationOnce(() => new Promise<PairInspection>((res) => { resolveSecond = res; }));
    await setupDirectoryScan();

    await waitFor(() => expect(document.querySelector('.detail-head')).not.toBeNull());
    expect(workbenchApi.compareSingle).toHaveBeenCalledWith('/l/b.png', '/r/b.png');

    // 点击另一行：selectItem 加载期间不得出现「正在扫描目录…」假进度（I-2）
    fireEvent.click(screen.getByText('a.png'));
    expect(workbenchApi.compareSingle).toHaveBeenCalledWith('/l/a.png', '/r/a.png');
    expect(screen.queryByText('正在扫描目录…')).toBeNull();

    await act(async () => { resolveSecond(PAIR); });
    expect(screen.queryByText('正在扫描目录…')).toBeNull();
    expect(document.querySelector('.detail-head')).not.toBeNull();
  });
});
