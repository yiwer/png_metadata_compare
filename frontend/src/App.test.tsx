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
});
