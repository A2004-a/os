import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CommandPalette } from '../../src/renderer/components/shell/CommandPalette';

describe('CommandPalette Component', () => {
  it('renders nothing when closed', () => {
    const { container } = render(
      <CommandPalette
        isOpen={false}
        onClose={vi.fn()}
        onOpenApp={vi.fn()}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders search input and results when open', () => {
    render(
      <CommandPalette
        isOpen={true}
        onClose={vi.fn()}
        onOpenApp={vi.fn()}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox', { name: /command search input/i });
    expect(input).toBeDefined();
    expect(screen.getByText(/Files/i)).toBeDefined();
    expect(screen.getByText(/Terminal/i)).toBeDefined();
  });

  it('filters results by search query', () => {
    render(
      <CommandPalette
        isOpen={true}
        onClose={vi.fn()}
        onOpenApp={vi.fn()}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );

    const input = screen.getByRole('textbox', { name: /command search input/i });
    fireEvent.change(input, { target: { value: 'Terminal' } });

    expect(screen.getByText(/Terminal/i)).toBeDefined();
    expect(screen.queryByText(/Calculator/i)).toBeNull();
  });

  it('executes item action and closes on click', () => {
    const handleOpenApp = vi.fn();
    const handleClose = vi.fn();

    render(
      <CommandPalette
        isOpen={true}
        onClose={handleClose}
        onOpenApp={handleOpenApp}
        onSwitchWorkspace={vi.fn()}
        onToggleTheme={vi.fn()}
      />
    );

    const terminalItem = screen.getByText(/Terminal \(Host Shell\)/i);
    fireEvent.click(terminalItem);

    expect(handleOpenApp).toHaveBeenCalledWith('io.novaos.terminal', 'Terminal');
    expect(handleClose).toHaveBeenCalled();
  });
});
