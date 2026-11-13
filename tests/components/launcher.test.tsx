import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Launcher } from '../../src/renderer/components/shell/Launcher';

describe('Launcher Component', () => {
  it('renders in dock mode with pinned apps', () => {
    const handleOpenApp = vi.fn();

    render(
      <Launcher
        mode="dock"
        windows={[]}
        onOpenApp={handleOpenApp}
        onFocusWindow={vi.fn()}
        onToggleCommandPalette={vi.fn()}
        onToggleQuickSettings={vi.fn()}
        onToggleNotifications={vi.fn()}
        activeWorkspaceId="ws-1"
      />
    );

    const dock = screen.getByRole('navigation', { name: /application dock/i });
    expect(dock).toBeDefined();

    const filesBtn = screen.getByRole('button', { name: /files/i });
    fireEvent.click(filesBtn);
    expect(handleOpenApp).toHaveBeenCalledWith('io.novaos.files', 'Files');
  });

  it('renders in taskbar mode', () => {
    render(
      <Launcher
        mode="taskbar"
        windows={[]}
        onOpenApp={vi.fn()}
        onFocusWindow={vi.fn()}
        onToggleCommandPalette={vi.fn()}
        onToggleQuickSettings={vi.fn()}
        onToggleNotifications={vi.fn()}
        activeWorkspaceId="ws-1"
      />
    );

    expect(screen.getByRole('navigation', { name: /taskbar/i })).toBeDefined();
    expect(screen.getByText(/Nova/i)).toBeDefined();
  });

  it('renders in top panel mode', () => {
    render(
      <Launcher
        mode="panel"
        windows={[]}
        onOpenApp={vi.fn()}
        onFocusWindow={vi.fn()}
        onToggleCommandPalette={vi.fn()}
        onToggleQuickSettings={vi.fn()}
        onToggleNotifications={vi.fn()}
        activeWorkspaceId="ws-1"
      />
    );

    expect(screen.getByRole('navigation', { name: /top panel/i })).toBeDefined();
    expect(screen.getByText(/Activities/i)).toBeDefined();
  });
});
