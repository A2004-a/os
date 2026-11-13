import React from 'react';
import { LauncherMode, ShellWindow } from '@shared/types';
import {
  Folder,
  FileText,
  Terminal,
  Settings,
  Activity,
  Calculator,
  Clock,
  Package,
  Search,
  Sliders,
  Bell,
} from 'lucide-react';

export interface AppItem {
  id: string;
  name: string;
  icon: React.ReactNode;
}

export const PINNED_APPS: AppItem[] = [
  { id: 'io.novaos.files', name: 'Files', icon: <Folder size={20} color="#06b6d4" /> },
  { id: 'io.novaos.notes', name: 'Notes', icon: <FileText size={20} color="#3b82f6" /> },
  { id: 'io.novaos.terminal', name: 'Terminal', icon: <Terminal size={20} color="#10b981" /> },
  { id: 'io.novaos.calc', name: 'Calculator', icon: <Calculator size={20} color="#f59e0b" /> },
  { id: 'io.novaos.clock', name: 'Clock', icon: <Clock size={20} color="#a855f7" /> },
  { id: 'io.novaos.sysmon', name: 'System Monitor', icon: <Activity size={20} color="#f43f5e" /> },
  { id: 'io.novaos.software', name: 'Software Center', icon: <Package size={20} color="#38bdf8" /> },
  { id: 'io.novaos.settings', name: 'Settings', icon: <Settings size={20} color="#94a3b8" /> },
];

interface LauncherProps {
  mode: LauncherMode;
  windows: ShellWindow[];
  onOpenApp: (appId: string, name: string) => void;
  onFocusWindow: (windowId: string) => void;
  onToggleCommandPalette: () => void;
  onToggleQuickSettings: () => void;
  onToggleNotifications: () => void;
  activeWorkspaceId: string;
  unreadNotificationsCount?: number;
}

