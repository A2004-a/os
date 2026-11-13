import React, { useState, useEffect } from 'react';
import { usePreferences } from './state/preferenceStore';
import { useWindowManager } from './state/windowStore';
import { DesktopCanvas } from './components/shell/DesktopCanvas';
import { WindowFrame } from './components/shell/WindowFrame';
import { SnapPreview } from './components/shell/SnapPreview';
import { Launcher } from './components/shell/Launcher';
import { CommandPalette } from './components/shell/CommandPalette';
import { QuickSettings } from './components/shell/QuickSettings';
import { NotificationCenter } from './components/shell/NotificationCenter';
import { TaskSwitcher } from './components/shell/TaskSwitcher';
import { SnapZone, ShellNotification } from '@shared/types';

// Built-in Applications
import { FilesApp } from './components/apps/FilesApp';
import { NotesApp } from './components/apps/NotesApp';
import { TerminalApp } from './components/apps/TerminalApp';
import { SettingsApp } from './components/apps/SettingsApp';
import { SystemMonitorApp } from './components/apps/SystemMonitorApp';
import { CalculatorApp } from './components/apps/CalculatorApp';
import { ClockApp } from './components/apps/ClockApp';
import { MediaViewerApp } from './components/apps/MediaViewerApp';
import { SoftwareCenterApp } from './components/apps/SoftwareCenterApp';
import { DevConsoleApp } from './components/apps/DevConsoleApp';

