import { fireEvent, render, screen } from '@testing-library/react';
import type { PairInspection } from '../lib/types';
import { DiffTree } from './DiffTree';
import { InspectorPanel } from './InspectorPanel';

const inspection: PairInspection = {
  left: {
    side: 'left',
    file_path: 'left.png',
    file_name: 'left.png',
    raw_json: '{"Title":"Left"}',
    metadata: { Title: 'Left' },
    error: null,
  },
  right: {
    side: 'right',
    file_path: 'right.png',
    file_name: 'right.png',
    raw_json: '{"Title":"Right"}',
    metadata: { Title: 'Right' },
    error: null,
  },
  diff_root: {
    path: 'StopPlateMetadata',
    status: 'modified',
    left_value: null,
    right_value: null,
    summary: 'changed',
    children: [
      {
        path: 'Title',
        status: 'modified',
        left_value: '"Left"',
        right_value: '"Right"',
        summary: 'Title changed',
        children: [],
      },
    ],
  },
  diff_summary: { modified: 1, added: 0, removed: 0, reordered: 0, error: 0 },
  default_selected_path: 'StopPlateMetadata',
};

describe('workbench sync', () => {
  it('updates the inspector when a diff node is selected', () => {
    let activePath = inspection.default_selected_path;
    const setActivePath = vi.fn((next: string) => {
      activePath = next;
    });

    const { rerender } = render(
      <>
        <DiffTree root={inspection.diff_root} activePath={activePath} onSelect={setActivePath} />
        <InspectorPanel
          inspection={inspection}
          singleSideInspection={null}
          activePath={activePath}
          activeTab="diff"
        />
      </>,
    );

    fireEvent.click(screen.getByRole('button', { name: /title changed/i }));
    expect(setActivePath).toHaveBeenCalledWith('Title');

    rerender(
      <>
        <DiffTree root={inspection.diff_root} activePath={activePath} onSelect={setActivePath} />
        <InspectorPanel
          inspection={inspection}
          singleSideInspection={null}
          activePath={activePath}
          activeTab="diff"
        />
      </>,
    );

    expect(screen.getByText('Selected Path')).toBeInTheDocument();
    expect(screen.getByText('Title')).toBeInTheDocument();
    expect(screen.getByText('"Left"')).toBeInTheDocument();
    expect(screen.getByText('"Right"')).toBeInTheDocument();
  });
});