export const Launcher: React.FC<LauncherProps> = ({
  mode,
  windows,
  onOpenApp,
  onFocusWindow,
  onToggleCommandPalette,
  onToggleQuickSettings,
  onToggleNotifications,
  unreadNotificationsCount = 0,
}) => {
  const isAppOpen = (appId: string) => windows.some((w) => w.appId === appId);
  const getAppWindow = (appId: string) => windows.find((w) => w.appId === appId);

  const handleAppClick = (app: AppItem) => {
    const existing = getAppWindow(app.id);
    if (existing) {
      onFocusWindow(existing.id);
    } else {
      onOpenApp(app.id, app.name);
    }
  };

  // 1. FLOATING DOCK MODE (macOS-Style Ergonomics)
  if (mode === 'dock') {
    return (
      <div
        role="navigation"
        aria-label="Application Dock"
        style={{
          position: 'absolute',
          bottom: '12px',
          left: '50%',
          transform: 'translateX(-50%)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '8px 14px',
          backgroundColor: 'var(--nova-surface-glass)',
          backdropFilter: 'var(--nova-blur-glass)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          zIndex: 'var(--nova-z-dock)' as any,
          transition: 'all var(--nova-duration-normal) var(--nova-ease-standard)',
        }}
      >
        {/* Command Search Pill */}
        <button
          onClick={onToggleCommandPalette}
          title="Command Palette (Ctrl+Space)"
          aria-label="Open Command Palette"
          style={{
            width: '40px',
            height: '40px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: 'var(--nova-radius-lg)',
            backgroundColor: 'var(--nova-surface-raised)',
            border: '1px solid var(--nova-border-subtle)',
            color: 'var(--nova-accent-primary)',
            cursor: 'pointer',
          }}
        >
          <Search size={18} />
        </button>

        <div style={{ width: '1px', height: '28px', backgroundColor: 'var(--nova-border-subtle)', margin: '0 4px' }} />

        {/* Pinned App Icons */}
        {PINNED_APPS.map((app) => {
          const open = isAppOpen(app.id);
          return (
            <button
              key={app.id}
              onClick={() => handleAppClick(app)}
              title={app.name}
              aria-label={app.name}
              style={{
                position: 'relative',
                width: '44px',
                height: '44px',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: 'var(--nova-radius-lg)',
                backgroundColor: 'var(--nova-surface-base)',
                border: '1px solid var(--nova-border-subtle)',
                cursor: 'pointer',
                transition: 'transform var(--nova-duration-fast) var(--nova-ease-spring)',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.transform = 'translateY(-4px) scale(1.08)')}
              onMouseLeave={(e) => (e.currentTarget.style.transform = 'translateY(0) scale(1)')}
            >
              {app.icon}
              {open && (
                <span
                  style={{
                    position: 'absolute',
                    bottom: '2px',
                    width: '4px',
                    height: '4px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--nova-accent-primary)',
                  }}
                />
              )}
            </button>
          );
        })}

        <div style={{ width: '1px', height: '28px', backgroundColor: 'var(--nova-border-subtle)', margin: '0 4px' }} />

        {/* Notification & Quick Settings */}
        <button
          onClick={onToggleNotifications}
          title="Notification Center"
          aria-label="Notification Center"
          style={{
            position: 'relative',
            width: '40px',
            height: '40px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: 'var(--nova-radius-lg)',
            backgroundColor: 'var(--nova-surface-raised)',
            border: '1px solid var(--nova-border-subtle)',
            color: 'var(--nova-text-secondary)',
            cursor: 'pointer',
          }}
        >
          <Bell size={17} />
          {unreadNotificationsCount > 0 && (
            <span
              style={{
                position: 'absolute',
                top: '4px',
                right: '4px',
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#f43f5e',
              }}
            />
          )}
        </button>

        <button
          onClick={onToggleQuickSettings}
          title="Quick Settings"
          aria-label="Quick Settings"
          style={{
            width: '40px',
            height: '40px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: 'var(--nova-radius-lg)',
            backgroundColor: 'var(--nova-surface-raised)',
            border: '1px solid var(--nova-border-subtle)',
            color: 'var(--nova-text-secondary)',
            cursor: 'pointer',
          }}
        >
          <Sliders size={17} />
        </button>
      </div>
    );
  }

  // 2. TASKBAR MODE (Windows-Style Productivity)
  if (mode === 'taskbar') {
    return (
      <div
        role="navigation"
        aria-label="Taskbar"
        style={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          right: 0,
          height: '44px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 12px',
          backgroundColor: 'var(--nova-surface-base)',
          borderTop: '1px solid var(--nova-border-subtle)',
          boxShadow: 'var(--nova-shadow-md)',
          zIndex: 'var(--nova-z-dock)' as any,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          {/* Nova Start Pill */}
          <button
            onClick={onToggleCommandPalette}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 12px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: 'var(--nova-surface-raised)',
              border: '1px solid var(--nova-border-subtle)',
              color: 'var(--nova-accent-primary)',
              fontWeight: 600,
              fontSize: 'var(--nova-text-sm)',
              cursor: 'pointer',
            }}
          >
            <span style={{ fontSize: '14px' }}>✦</span> Nova
          </button>

          {/* Running Windows Tabs */}
          {windows.map((win) => (
            <button
              key={win.id}
              onClick={() => onFocusWindow(win.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '4px 10px',
                maxWidth: '180px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: win.focused ? 'var(--nova-surface-raised)' : 'transparent',
                border: `1px solid ${win.focused ? 'var(--nova-border-strong)' : 'transparent'}`,
                color: win.focused ? 'var(--nova-text-primary)' : 'var(--nova-text-secondary)',
                fontSize: 'var(--nova-text-xs)',
                cursor: 'pointer',
              }}
            >
              {win.icon && <span>{win.icon}</span>}
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {win.title}
              </span>
            </button>
          ))}
        </div>

        {/* System Tray Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <button
            onClick={onToggleNotifications}
            style={{ padding: '6px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
          >
            <Bell size={16} />
          </button>
          <button
            onClick={onToggleQuickSettings}
            style={{ padding: '6px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
          >
            <Sliders size={16} />
          </button>
        </div>
      </div>
    );
  }

  // 3. TOP PANEL MODE (Linux-Style Clean Status)
  return (
    <div
      role="navigation"
      aria-label="Top Panel"
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        height: '34px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 16px',
        backgroundColor: 'var(--nova-surface-base)',
        borderBottom: '1px solid var(--nova-border-subtle)',
        zIndex: 'var(--nova-z-dock)' as any,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <button
          onClick={onToggleCommandPalette}
          style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-accent-primary)', cursor: 'pointer' }}
        >
          ✦ Activities
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          {PINNED_APPS.slice(0, 5).map((app) => (
            <button
              key={app.id}
              onClick={() => handleAppClick(app)}
              title={app.name}
              style={{ padding: '4px', color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
            >
              {app.icon}
            </button>
          ))}
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
        <button
          onClick={onToggleNotifications}
          style={{ color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
        >
          <Bell size={15} />
        </button>
        <button
          onClick={onToggleQuickSettings}
          style={{ color: 'var(--nova-text-secondary)', cursor: 'pointer' }}
        >
          <Sliders size={15} />
        </button>
      </div>
    </div>
  );
};