export const App: React.FC = () => {
  const {
    preferences,
    setTheme,
    setAccent,
    setLauncherMode,
    toggleReducedMotion,
    toggleReducedTransparency,
  } = usePreferences();

  const {
    windows,
    activeWorkspaceId,
    workspaces,
    openApp,
    focusWindow,
    closeWindow,
    minimizeWindow,
    maximizeWindow,
    snapWindow,
    moveWindow,
    resizeWindow,
    switchWorkspace,
  } = useWindowManager();

  const [snapPreviewZone, setSnapPreviewZone] = useState<SnapZone | null>(null);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isQuickSettingsOpen, setIsQuickSettingsOpen] = useState(false);
  const [isNotificationsOpen, setIsNotificationsOpen] = useState(false);
  const [isTaskSwitcherOpen, setIsTaskSwitcherOpen] = useState(false);
  const [switcherIndex, setSwitcherIndex] = useState(0);

  const [notifications, setNotifications] = useState<ShellNotification[]>([
    {
      id: 'notif-1',
      title: 'Welcome to NovaOS',
      message: 'Hybrid desktop platform initialized with strict Electron isolation.',
      timestamp: Date.now() - 60000,
      sourceAppId: 'io.novaos.system',
      read: false,
      urgency: 'normal',
    },
  ]);

  const containerBounds = {
    width: window.innerWidth,
    height: window.innerHeight,
  };

  // Keyboard Shortcuts Listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Space or Cmd+Space: Command Palette
      if ((e.ctrlKey || e.metaKey) && e.code === 'Space') {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      }

      // Alt+Tab: Task Switcher
      if (e.altKey && e.key === 'Tab') {
        e.preventDefault();
        const activeWindows = windows.filter((w) => w.workspaceId === activeWorkspaceId);
        if (activeWindows.length > 0) {
          setIsTaskSwitcherOpen(true);
          setSwitcherIndex((prev) => (prev + 1) % activeWindows.length);
        }
      }

      // Snapping Shortcuts: Alt+Left, Alt+Right, Alt+Up
      const focusedWin = windows.find((w) => w.focused && w.workspaceId === activeWorkspaceId);
      if (focusedWin && e.altKey) {
        if (e.key === 'ArrowLeft') {
          e.preventDefault();
          snapWindow(focusedWin.id, 'left-half', containerBounds);
        } else if (e.key === 'ArrowRight') {
          e.preventDefault();
          snapWindow(focusedWin.id, 'right-half', containerBounds);
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          maximizeWindow(focusedWin.id, containerBounds);
        } else if (e.key === 'ArrowDown') {
          e.preventDefault();
          minimizeWindow(focusedWin.id);
        }
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'Alt' && isTaskSwitcherOpen) {
        setIsTaskSwitcherOpen(false);
        const activeWindows = windows.filter((w) => w.workspaceId === activeWorkspaceId);
        const target = activeWindows[switcherIndex];
        if (target) {
          focusWindow(target.id);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [windows, activeWorkspaceId, isTaskSwitcherOpen, switcherIndex, snapWindow, maximizeWindow, minimizeWindow, focusWindow, containerBounds]);

  const renderAppContent = (appId: string) => {
    switch (appId) {
      case 'io.novaos.files':
        return <FilesApp />;
      case 'io.novaos.notes':
        return <NotesApp />;
      case 'io.novaos.terminal':
        return <TerminalApp />;
      case 'io.novaos.settings':
        return (
          <SettingsApp
            preferences={preferences}
            onSetTheme={setTheme}
            onSetAccent={setAccent}
            onSetLauncherMode={setLauncherMode}
            onToggleReducedMotion={toggleReducedMotion}
            onToggleReducedTransparency={toggleReducedTransparency}
          />
        );
      case 'io.novaos.sysmon':
        return <SystemMonitorApp />;
      case 'io.novaos.calc':
        return <CalculatorApp />;
      case 'io.novaos.clock':
        return <ClockApp />;
      case 'io.novaos.media':
        return <MediaViewerApp />;
      case 'io.novaos.software':
        return <SoftwareCenterApp onOpenApp={openApp} />;
      case 'io.novaos.devconsole':
        return <DevConsoleApp />;
      default:
        return (
          <div style={{ padding: '24px', color: 'var(--nova-text-muted)' }}>
            App view not found: {appId}
          </div>
        );
    }
  };

  const activeWindows = windows.filter((w) => w.workspaceId === activeWorkspaceId);

  return (
    <div style={{ position: 'relative', width: '100vw', height: '100vh', overflow: 'hidden' }}>
      {/* Desktop Canvas */}
      <DesktopCanvas
        workspaces={workspaces}
        activeWorkspaceId={activeWorkspaceId}
        onSwitchWorkspace={switchWorkspace}
        onOpenApp={openApp}
        onOpenNote={() => openApp('io.novaos.notes', 'Notes')}
        onOpenTerminal={() => openApp('io.novaos.terminal', 'Terminal')}
      >
        {/* Render Open Windows */}
        {activeWindows.map((win) => (
          <WindowFrame
            key={win.id}
            window={win}
            containerBounds={containerBounds}
            onFocus={() => focusWindow(win.id)}
            onClose={() => closeWindow(win.id)}
            onMinimize={() => minimizeWindow(win.id)}
            onMaximize={() => maximizeWindow(win.id, containerBounds)}
            onSnap={(zone) => snapWindow(win.id, zone, containerBounds)}
            onMove={(x, y) => moveWindow(win.id, x, y)}
            onResize={(b) => resizeWindow(win.id, b)}
            onSnapPreview={setSnapPreviewZone}
          >
            {renderAppContent(win.appId)}
          </WindowFrame>
        ))}

        {/* Snap Ghost Preview Indicator */}
        <SnapPreview snapZone={snapPreviewZone} containerBounds={containerBounds} />
      </DesktopCanvas>

      {/* Launcher (Dock / Taskbar / Panel) */}
      <Launcher
        mode={preferences.launcherMode}
        windows={activeWindows}
        onOpenApp={openApp}
        onFocusWindow={focusWindow}
        onToggleCommandPalette={() => setIsCommandPaletteOpen((p) => !p)}
        onToggleQuickSettings={() => setIsQuickSettingsOpen((p) => !p)}
        onToggleNotifications={() => setIsNotificationsOpen((p) => !p)}
        activeWorkspaceId={activeWorkspaceId}
        unreadNotificationsCount={notifications.filter((n) => !n.read).length}
      />

      {/* Global Command Palette */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        onOpenApp={openApp}
        onSwitchWorkspace={switchWorkspace}
        onToggleTheme={() => setTheme(preferences.theme === 'dark' ? 'light' : 'dark')}
      />

      {/* Quick Settings Drawer */}
      <QuickSettings
        isOpen={isQuickSettingsOpen}
        onClose={() => setIsQuickSettingsOpen(false)}
        preferences={preferences}
        onSetTheme={setTheme}
        onSetAccent={setAccent}
        onSetLauncherMode={setLauncherMode}
        onToggleReducedMotion={toggleReducedMotion}
        onToggleReducedTransparency={toggleReducedTransparency}
      />

      {/* Notification Center */}
      <NotificationCenter
        isOpen={isNotificationsOpen}
        onClose={() => setIsNotificationsOpen(false)}
        notifications={notifications}
        onDismiss={(id) => setNotifications((prev) => prev.filter((n) => n.id !== id))}
        onClearAll={() => setNotifications([])}
      />

      {/* Task Switcher (Alt+Tab) */}
      <TaskSwitcher
        isOpen={isTaskSwitcherOpen}
        windows={activeWindows}
        selectedIndex={switcherIndex}
        onSelectIndex={(idx) => {
          setSwitcherIndex(idx);
          const target = activeWindows[idx];
          if (target) focusWindow(target.id);
          setIsTaskSwitcherOpen(false);
        }}
      />
    </div>
  );
};

