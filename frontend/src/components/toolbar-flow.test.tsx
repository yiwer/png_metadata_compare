import { fireEvent, render, screen } from '@testing-library/react';
import { Toolbar } from './Toolbar';

describe('toolbar flow', () => {
  it('disables compare until both paths are present and triggers pick actions', () => {
    const props = {
      mode: 'single' as const,
      leftInput: '',
      rightInput: '',
      isLoading: false,
      onLeftInputChange: vi.fn(),
      onRightInputChange: vi.fn(),
      onPickLeft: vi.fn(),
      onPickRight: vi.fn(),
      onCompare: vi.fn(),
    };

    render(<Toolbar {...props} />);

    expect(screen.getByRole('button', { name: /compare/i })).toBeDisabled();

    fireEvent.click(screen.getByRole('button', { name: /choose left png/i }));
    expect(props.onPickLeft).toHaveBeenCalled();
  });
});
