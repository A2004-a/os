import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { NovaLauncher } from '../../src/renderer/components/shell/NovaLauncher';

describe('NovaLauncher Component', () => {
  it('renders start menu overview with pinned apps and quick actions when query is empty', () => {
    const handleOpenApp = vi.fn();

    render(
      <NovaLauncher
        isOpen={true}
        onClose={vi.fn()}
        onOpenApp={handleOpenApp}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );

    expect(screen.getByRole('dialog', { name: /nova launcher/i })).toBeDefined();
    expect(screen.getByText(/Pinned Apps/i)).toBeDefined();
    expect(screen.getByText(/Continue Working/i)).toBeDefined();
    expect(screen.getByText(/New Note/i)).toBeDefined();
    expect(screen.getByText(/NovaOS User/i)).toBeDefined();

    // Click on Pinned App
    const filesAppBtn = screen.getByRole('button', { name: /files/i });
    fireEvent.click(filesAppBtn);
    expect(handleOpenApp).toHaveBeenCalledWith('io.novaos.files', 'Files');
  });

  it('filters results dynamically when searching', () => {
    render(
      <NovaLauncher
        isOpen={true}
        onClose={vi.fn()}
        onOpenApp={vi.fn()}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );

    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'terminal' } });

    expect(screen.getByText(/Search Results/i)).toBeDefined();
    expect(screen.getByText('Terminal')).toBeDefined();
    expect(screen.getByText(/Native platform shell/i)).toBeDefined();
  });
});
