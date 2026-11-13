import React, { useState } from 'react';
import { Workspace } from '@shared/types';
import { Folder, FileText, Terminal, Activity, Trash2, Image, Sparkles, Plus, Settings } from 'lucide-react';

interface DesktopIcon {
  id: string;
  name: string;
  icon: React.ReactNode;
  defaultX: number;
  defaultY: number;
  action: () => void;
}

interface DesktopCanvasProps {
  workspaces: Workspace[];
  activeWorkspaceId: string;
  onSwitchWorkspace: (id: string) => void;
  onOpenApp: (appId: string, name: string) => void;
  onOpenNote: () => void;
  onOpenTerminal: () => void;
  wallpaperId?: string;
  onSelectWallpaper?: (id: string) => void;
  children: React.ReactNode;
}

export const WALLPAPERS: Record<string, string> = {
  cosmic: 'radial-gradient(ellipse at 50% 30%, #151d2f 0%, #080c14 80%)',
  obsidian: 'radial-gradient(ellipse at 70% 20%, #1e1e24 0%, #09090b 80%)',
  emerald: 'radial-gradient(ellipse at 40% 40%, #064e3b 0%, #022c22 45%, #050d0a 85%)',
  solar: 'radial-gradient(ellipse at 60% 30%, #78350f 0%, #451a03 40%, #0c0a09 85%)',
  frost: 'radial-gradient(ellipse at 50% 50%, #1e293b 0%, #0f172a 60%, #020617 100%)',
};

