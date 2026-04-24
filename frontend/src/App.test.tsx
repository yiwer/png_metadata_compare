import { render, screen } from '@testing-library/react';
import App from './App';

describe('App shell', () => {
  it('renders the static workbench shell regions', () => {
    render(<App />);

    expect(screen.getByRole('banner', { name: /png metadata compare/i })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: /mode switch/i })).toBeInTheDocument();
    expect(screen.getByRole('toolbar', { name: /compare actions/i })).toBeInTheDocument();
    expect(screen.getByRole('complementary', { name: /result rail/i })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: /preview strip/i })).toBeInTheDocument();
    expect(screen.getByRole('tablist', { name: /analysis views/i })).toBeInTheDocument();
    expect(screen.getByRole('main', { name: /analysis workspace/i })).toBeInTheDocument();
    expect(screen.getByRole('complementary', { name: /detail inspector/i })).toBeInTheDocument();
  });
});
