import React, { useState, useEffect, useCallback } from 'react';
import { usePreferences } from './state/preferenceStore';
import { useWindowManager } from './state/windowStore';
import { DesktopCanvas } from './components/shell/DesktopCanvas';
import { WindowFrame } from './components/shell/WindowFrame';
import { SnapPreview } from './components/shell/SnapPreview';
import { SnapAssist } from './components/shell/SnapAssist';
import { Launcher } from './components/shell/Launcher';
import { NovaLauncher } from './components/shell/NovaLauncher';
import { CalendarPopover } from './components/shell/CalendarPopover';
import { QuickSettings } from './components/shell/QuickSettings';
import { NotificationCenter } from './components/shell/NotificationCenter';
import { TaskSwitcher } from './components/shell/TaskSwitcher';
import { ClipboardHistory, ClipboardItem } from './components/shell/ClipboardHistory';
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
  const [snapAssistRemainingZone, setSnapAssistRemainingZone] = useState<SnapZone | null>(null);

  const [isLauncherOpen, setIsLauncherOpen] = useState(false);
  const [isCalendarOpen, setIsCalendarOpen] = useState(false);
  const [isQuickSettingsOpen, setIsQuickSettingsOpen] = useState(false);
  const [isNotificationsOpen, setIsNotificationsOpen] = useState(false);
  const [isTaskSwitcherOpen, setIsTaskSwitcherOpen] = useState(false);
  const [isClipboardOpen, setIsClipboardOpen] = useState(false);
  const [switcherIndex, setSwitcherIndex] = useState(0);

  const [focusModeActive, setFocusModeActive] = useState(false);
  const [currentWallpaper, setCurrentWallpaper] = useState('cosmic');

  const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>([
    { id: 'clip-1', text: 'https://novaos.internal/docs/security', timestamp: Date.now() - 300000 },
    { id: 'clip-2', text: 'npm run test:security', timestamp: Date.now() - 120000 },
    { id: 'clip-3', text: 'One Desktop. Three Worlds. One Experience.', timestamp: Date.now() - 60000 },
  ]);

  const [notifications, setNotifications] = useState<ShellNotification[]>([
    {
      id: 'notif-1',
      title: 'Welcome to NovaOS',
      message: 'Hybrid desktop platform initialized with strict Electron isolation and Nova Surface design language.',
      timestamp: Date.now() - 60000,
      sourceAppId: 'io.novaos.system',
      read: false,
      urgency: 'normal',
    },
    {
      id: 'notif-2',
      title: 'System Security Audit',
      message: 'Zero-trust capability broker active. Host folder access confined to approved boundaries.',
      timestamp: Date.now() - 30000,
      sourceAppId: 'io.novaos.security',
      read: false,
      urgency: 'normal',
    },
  ]);

  const containerBounds = {
    width: window.innerWidth,
    height: window.innerHeight,
  };

  const activeWindows = windows.filter((w) => w.workspaceId === activeWorkspaceId);

  // Focus Group: Minimize background windows to focus on target
  const handleFocusGroup = useCallback((targetWindowId: string) => {
    windows.forEach((w) => {
      if (w.id !== targetWindowId && w.workspaceId === activeWorkspaceId && w.state !== 'minimized') {
        minimizeWindow(w.id);
      }
    });
    focusWindow(targetWindowId);
  }, [windows, activeWorkspaceId, minimizeWindow, focusWindow]);

  // Show Desktop Peek
  const handleShowDesktop = useCallback(() => {
    const hasUnminimized = activeWindows.some((w) => w.state !== 'minimized');
    if (hasUnminimized) {
      activeWindows.forEach((w) => minimizeWindow(w.id));
    } else {
      activeWindows.forEach((w) => focusWindow(w.id));
    }
  }, [activeWindows, minimizeWindow, focusWindow]);

  // Handle Window Snap with Snap Assist Trigger
  const handleSnap = useCallback((windowId: string, zone: SnapZone) => {
    snapWindow(windowId, zone, containerBounds);

    // Determine complementary remaining zone for Snap Assist
    let remaining: SnapZone | null = null;
    if (zone === 'left-half') remaining = 'right-half';
    else if (zone === 'right-half') remaining = 'left-half';
    else if (zone === 'top-half') remaining = 'bottom-half';
    else if (zone === 'bottom-half') remaining = 'top-half';
    else if (zone === 'top-left') remaining = 'top-right';
    else if (zone === 'top-right') remaining = 'bottom-right';
    else if (zone === 'bottom-left') remaining = 'bottom-right';
    else if (zone === 'bottom-right') remaining = 'bottom-left';

    const candidates = activeWindows.filter((w) => w.id !== windowId && w.state !== 'minimized');
    if (remaining && candidates.length > 0) {
      setSnapAssistRemainingZone(remaining);
    } else {
      setSnapAssistRemainingZone(null);
    }
  }, [snapWindow, containerBounds, activeWindows]);

  // Snap Assist Select
  const handleSnapAssistSelect = useCallback((candidateId: string, zone: SnapZone) => {
    snapWindow(candidateId, zone, containerBounds);
    setSnapAssistRemainingZone(null);
  }, [snapWindow, containerBounds]);

  // Keyboard Shortcuts Listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Space or Win key: Toggle Nova Launcher & Search
      if ((e.ctrlKey || e.metaKey) && e.code === 'Space') {
        e.preventDefault();
        setIsLauncherOpen((prev) => !prev);
      }

      // Alt+Tab: Task Switcher
      if (e.altKey && e.key === 'Tab') {
        e.preventDefault();
        if (activeWindows.length > 0) {
          setIsTaskSwitcherOpen(true);
          setSwitcherIndex((prev) => (prev + 1) % activeWindows.length);
        }
      }

      // Win+V or Alt+V: Clipboard History
      if ((e.metaKey || e.altKey) && e.key.toLowerCase() === 'v') {
        e.preventDefault();
        setIsClipboardOpen((prev) => !prev);
      }

      // Escape: Dismiss active popovers
      if (e.key === 'Escape') {
        setIsLauncherOpen(false);
        setIsCalendarOpen(false);
        setIsQuickSettingsOpen(false);
        setIsNotificationsOpen(false);
        setIsClipboardOpen(false);
        setSnapAssistRemainingZone(null);
      }

      // Snapping Shortcuts: Alt+Left, Alt+Right, Alt+Up, Alt+Down
      const focusedWin = activeWindows.find((w) => w.focused);
      if (focusedWin && e.altKey) {
        if (e.key === 'ArrowLeft') {
          e.preventDefault();
          handleSnap(focusedWin.id, 'left-half');
        } else if (e.key === 'ArrowRight') {
          e.preventDefault();
          handleSnap(focusedWin.id, 'right-half');
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
  }, [activeWindows, isTaskSwitcherOpen, switcherIndex, handleSnap, maximizeWindow, minimizeWindow, focusWindow, containerBounds]);

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
        wallpaperId={currentWallpaper}
        onSelectWallpaper={setCurrentWallpaper}
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
            onSnap={(zone) => handleSnap(win.id, zone)}
            onMove={(x, y) => moveWindow(win.id, x, y)}
            onResize={(b) => resizeWindow(win.id, b)}
            onSnapPreview={setSnapPreviewZone}
            onFocusGroup={() => handleFocusGroup(win.id)}
          >
            {renderAppContent(win.appId)}
          </WindowFrame>
        ))}

        {/* Snap Ghost Preview Indicator */}
        <SnapPreview snapZone={snapPreviewZone} containerBounds={containerBounds} />

        {/* Snap Assist Floating Picker */}
        {snapAssistRemainingZone && (
          <SnapAssist
            remainingZone={snapAssistRemainingZone}
            candidateWindows={activeWindows.filter((w) => w.state !== 'snapped' || w.snapZone !== snapAssistRemainingZone)}
            containerBounds={containerBounds}
            onSelectWindow={handleSnapAssistSelect}
            onDismiss={() => setSnapAssistRemainingZone(null)}
          />
        )}
      </DesktopCanvas>

      {/* Nova Bar (Centered / Left Taskbar or Dock / Panel) */}
      <Launcher
        mode={preferences.launcherMode}
        windows={activeWindows}
        onOpenApp={openApp}
        onFocusWindow={focusWindow}
        onCloseWindow={closeWindow}
        onToggleCommandPalette={() => setIsLauncherOpen((p) => !p)}
        onToggleQuickSettings={() => setIsQuickSettingsOpen((p) => !p)}
        onToggleNotifications={() => setIsNotificationsOpen((p) => !p)}
        onToggleCalendar={() => setIsCalendarOpen((p) => !p)}
        onShowDesktop={handleShowDesktop}
        activeWorkspaceId={activeWorkspaceId}
        unreadNotificationsCount={notifications.filter((n) => !n.read).length}
        taskbarAlignment="center"
        privacyIndicatorActive={false}
      />

      {/* Windows 11-Grade Nova Launcher & Universal Search */}
      <NovaLauncher
        isOpen={isLauncherOpen}
        onClose={() => setIsLauncherOpen(false)}
        onOpenApp={openApp}
        onSwitchWorkspace={switchWorkspace}
        onToggleTheme={() => setTheme(preferences.theme === 'dark' ? 'light' : 'dark')}
        onOpenNote={() => openApp('io.novaos.notes', 'Notes')}
        onOpenTerminal={() => openApp('io.novaos.terminal', 'Terminal')}
      />

      {/* Interactive Calendar Popover */}
      <CalendarPopover
        isOpen={isCalendarOpen}
        onClose={() => setIsCalendarOpen(false)}
        onOpenApp={openApp}
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
        focusMode={focusModeActive}
        onToggleFocusMode={() => setFocusModeActive((prev) => !prev)}
      />

      {/* Notification Center */}
      <NotificationCenter
        isOpen={isNotificationsOpen}
        onClose={() => setIsNotificationsOpen(false)}
        notifications={notifications}
        onDismiss={(id) => setNotifications((prev) => prev.filter((n) => n.id !== id))}
        onClearAll={() => setNotifications([])}
      />

      {/* In-App Clipboard History (Win+V) */}
      <ClipboardHistory
        isOpen={isClipboardOpen}
        onClose={() => setIsClipboardOpen(false)}
        items={clipboardItems}
        onSelectItem={(_text) => {
          setIsClipboardOpen(false);
        }}
        onClearAll={() => setClipboardItems([])}
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