export const DesktopCanvas: React.FC<DesktopCanvasProps> = ({
  workspaces,
  activeWorkspaceId,
  onSwitchWorkspace,
  onOpenApp,
  onOpenNote,
  onOpenTerminal,
  wallpaperId = 'cosmic',
  onSelectWallpaper,
  children,
}) => {
  const [selectedIconId, setSelectedIconId] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const [selectionBox, setSelectionBox] = useState<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);
  const [showIcons, setShowIcons] = useState(true);
  const [currentWallpaper, setCurrentWallpaper] = useState(wallpaperId);

  // Icon coordinates
  const [iconPositions, setIconPositions] = useState<Record<string, { x: number; y: number }>>(() => {
    return {
      'icon-home': { x: 24, y: 70 },
      'icon-welcome': { x: 24, y: 170 },
      'icon-term': { x: 24, y: 270 },
      'icon-sysmon': { x: 24, y: 370 },
      'icon-trash': { x: 24, y: 470 },
    };
  });

  const desktopIcons: DesktopIcon[] = [
    {
      id: 'icon-home',
      name: 'Nova Home',
      icon: <Folder size={32} color="#06b6d4" />,
      defaultX: 24,
      defaultY: 70,
      action: () => onOpenApp('io.novaos.files', 'Files'),
    },
    {
      id: 'icon-welcome',
      name: 'Welcome Guide',
      icon: <FileText size={32} color="#3b82f6" />,
      defaultX: 24,
      defaultY: 170,
      action: () => onOpenApp('io.novaos.notes', 'Notes'),
    },
    {
      id: 'icon-term',
      name: 'Terminal',
      icon: <Terminal size={32} color="#10b981" />,
      defaultX: 24,
      defaultY: 270,
      action: onOpenTerminal,
    },
    {
      id: 'icon-sysmon',
      name: 'System Monitor',
      icon: <Activity size={32} color="#f43f5e" />,
      defaultX: 24,
      defaultY: 370,
      action: () => onOpenApp('io.novaos.sysmon', 'System Monitor'),
    },
    {
      id: 'icon-trash',
      name: 'Trash Bin',
      icon: <Trash2 size={32} color="#94a3b8" />,
      defaultX: 24,
      defaultY: 470,
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

  const handleNextWallpaper = () => {
    const keys = Object.keys(WALLPAPERS);
    const currIdx = keys.indexOf(currentWallpaper);
    const nextKey = keys[(currIdx + 1) % keys.length];
    setCurrentWallpaper(nextKey);
    if (onSelectWallpaper) onSelectWallpaper(nextKey);
  };

  // Dragging individual desktop icon
  const handleIconDrag = (iconId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedIconId(iconId);

    const startX = e.clientX;
    const startY = e.clientY;
    const initialPos = iconPositions[iconId] || { x: 24, y: 70 };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      setIconPositions((prev) => ({
        ...prev,
        [iconId]: {
          x: Math.max(12, Math.min(window.innerWidth - 90, initialPos.x + dx)),
          y: Math.max(50, Math.min(window.innerHeight - 100, initialPos.y + dy)),
        },
      }));
    };

    const handleMouseUp = () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  const bgStyle = WALLPAPERS[currentWallpaper] || WALLPAPERS.cosmic;

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
        background: bgStyle,
        userSelect: 'none',
        transition: 'background 0.5s ease',
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

      {/* Desktop Icons (Draggable & Selectable) */}
      {showIcons &&
        desktopIcons.map((icon) => {
          const isSelected = selectedIconId === icon.id;
          const pos = iconPositions[icon.id] || { x: icon.defaultX, y: icon.defaultY };

          return (
            <div
              key={icon.id}
              onMouseDown={(e) => handleIconDrag(icon.id, e)}
              onClick={(e) => {
                e.stopPropagation();
                setSelectedIconId(icon.id);
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                icon.action();
              }}
              style={{
                position: 'absolute',
                left: `${pos.x}px`,
                top: `${pos.y}px`,
                width: '76px',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: '4px',
                padding: '8px 4px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: isSelected ? 'var(--nova-surface-raised)' : 'transparent',
                border: `1px solid ${isSelected ? 'var(--nova-border-focus)' : 'transparent'}`,
                cursor: 'grab',
                textAlign: 'center',
                zIndex: 10,
                transition: 'background-color var(--nova-duration-fast) var(--nova-ease-standard)',
              }}
            >
              {icon.icon}
              <span
                style={{
                  fontSize: '11px',
                  color: 'var(--nova-text-primary)',
                  textShadow: '0 1px 3px rgba(0, 0, 0, 0.85)',
                  wordBreak: 'break-word',
                  lineHeight: 1.2,
                  pointerEvents: 'none',
                }}
              >
                {icon.name}
              </span>
            </div>
          );
        })}

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
            width: '200px',
            backgroundColor: 'var(--nova-surface-glass)',
            backdropFilter: 'var(--nova-blur-glass)',
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
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Plus size={14} color="var(--nova-accent-primary)" />
            New Note
          </button>
          <button
            onClick={() => { onOpenTerminal(); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Terminal size={14} color="#10b981" />
            Open Terminal Here
          </button>
          <div style={{ height: '1px', backgroundColor: 'var(--nova-border-subtle)', margin: '4px 0' }} />
          <button
            onClick={() => { handleNextWallpaper(); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Image size={14} color="#f59e0b" />
            Next Wallpaper
          </button>
          <button
            onClick={() => { setShowIcons(!showIcons); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Sparkles size={14} color="#a855f7" />
            {showIcons ? 'Hide Desktop Icons' : 'Show Desktop Icons'}
          </button>
          <div style={{ height: '1px', backgroundColor: 'var(--nova-border-subtle)', margin: '4px 0' }} />
          <button
            onClick={() => { onOpenApp('io.novaos.settings', 'Settings'); setContextMenu(null); }}
            style={{ width: '100%', textAlign: 'left', padding: '6px 10px', fontSize: 'var(--nova-text-xs)', borderRadius: 'var(--nova-radius-sm)', cursor: 'pointer', color: 'var(--nova-text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Settings size={14} color="#94a3b8" />
            Display & Settings
          </button>
        </div>
      )}

      {/* Render Open Windows */}
      {children}
    </div>
  );
};
