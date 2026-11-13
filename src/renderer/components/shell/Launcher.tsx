import React, { useState, useEffect } from 'react';
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
  Wifi,
  Volume2,
  VolumeX,
  Battery,
  X,
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
  onCloseWindow?: (windowId: string) => void;
  onToggleCommandPalette: () => void;
  onToggleQuickSettings: () => void;
  onToggleNotifications: () => void;
  onToggleCalendar?: () => void;
  onShowDesktop?: () => void;
  activeWorkspaceId: string;
  unreadNotificationsCount?: number;
  taskbarAlignment?: 'center' | 'left';
  privacyIndicatorActive?: boolean;
}

export const Launcher: React.FC<LauncherProps> = ({
  mode,
  windows,
  onOpenApp,
  onFocusWindow,
  onCloseWindow,
  onToggleCommandPalette,
  onToggleQuickSettings,
  onToggleNotifications,
  onToggleCalendar,
  onShowDesktop,
  unreadNotificationsCount = 0,
  taskbarAlignment = 'center',
  privacyIndicatorActive = false,
}) => {
  const [hoveredAppId, setHoveredAppId] = useState<string | null>(null);
  const [volume, setVolume] = useState(80);
  const [isMuted, setIsMuted] = useState(false);
  const [showVolumeSlider, setShowVolumeSlider] = useState(false);
  const [currentTime, setCurrentTime] = useState(() => new Date());

  useEffect(() => {
    const timer = setInterval(() => setCurrentTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const isAppOpen = (appId: string) => windows.some((w) => w.appId === appId);
  const getAppWindows = (appId: string) => windows.filter((w) => w.appId === appId);

  const handleAppClick = (app: AppItem) => {
    const existing = getAppWindows(app.id);
    if (existing.length > 0) {
      // Focus or cycle
      const unfocused = existing.find((w) => !w.focused);
      if (unfocused) onFocusWindow(unfocused.id);
      else onFocusWindow(existing[0].id);
    } else {
      onOpenApp(app.id, app.name);
    }
  };

  const formattedTime = currentTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  const formattedDate = currentTime.toLocaleDateString([], { month: 'short', day: 'numeric' });

  // 1. DOCK MODE (macOS-Style Ergonomics)
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
        {/* Nova Start Pill */}
        <button
          onClick={onToggleCommandPalette}
          title="Nova Launcher (Ctrl+Space)"
          aria-label="Open Nova Launcher"
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
          <span style={{ fontSize: '18px', fontWeight: 700 }}>✦</span>
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
                    bottom: '3px',
                    width: '5px',
                    height: '5px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--nova-accent-primary)',
                  }}
                />
              )}
            </button>
          );
        })}

        <div style={{ width: '1px', height: '28px', backgroundColor: 'var(--nova-border-subtle)', margin: '0 4px' }} />

        {/* Notifications & Quick Settings */}
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

  // 2. TASKBAR MODE (Windows 11 Polish)
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
          height: '48px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 12px',
          backgroundColor: 'var(--nova-surface-glass)',
          backdropFilter: 'var(--nova-blur-glass)',
          borderTop: '1px solid var(--nova-border-subtle)',
          boxShadow: 'var(--nova-shadow-md)',
          zIndex: 'var(--nova-z-dock)' as any,
        }}
      >
        {/* Left Side: Start Button or Left-aligned Apps */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          {/* Windows 11-Style Nova Start Button */}
          <button
            onClick={onToggleCommandPalette}
            title="Start / Search (Ctrl+Space)"
            aria-label="Nova Start"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '6px',
              padding: '0 10px',
              height: '38px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: 'var(--nova-surface-raised)',
              border: '1px solid var(--nova-border-subtle)',
              color: 'var(--nova-accent-primary)',
              cursor: 'pointer',
              fontWeight: 700,
              fontSize: '13px',
              transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
          >
            <span style={{ fontSize: '16px', fontWeight: 800 }}>✦</span>
            <span>Nova</span>
          </button>

          {/* Quick Search Box Trigger */}
          <button
            onClick={onToggleCommandPalette}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '0 12px',
              height: '36px',
              borderRadius: 'var(--nova-radius-full)',
              backgroundColor: 'var(--nova-surface-base)',
              border: '1px solid var(--nova-border-subtle)',
              color: 'var(--nova-text-muted)',
              fontSize: 'var(--nova-text-xs)',
              cursor: 'pointer',
            }}
          >
            <Search size={14} color="var(--nova-accent-primary)" />
            <span>Search</span>
          </button>

          {/* If left-aligned, render apps here */}
          {taskbarAlignment === 'left' && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginLeft: '8px' }}>
              {PINNED_APPS.map((app) => {
                const open = isAppOpen(app.id);
                const appWins = getAppWindows(app.id);
                const hasFocus = appWins.some((w) => w.focused);
                return (
                  <div key={app.id} style={{ position: 'relative' }}>
                    <button
                      onClick={() => handleAppClick(app)}
                      onMouseEnter={() => setHoveredAppId(app.id)}
                      onMouseLeave={() => setHoveredAppId(null)}
                      title={app.name}
                      style={{
                        position: 'relative',
                        width: '40px',
                        height: '38px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        borderRadius: 'var(--nova-radius-md)',
                        backgroundColor: hasFocus ? 'var(--nova-surface-raised)' : 'transparent',
                        cursor: 'pointer',
                      }}
                    >
                      {app.icon}
                      {open && (
                        <span
                          style={{
                            position: 'absolute',
                            bottom: '2px',
                            width: hasFocus ? '16px' : '6px',
                            height: '3px',
                            borderRadius: '2px',
                            backgroundColor: 'var(--nova-accent-primary)',
                            transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
                          }}
                        />
                      )}
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Center: Centered Taskbar App Icons (when taskbarAlignment is 'center') */}
        {taskbarAlignment === 'center' && (
          <div
            style={{
              position: 'absolute',
              left: '50%',
              transform: 'translateX(-50%)',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
            }}
          >
            {PINNED_APPS.map((app) => {
              const open = isAppOpen(app.id);
              const appWins = getAppWindows(app.id);
              const hasFocus = appWins.some((w) => w.focused);
              const isHovered = hoveredAppId === app.id;

              return (
                <div
                  key={app.id}
                  style={{ position: 'relative' }}
                  onMouseEnter={() => setHoveredAppId(app.id)}
                  onMouseLeave={() => setHoveredAppId(null)}
                >
                  <button
                    onClick={() => handleAppClick(app)}
                    title={app.name}
                    aria-label={app.name}
                    style={{
                      position: 'relative',
                      width: '40px',
                      height: '38px',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      borderRadius: 'var(--nova-radius-md)',
                      backgroundColor: hasFocus
                        ? 'var(--nova-surface-raised)'
                        : isHovered
                        ? 'var(--nova-surface-base)'
                        : 'transparent',
                      cursor: 'pointer',
                      transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
                    }}
                  >
                    {app.icon}
                    {open && (
                      <span
                        style={{
                          position: 'absolute',
                          bottom: '2px',
                          width: hasFocus ? '16px' : '6px',
                          height: '3px',
                          borderRadius: '2px',
                          backgroundColor: 'var(--nova-accent-primary)',
                          transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
                        }}
                      />
                    )}
                  </button>

                  {/* Windows 11-Style Hover Thumbnail Preview Card */}
                  {isHovered && open && appWins.length > 0 && (
                    <div
                      style={{
                        position: 'absolute',
                        bottom: '48px',
                        left: '50%',
                        transform: 'translateX(-50%)',
                        backgroundColor: 'var(--nova-surface-glass)',
                        backdropFilter: 'var(--nova-blur-glass)',
                        border: '1px solid var(--nova-border-strong)',
                        borderRadius: 'var(--nova-radius-lg)',
                        padding: '10px',
                        boxShadow: 'var(--nova-shadow-lg)',
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '6px',
                        minWidth: '180px',
                        zIndex: 2000,
                        animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
                      }}
                      onClick={(e) => e.stopPropagation()}
                    >
                      {appWins.map((w) => (
                        <div
                          key={w.id}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            padding: '6px 8px',
                            borderRadius: 'var(--nova-radius-md)',
                            backgroundColor: w.focused ? 'var(--nova-surface-raised)' : 'var(--nova-surface-base)',
                            cursor: 'pointer',
                            fontSize: '11px',
                          }}
                          onClick={() => {
                            onFocusWindow(w.id);
                            setHoveredAppId(null);
                          }}
                        >
                          <span style={{ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '130px' }}>
                            {w.title}
                          </span>
                          {onCloseWindow && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                onCloseWindow(w.id);
                              }}
                              title="Close Window"
                              style={{ padding: '2px', borderRadius: '3px', color: 'var(--nova-text-muted)', cursor: 'pointer' }}
                              onMouseEnter={(e) => (e.currentTarget.style.color = '#f43f5e')}
                              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--nova-text-muted)')}
                            >
                              <X size={12} />
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}

        {/* Right Side: Windows 11 System Status Tray */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          {/* Privacy Indicator Dot (when active) */}
          {privacyIndicatorActive && (
            <div
              title="Camera, Microphone, or Location access active"
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: '#10b981',
                boxShadow: '0 0 8px #10b981',
              }}
            />
          )}

          {/* Quick Status Pill (WiFi, Volume, Battery) */}
          <div
            onClick={onToggleQuickSettings}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '4px 10px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: 'var(--nova-surface-raised)',
              border: '1px solid var(--nova-border-subtle)',
              cursor: 'pointer',
              color: 'var(--nova-text-secondary)',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
          >
            <span title="Connected (Simulated Wi-Fi)"><Wifi size={14} /></span>
            <div
              onClick={(e) => {
                e.stopPropagation();
                setShowVolumeSlider(!showVolumeSlider);
              }}
              style={{ display: 'flex', alignItems: 'center' }}
            >
              {isMuted ? <VolumeX size={14} color="#f43f5e" /> : <Volume2 size={14} />}
            </div>
            <span title="100% (Simulated Power Source)"><Battery size={14} /></span>
          </div>

          {/* Quick Volume Slider Popover */}
          {showVolumeSlider && (
            <div
              style={{
                position: 'absolute',
                bottom: '54px',
                right: '120px',
                backgroundColor: 'var(--nova-surface-glass)',
                backdropFilter: 'var(--nova-blur-glass)',
                border: '1px solid var(--nova-border-strong)',
                borderRadius: 'var(--nova-radius-lg)',
                padding: '12px 16px',
                boxShadow: 'var(--nova-shadow-lg)',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
                zIndex: 2000,
              }}
              onClick={(e) => e.stopPropagation()}
            >
              <button
                onClick={() => setIsMuted(!isMuted)}
                style={{ cursor: 'pointer', color: isMuted ? '#f43f5e' : 'var(--nova-text-primary)' }}
              >
                {isMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
              </button>
              <input
                type="range"
                min="0"
                max="100"
                value={isMuted ? 0 : volume}
                onChange={(e) => {
                  setVolume(Number(e.target.value));
                  if (isMuted) setIsMuted(false);
                }}
                style={{ width: '120px', accentColor: 'var(--nova-accent-primary)', cursor: 'pointer' }}
              />
              <span style={{ fontSize: '11px', width: '28px', textAlign: 'right', color: 'var(--nova-text-muted)' }}>
                {isMuted ? '0%' : `${volume}%`}
              </span>
            </div>
          )}

          {/* Interactive Clock & Calendar Toggle */}
          <div
            onClick={onToggleCalendar}
            title="Click to view Calendar & Schedule"
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'flex-end',
              justifyContent: 'center',
              padding: '2px 8px',
              borderRadius: 'var(--nova-radius-md)',
              cursor: 'pointer',
              lineHeight: 1.2,
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
              {formattedTime}
            </span>
            <span style={{ fontSize: '10px', color: 'var(--nova-text-muted)' }}>
              {formattedDate}
            </span>
          </div>

          {/* Notification Center Trigger */}
          <button
            onClick={onToggleNotifications}
            title="Notification Center"
            aria-label="Notification Center"
            style={{
              position: 'relative',
              padding: '6px',
              borderRadius: 'var(--nova-radius-md)',
              color: 'var(--nova-text-secondary)',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <Bell size={16} />
            {unreadNotificationsCount > 0 && (
              <span
                style={{
                  position: 'absolute',
                  top: '2px',
                  right: '2px',
                  width: '7px',
                  height: '7px',
                  borderRadius: '50%',
                  backgroundColor: '#f43f5e',
                }}
              />
            )}
          </button>

          {/* Windows 11 Show Desktop Peek Button at Edge */}
          {onShowDesktop && (
            <div
              onClick={onShowDesktop}
              title="Show Desktop (Minimize all windows)"
              style={{
                width: '6px',
                height: '32px',
                borderLeft: '1px solid var(--nova-border-subtle)',
                cursor: 'pointer',
                marginLeft: '4px',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-accent-primary)')}
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
            />
          )}
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
        height: '36px',
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

      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span style={{ fontSize: '12px', color: 'var(--nova-text-secondary)' }}>
          {formattedTime}
        </span>
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
