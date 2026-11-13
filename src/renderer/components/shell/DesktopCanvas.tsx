import React, { useState } from 'react';
import { Workspace } from '@shared/types';
import { Folder, FileText, Terminal, Activity, Trash2 } from 'lucide-react';

interface DesktopIcon {
  id: string;
  name: string;
  icon: React.ReactNode;
  action: () => void;
}

interface DesktopCanvasProps {
  workspaces: Workspace[];
  activeWorkspaceId: string;
  onSwitchWorkspace: (id: string) => void;
  onOpenApp: (appId: string, name: string) => void;
  onOpenNote: () => void;
  onOpenTerminal: () => void;
  children: React.ReactNode;
}

export const DesktopCanvas: React.FC<DesktopCanvasProps> = ({
  workspaces,
  activeWorkspaceId,
  onSwitchWorkspace,
  onOpenApp,
  onOpenNote,
  onOpenTerminal,
  children,
}) => {
  const [selectedIconId, setSelectedIconId] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const [selectionBox, setSelectionBox] = useState<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);

  const desktopIcons: DesktopIcon[] = [
    {
      id: 'icon-home',
      name: 'Nova Home',
      icon: <Folder size={32} color="#06b6d4" />,
      action: () => onOpenApp('io.novaos.files', 'Files'),
    },
    {
      id: 'icon-welcome',
      name: 'Welcome Guide',
      icon: <FileText size={32} color="#3b82f6" />,
      action: () => onOpenApp('io.novaos.notes', 'Notes'),
    },
    {
      id: 'icon-term',
      name: 'Terminal',
      icon: <Terminal size={32} color="#10b981" />,
      action: onOpenTerminal,
    },
    {
      id: 'icon-sysmon',
      name: 'System Monitor',
      icon: <Activity size={32} color="#f43f5e" />,
      action: () => onOpenApp('io.novaos.sysmon', 'System Monitor'),
    },
    {
      id: 'icon-trash',
      name: 'Trash Bin',
      icon: <Trash2 size={32} color="#94a3b8" />,
      action: () => onOpenApp('io.novaos.files', 'Files'),
    },
  ];

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0 && (e.target as HTMLElement).id === 'desktop-canvas-bg') {
      setSelectedIconId(null);
      setContextMenu(null);
      setSelectionBox({
        startX: e.clientX,
        startY: e.clientY,
        currentX: e.clientX,
        currentY: e.clientY,
      });

      const handleMouseMove = (moveEvent: MouseEvent) => {
        setSelectionBox((prev) => (prev ? { ...prev, currentX: moveEvent.clientX, currentY: moveEvent.clientY } : null));
      };

      const handleMouseUp = () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
        setSelectionBox(null);
      };

      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  return (
    <div
      id="desktop-canvas-bg"
      onMouseDown={handleMouseDown}
      onContextMenu={handleContextMenu}
      onClick={() => setContextMenu(null)}
      style={{
        position: 'relative',
        width: '100vw',
        height: '100vh',
        overflow: 'hidden',
        background: 'radial-gradient(ellipse at 50% 30%, #151d2f 0%, #080c14 80%)',
        userSelect: 'none',
      }}
    >
      {/* Workspace Indicator Pill */}
      <div
        style={{
          position: 'absolute',
          top: '12px',
          left: '16px',
          display: 'flex',
          gap: '4px',
          padding: '4px 6px',
          backgroundColor: 'var(--nova-surface-glass)',
          backdropFilter: 'var(--nova-blur-glass)',
          borderRadius: 'var(--nova-radius-full)',
          border: '1px solid var(--nova-border-subtle)',
          zIndex: 50,
        }}
      >
        {workspaces.map((ws) => {
          const isActive = ws.id === activeWorkspaceId;
          return (
            <button
              key={ws.id}
              onClick={() => onSwitchWorkspace(ws.id)}
              style={{
                padding: '4px 10px',
                borderRadius: 'var(--nova-radius-full)',
                backgroundColor: isActive ? 'var(--nova-accent-primary)' : 'transparent',
                color: isActive ? 'var(--nova-text-inverse)' : 'var(--nova-text-secondary)',
                fontSize: 'var(--nova-text-xs)',
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
              }}
            >
              {ws.name}
            </button>
          );
        })}
      </div>

      {/* Desktop Icons Grid */}
      <div
        style={{
          position: 'absolute',
          top: '60px',
          left: '20px',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px',
          zIndex: 10,
        }}
      >
        {desktopIcons.map((icon) => {
          const isSelected = selectedIconId === icon.id;
          return (
            <div
              key={icon.id}
              onClick={(e) => {
                e.stopPropagation();
                setSelectedIconId(icon.id);
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                icon.action();
              }}
              style={{
                width: '76px',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: '4px',
                padding: '8px 4px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                border: `1px solid ${isSelected ? 'var(--nova-border-focus)' : 'transparent'}`,
                cursor: 'pointer',
                textAlign: 'center',
              }}
            >
              {icon.icon}
              <span
                style={{
                  fontSize: '11px',
                  color: 'var(--nova-text-primary)',
                  textShadow: '0 1px 3px rgba(0, 0, 0, 0.8)',
                  wordBreak: 'break-word',
                  lineHeight: 1.2,
                }}
              >
                {icon.name}
              </span>
            </div>
          );
        })}
      </div>

      {/* Rubber-band Drag Selection Box */}
      {selectionBox && (
        <div
          style={{
            position: 'absolute',
            left: `${Math.min(selectionBox.startX, selectionBox.currentX)}px`,
            top: `${Math.min(selectionBox.startY, selectionBox.currentY)}px`,
            width: `${Math.abs(selectionBox.currentX - selectionBox.startX)}px`,
            height: `${Math.abs(selectionBox.currentY - selectionBox.startY)}px`,
            backgroundColor: 'var(--nova-accent-glow)',
            border: '1px solid var(--nova-accent-primary)',
            borderRadius: '2px',
            pointerEvents: 'none',
            zIndex: 90,
          }}
        />
      )}

      {/* Desktop Context Menu */}
      {contextMenu && (
        <div
          style={{
            position: 'fixed',
            left: `${contextMenu.x}px`,
            top: `${contextMenu.y}px`,
            width: '180px',
            backgroundColor: 'var(--nova-surface-base)',
            border: '1px solid var(--nova-border-strong)',
            borderRadius: 'var(--nova-radius-lg)',
            boxShadow: 'var(--nova-shadow-lg)',
            padding: '6px',
            zIndex: 'var(--nova-z-popover)' as any,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={() => { onOpenNote(); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            + Create New Note
          </button>
          <button
            onClick={() => { onOpenTerminal(); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            Open Terminal Here
          </button>
          <div style={{ height: '1px', backgroundColor: 'var(--nova-border-subtle)', margin: '4px 0' }} />
          <button
            onClick={() => { onOpenApp('io.novaos.settings', 'Settings'); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            Display & Settings
          </button>
        </div>
      )}

      {/* Render All Open Windows */}
      {children}
    </div>
  );
};

