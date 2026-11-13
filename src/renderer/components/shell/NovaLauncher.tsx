import React, { useState, useEffect, useRef, useMemo } from 'react';
import {
  Search,
  Folder,
  FileText,
  Terminal,
  Settings,
  Activity,
  Calculator,
  Clock,
  Package,
  Wrench,
  SunMoon,
  Layers,
  Power,
} from 'lucide-react';
import { Badge } from '../common/Badge';

export interface LauncherSearchResult {
  id: string;
  title: string;
  subtitle?: string;
  category: 'Applications' | 'Continue Working' | 'Files' | 'Settings' | 'Commands' | 'Workspaces';
  icon: React.ReactNode;
  tier: 'native' | 'managed' | 'simulated' | 'neutral';
  action: () => void;
}

interface NovaLauncherProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenApp: (appId: string, name: string) => void;
  onSwitchWorkspace: (workspaceId: string) => void;
  onToggleTheme: () => void;
  onOpenNote?: () => void;
  onOpenTerminal?: () => void;
}

export const NovaLauncher: React.FC<NovaLauncherProps> = ({
  isOpen,
  onClose,
  onOpenApp,
  onSwitchWorkspace,
  onToggleTheme,
  onOpenNote,
  onOpenTerminal,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 60);
    }
  }, [isOpen]);

  const pinnedApps = [
    { id: 'io.novaos.files', name: 'Files', desc: 'Virtual & Permitted Host', icon: <Folder size={22} color="#06b6d4" /> },
    { id: 'io.novaos.notes', name: 'Notes', desc: 'Markdown & Local Notes', icon: <FileText size={22} color="#3b82f6" /> },
    { id: 'io.novaos.terminal', name: 'Terminal', desc: 'Sandboxed Host Shell', icon: <Terminal size={22} color="#10b981" /> },
    { id: 'io.novaos.calc', name: 'Calculator', desc: 'Math & Programmer', icon: <Calculator size={22} color="#f59e0b" /> },
    { id: 'io.novaos.clock', name: 'Clock', desc: 'Timers & Focus Sessions', icon: <Clock size={22} color="#a855f7" /> },
    { id: 'io.novaos.sysmon', name: 'System Monitor', desc: 'Host Hardware & Metrics', icon: <Activity size={22} color="#f43f5e" /> },
    { id: 'io.novaos.software', name: 'Software Center', desc: 'App Catalog & Store', icon: <Package size={22} color="#38bdf8" /> },
    { id: 'io.novaos.settings', name: 'Settings', desc: 'System & Personalization', icon: <Settings size={22} color="#94a3b8" /> },
  ];

  const recentDocuments = [
    { title: 'Welcome to NovaOS.md', path: 'nova://documents/Welcome to NovaOS.md', time: '10m ago' },
    { title: 'Architecture Audit & Security.md', path: 'nova://documents/Audit.md', time: '1h ago' },
    { title: 'Project Roadmap 2026.txt', path: 'nova://projects/Roadmap.txt', time: 'Yesterday' },
  ];

  const allSearchItems: LauncherSearchResult[] = useMemo(() => [
    {
      id: 'app-files',
      title: 'Files',
      subtitle: 'Nova Home virtual storage and approved host folders',
      category: 'Applications',
      icon: <Folder size={18} color="#06b6d4" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.files', 'Files'); onClose(); },
    },
    {
      id: 'app-notes',
      title: 'Notes',
      subtitle: 'Local-first Markdown notes with automatic saving',
      category: 'Applications',
      icon: <FileText size={18} color="#3b82f6" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.notes', 'Notes'); onClose(); },
    },
    {
      id: 'app-terminal',
      title: 'Terminal',
      subtitle: 'Native platform shell (PowerShell / zsh) with audit logging',
      category: 'Applications',
      icon: <Terminal size={18} color="#10b981" />,
      tier: 'native',
      action: () => { onOpenApp('io.novaos.terminal', 'Terminal'); onClose(); },
    },
    {
      id: 'app-calc',
      title: 'Calculator',
      subtitle: 'Standard, scientific, and keyboard calculations',
      category: 'Applications',
      icon: <Calculator size={18} color="#f59e0b" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.calc', 'Calculator'); onClose(); },
    },
    {
      id: 'app-clock',
      title: 'Clock & Focus',
      subtitle: 'World clock, stopwatch, timer, and Pomodoro focus sessions',
      category: 'Applications',
      icon: <Clock size={18} color="#a855f7" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.clock', 'Clock & Focus'); onClose(); },
    },
    {
      id: 'app-sysmon',
      title: 'System Monitor',
      subtitle: 'Real-time CPU, RAM, OS metrics and active processes',
      category: 'Applications',
      icon: <Activity size={18} color="#f43f5e" />,
      tier: 'native',
      action: () => { onOpenApp('io.novaos.sysmon', 'System Monitor'); onClose(); },
    },
    {
      id: 'app-software',
      title: 'Software Center',
      subtitle: 'Explore and launch sandboxed ecosystem applications',
      category: 'Applications',
      icon: <Package size={18} color="#38bdf8" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.software', 'Software Center'); onClose(); },
    },
    {
      id: 'app-settings',
      title: 'Settings',
      subtitle: 'Themes, accents, capability permissions, and storage audit',
      category: 'Applications',
      icon: <Settings size={18} color="#94a3b8" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.settings', 'Settings'); onClose(); },
    },
    {
      id: 'app-devconsole',
      title: 'Developer Console',
      subtitle: 'Inspect secure IPC events, capability grants, and audit log',
      category: 'Applications',
      icon: <Wrench size={18} color="#f97316" />,
      tier: 'managed',
      action: () => { onOpenApp('io.novaos.devconsole', 'Developer Console'); onClose(); },
    },
    {
      id: 'cmd-theme',
      title: 'Toggle Theme (Dark / Light)',
      subtitle: 'Switch between Nova Surface dark, light, and high-contrast modes',
      category: 'Commands',
      icon: <SunMoon size={18} color="#eab308" />,
      tier: 'neutral',
      action: () => { onToggleTheme(); onClose(); },
    },
    {
      id: 'file-welcome',
      title: 'Welcome to NovaOS.md',
      subtitle: 'nova://documents/Welcome to NovaOS.md',
      category: 'Files',
      icon: <FileText size={18} color="#3b82f6" />,
      tier: 'simulated',
      action: () => { onOpenApp('io.novaos.notes', 'Notes'); onClose(); },
    },
    {
      id: 'ws-1',
      title: 'Switch to Workspace 1',
      subtitle: 'Main productivity space',
      category: 'Workspaces',
      icon: <Layers size={18} color="#06b6d4" />,
      tier: 'neutral',
      action: () => { onSwitchWorkspace('ws-1'); onClose(); },
    },
    {
      id: 'ws-2',
      title: 'Switch to Workspace 2',
      subtitle: 'Developer tools & Terminal',
      category: 'Workspaces',
      icon: <Layers size={18} color="#10b981" />,
      tier: 'neutral',
      action: () => { onSwitchWorkspace('ws-2'); onClose(); },
    },
  ], [onOpenApp, onSwitchWorkspace, onToggleTheme, onClose]);

  const filteredResults = useMemo(() => {
    if (!query.trim()) return [];
    const q = query.toLowerCase();
    return allSearchItems.filter(
      (item) => item.title.toLowerCase().includes(q) || (item.subtitle && item.subtitle.toLowerCase().includes(q))
    );
  }, [query, allSearchItems]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (filteredResults.length > 0) {
        setSelectedIndex((prev) => (prev + 1) % filteredResults.length);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (filteredResults.length > 0) {
        setSelectedIndex((prev) => (prev - 1 + filteredResults.length) % filteredResults.length);
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filteredResults.length > 0 && filteredResults[selectedIndex]) {
        filteredResults[selectedIndex].action();
      }
    }
  };

  if (!isOpen) return null;

  const isSearchActive = query.trim().length > 0;

  return (
    <div
      role="presentation"
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.4)',
        backdropFilter: 'blur(4px)',
        display: 'flex',
        alignItems: 'flex-end',
        justifyContent: 'center',
        paddingBottom: '56px',
        zIndex: 'var(--nova-z-modal)' as any,
      }}
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Nova Launcher and Universal Search"
        style={{
          width: '580px',
          maxHeight: '620px',
          backgroundColor: 'var(--nova-surface-glass)',
          backdropFilter: 'var(--nova-blur-glass)',
          border: '1px solid var(--nova-border-strong)',
          borderRadius: 'var(--nova-radius-xl)',
          boxShadow: 'var(--nova-shadow-lg)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          animation: 'fadeIn var(--nova-duration-fast) var(--nova-ease-standard)',
        }}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Top Search Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
            padding: '16px 20px',
            borderBottom: '1px solid var(--nova-border-subtle)',
            backgroundColor: 'var(--nova-surface-raised)',
          }}
        >
          <Search size={20} color="var(--nova-accent-primary)" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            placeholder="Search apps, files, settings, and commands..."
            aria-label="Search"
            style={{
              flex: 1,
              fontSize: 'var(--nova-text-base)',
              color: 'var(--nova-text-primary)',
              backgroundColor: 'transparent',
            }}
          />
          {isSearchActive && (
            <button
              onClick={() => setQuery('')}
              style={{
                fontSize: '11px',
                color: 'var(--nova-text-muted)',
                padding: '2px 6px',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Clear
            </button>
          )}
        </div>

        {/* Dynamic Content: Search Results OR Start Menu Overview */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px 20px' }}>
          {isSearchActive ? (
            /* SEARCH RESULTS VIEW */
            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                <span style={{ fontSize: '11px', fontWeight: 600, color: 'var(--nova-text-muted)', textTransform: 'uppercase' }}>
                  Search Results ({filteredResults.length})
                </span>
                <span style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>
                  Local-first • Zero remote tracking
                </span>
              </div>

              {filteredResults.length === 0 ? (
                <div style={{ padding: '32px 16px', textAlign: 'center', color: 'var(--nova-text-muted)' }}>
                  No matching apps, files, or commands found for "{query}".
                </div>
              ) : (
                filteredResults.map((item, idx) => {
                  const isSel = idx === selectedIndex;
                  return (
                    <div
                      key={item.id}
                      onClick={item.action}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '10px 14px',
                        borderRadius: 'var(--nova-radius-md)',
                        backgroundColor: isSel ? 'var(--nova-surface-raised)' : 'transparent',
                        border: `1px solid ${isSel ? 'var(--nova-border-focus)' : 'transparent'}`,
                        cursor: 'pointer',
                        transition: 'all var(--nova-duration-fast) var(--nova-ease-standard)',
                      }}
                      onMouseEnter={() => setSelectedIndex(idx)}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                        <div
                          style={{
                            width: '32px',
                            height: '32px',
                            borderRadius: 'var(--nova-radius-md)',
                            backgroundColor: 'var(--nova-surface-base)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                          }}
                        >
                          {item.icon}
                        </div>
                        <div>
                          <div style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
                            {item.title}
                          </div>
                          {item.subtitle && (
                            <div style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)' }}>
                              {item.subtitle}
                            </div>
                          )}
                        </div>
                      </div>
                      <Badge tier={item.tier}>{item.tier}</Badge>
                    </div>
                  );
                })
              )}
            </div>
          ) : (
            /* START MENU OVERVIEW VIEW */
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              {/* Quick Actions Row */}
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => { onOpenNote ? onOpenNote() : onOpenApp('io.novaos.notes', 'Notes'); onClose(); }}
                  style={{
                    flex: 1,
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    padding: '8px 12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: 'var(--nova-surface-raised)',
                    border: '1px solid var(--nova-border-subtle)',
                    fontSize: '12px',
                    fontWeight: 500,
                    cursor: 'pointer',
                    color: 'var(--nova-text-primary)',
                  }}
                >
                  <FileText size={15} color="var(--nova-accent-primary)" />
                  New Note
                </button>
                <button
                  onClick={() => { onOpenTerminal ? onOpenTerminal() : onOpenApp('io.novaos.terminal', 'Terminal'); onClose(); }}
                  style={{
                    flex: 1,
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    padding: '8px 12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: 'var(--nova-surface-raised)',
                    border: '1px solid var(--nova-border-subtle)',
                    fontSize: '12px',
                    fontWeight: 500,
                    cursor: 'pointer',
                    color: 'var(--nova-text-primary)',
                  }}
                >
                  <Terminal size={15} color="#10b981" />
                  Terminal
                </button>
                <button
                  onClick={() => { onOpenApp('io.novaos.calc', 'Calculator'); onClose(); }}
                  style={{
                    flex: 1,
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    padding: '8px 12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: 'var(--nova-surface-raised)',
                    border: '1px solid var(--nova-border-subtle)',
                    fontSize: '12px',
                    fontWeight: 500,
                    cursor: 'pointer',
                    color: 'var(--nova-text-primary)',
                  }}
                >
                  <Calculator size={15} color="#f59e0b" />
                  Calculator
                </button>
              </div>

              {/* Pinned Applications Grid */}
              <div>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px' }}>
                  <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--nova-text-muted)' }}>
                    Pinned Apps
                  </span>
                  <button
                    onClick={() => onOpenApp('io.novaos.software', 'Software Center')}
                    style={{ fontSize: '11px', color: 'var(--nova-accent-primary)', cursor: 'pointer' }}
                  >
                    All Apps &gt;
                  </button>
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '10px' }}>
                  {pinnedApps.map((app) => (
                    <button
                      key={app.id}
                      onClick={() => { onOpenApp(app.id, app.name); onClose(); }}
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        gap: '6px',
                        padding: '12px 8px',
                        borderRadius: 'var(--nova-radius-lg)',
                        backgroundColor: 'var(--nova-surface-base)',
                        border: '1px solid var(--nova-border-subtle)',
                        cursor: 'pointer',
                        transition: 'transform var(--nova-duration-fast) var(--nova-ease-standard)',
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)';
                        e.currentTarget.style.transform = 'translateY(-2px)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)';
                        e.currentTarget.style.transform = 'translateY(0)';
                      }}
                    >
                      {app.icon}
                      <span style={{ fontSize: '12px', fontWeight: 500, color: 'var(--nova-text-primary)' }}>
                        {app.name}
                      </span>
                    </button>
                  ))}
                </div>
              </div>

              {/* Continue Working / Recommended Documents */}
              <div>
                <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--nova-text-muted)', display: 'block', marginBottom: '10px' }}>
                  Continue Working
                </span>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {recentDocuments.map((doc, i) => (
                    <div
                      key={i}
                      onClick={() => { onOpenApp('io.novaos.notes', 'Notes'); onClose(); }}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '8px 12px',
                        borderRadius: 'var(--nova-radius-md)',
                        backgroundColor: 'var(--nova-surface-base)',
                        border: '1px solid var(--nova-border-subtle)',
                        cursor: 'pointer',
                      }}
                      onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-raised)')}
                      onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <FileText size={16} color="var(--nova-accent-primary)" />
                        <div>
                          <div style={{ fontSize: '12px', fontWeight: 500, color: 'var(--nova-text-primary)' }}>{doc.title}</div>
                          <div style={{ fontSize: '10px', color: 'var(--nova-text-muted)' }}>{doc.path}</div>
                        </div>
                      </div>
                      <span style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>{doc.time}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Bottom Footer (User & System Controls) */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '12px 20px',
            borderTop: '1px solid var(--nova-border-subtle)',
            backgroundColor: 'var(--nova-surface-raised)',
          }}
        >
          {/* User Profile */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '50%',
                backgroundColor: 'var(--nova-accent-primary)',
                color: 'var(--nova-text-inverse)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontWeight: 700,
                fontSize: '13px',
              }}
            >
              N
            </div>
            <div>
              <div style={{ fontSize: '12px', fontWeight: 600, color: 'var(--nova-text-primary)' }}>NovaOS User</div>
              <div style={{ fontSize: '10px', color: 'var(--nova-text-muted)' }}>Local Session • Sandbox Safe</div>
            </div>
          </div>

          {/* Quick Settings & Power Options */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <button
              onClick={() => { onOpenApp('io.novaos.settings', 'Settings'); onClose(); }}
              title="Open Settings"
              style={{
                padding: '6px',
                borderRadius: 'var(--nova-radius-sm)',
                cursor: 'pointer',
                color: 'var(--nova-text-secondary)',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
            >
              <Settings size={16} />
            </button>
            <button
              onClick={() => {
                alert('NovaOS Simulated Power: In a real desktop environment, this simulates power management.');
              }}
              title="Power & Session Options"
              style={{
                padding: '6px',
                borderRadius: 'var(--nova-radius-sm)',
                cursor: 'pointer',
                color: 'var(--nova-text-secondary)',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--nova-surface-base)')}
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
            >
              <Power size={16} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
