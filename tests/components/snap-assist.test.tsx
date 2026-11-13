import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SnapAssist } from '../../src/renderer/components/shell/SnapAssist';
import { ShellWindow } from '@shared/types';

describe('SnapAssist Component', () => {
  const candidateWindows: ShellWindow[] = [
    {
      id: 'win-1',
      appId: 'io.novaos.notes',
      title: 'Project Notes',
      workspaceId: 'ws-1',
      bounds: { x: 50, y: 50, width: 600, height: 400 },
      state: 'normal',
      zIndex: 101,
      focused: false,
      permissions: [],
    },
    {
      id: 'win-2',
      appId: 'io.novaos.terminal',
      title: 'Host Terminal',
      workspaceId: 'ws-1',
      bounds: { x: 100, y: 100, width: 600, height: 400 },
      state: 'normal',
      zIndex: 102,
      focused: false,
      permissions: [],
    },
  ];

  it('renders snap assist candidates when remaining zone is provided', () => {
    const handleSelect = vi.fn();

    render(
      <SnapAssist
        remainingZone="right-half"
        candidateWindows={candidateWindows}
        containerBounds={{ width: 1200, height: 800 }}
        onSelectWindow={handleSelect}
        onDismiss={vi.fn()}
      />
    );

    expect(screen.getByRole('region', { name: /snap assist/i })).toBeDefined();
    expect(screen.getByText('Project Notes')).toBeDefined();
    expect(screen.getByText('Host Terminal')).toBeDefined();

    // Select candidate
    const notesBtn = screen.getByRole('button', { name: /snap project notes/i });
    fireEvent.click(notesBtn);
    expect(handleSelect).toHaveBeenCalledWith('win-1', 'right-half');
  });

  it('renders nothing when candidate list is empty', () => {
    const { container } = render(
      <SnapAssist
        remainingZone="right-half"
        candidateWindows={[]}
        containerBounds={{ width: 1200, height: 800 }}
        onSelectWindow={vi.fn()}
        onDismiss={vi.fn()}
      />
    );

    expect(container.firstChild).toBeNull();
  });
});
