import React, { useState, useEffect } from 'react';
import {
  Palette,
  Layers,
  ShieldCheck,
  HardDrive,
  Keyboard,
  Info,
  Moon,
  Sun,
  Eye,
  Trash2,
} from 'lucide-react';
import { UserPreferences, PermissionGrant, ThemeMode, AccentColor, LauncherMode } from '@shared/types';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

interface SettingsAppProps {
  preferences: UserPreferences;
  onSetTheme: (theme: ThemeMode) => void;
  onSetAccent: (accent: AccentColor) => void;
  onSetLauncherMode: (mode: LauncherMode) => void;
  onToggleReducedMotion: () => void;
  onToggleReducedTransparency: () => void;
}

type SettingsSection = 'appearance' | 'workspaces' | 'permissions' | 'storage' | 'shortcuts' | 'about';

export const SettingsApp: React.FC<SettingsAppProps> = ({
  preferences,
  onSetTheme,
  onSetAccent,
  onSetLauncherMode,
  onToggleReducedMotion,
  onToggleReducedTransparency,
}) => {
  const [activeSection, setActiveSection] = useState<SettingsSection>('appearance');
  const [grants, setGrants] = useState<PermissionGrant[]>([]);

  const loadGrants = async () => {
    if (window.nova?.permissions?.getGrants) {
      const res = await window.nova.permissions.getGrants();
      if (res.success && res.grants) {
        setGrants(res.grants);
      }
    }
  };

  useEffect(() => {
    loadGrants();
  }, []);

  const handleRevoke = async (grantId: string) => {
    if (window.nova?.permissions?.revoke) {
      await window.nova.permissions.revoke(grantId);
      loadGrants();
    }
  };

  const navItems: { id: SettingsSection; label: string; icon: React.ReactNode }[] = [
    { id: 'appearance', label: 'Appearance', icon: <Palette size={16} /> },
    { id: 'workspaces', label: 'Workspaces', icon: <Layers size={16} /> },
    { id: 'permissions', label: 'Permissions & Security', icon: <ShieldCheck size={16} /> },
    { id: 'storage', label: 'Storage & Virtual FS', icon: <HardDrive size={16} /> },
    { id: 'shortcuts', label: 'Keyboard Shortcuts', icon: <Keyboard size={16} /> },
    { id: 'about', label: 'About NovaOS', icon: <Info size={16} /> },
  ];

  const accents: AccentColor[] = ['cyan', 'blue', 'purple', 'amber', 'emerald', 'rose'];

  return (
    <div style={{ display: 'flex', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Settings Navigation Sidebar */}
      <div
        style={{
          width: '210px',
          backgroundColor: 'var(--nova-surface-base)',
          borderRight: '1px solid var(--nova-border-subtle)',
          padding: '12px 8px',
          display: 'flex',
          flexDirection: 'column',
          gap: '4px',
        }}
      >
        <div style={{ padding: '8px 12px', fontSize: 'var(--nova-text-sm)', fontWeight: 600, color: 'var(--nova-text-primary)', marginBottom: '8px' }}>
          Settings
        </div>
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setActiveSection(item.id)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '8px 12px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: activeSection === item.id ? 'var(--nova-surface-raised)' : 'transparent',
              color: activeSection === item.id ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
              fontSize: 'var(--nova-text-xs)',
              fontWeight: 500,
              cursor: 'pointer',
              textAlign: 'left',
              transition: 'background-color var(--nova-duration-fast) var(--nova-ease-standard)',
            }}
          >
            {item.icon}
            {item.label}
          </button>
        ))}
      </div>

      {/* Main Settings Content */}
      <div style={{ flex: 1, padding: '28px 36px', overflowY: 'auto' }}>
        {/* APPEARANCE */}
        {activeSection === 'appearance' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '600px' }}>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                <h2 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 600 }}>Appearance & Themes</h2>
                <Badge tier="simulated">Simulated</Badge>
              </div>
              <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
                Customize the Nova Surface design system inside the desktop environment.
              </p>
            </div>

            {/* Theme Selector */}
            <div>
              <label style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, display: 'block', marginBottom: '8px' }}>
                SURFACE THEME
              </label>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
                <button
                  onClick={() => onSetTheme('dark')}
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: preferences.theme === 'dark' ? 'var(--nova-accent-glow)' : 'var(--nova-surface-base)',
                    border: `1px solid ${preferences.theme === 'dark' ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                    cursor: 'pointer',
                  }}
                >
                  <Moon size={18} color="var(--nova-accent-primary)" />
                  <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 500 }}>Dark</span>
                </button>
                <button
                  onClick={() => onSetTheme('light')}
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: preferences.theme === 'light' ? 'var(--nova-accent-glow)' : 'var(--nova-surface-base)',
                    border: `1px solid ${preferences.theme === 'light' ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                    cursor: 'pointer',
                  }}
                >
                  <Sun size={18} color="var(--nova-accent-primary)" />
                  <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 500 }}>Light</span>
                </button>
                <button
                  onClick={() => onSetTheme('high-contrast')}
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '12px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: preferences.theme === 'high-contrast' ? 'var(--nova-accent-glow)' : 'var(--nova-surface-base)',
                    border: `1px solid ${preferences.theme === 'high-contrast' ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                    cursor: 'pointer',
                  }}
                >
                  <Eye size={18} color="var(--nova-accent-primary)" />
                  <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 500 }}>High Contrast</span>
                </button>
              </div>
            </div>

            {/* Accent Color Palette */}
            <div>
              <label style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, display: 'block', marginBottom: '8px' }}>
                ACCENT PALETTE
              </label>
              <div style={{ display: 'flex', gap: '12px' }}>
                {accents.map((acc) => (
                  <button
                    key={acc}
                    onClick={() => onSetAccent(acc)}
                    style={{
                      width: '36px',
                      height: '36px',
                      borderRadius: '50%',
                      border: preferences.accent === acc ? '3px solid #ffffff' : '2px solid transparent',
                      cursor: 'pointer',
                      backgroundColor:
                        acc === 'cyan' ? '#06b6d4' :
                        acc === 'blue' ? '#3b82f6' :
                        acc === 'purple' ? '#a855f7' :
                        acc === 'amber' ? '#f59e0b' :
                        acc === 'emerald' ? '#10b981' : '#f43f5e',
                    }}
                  />
                ))}
              </div>
            </div>

            {/* Launcher Layout Mode */}
            <div>
              <label style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, display: 'block', marginBottom: '8px' }}>
                DESKTOP SHELL LAYOUT
              </label>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
                {(['dock', 'taskbar', 'panel'] as LauncherMode[]).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => onSetLauncherMode(mode)}
                    style={{
                      padding: '12px',
                      borderRadius: 'var(--nova-radius-md)',
                      backgroundColor: preferences.launcherMode === mode ? 'var(--nova-surface-raised)' : 'var(--nova-surface-base)',
                      border: `1px solid ${preferences.launcherMode === mode ? 'var(--nova-accent-primary)' : 'var(--nova-border-subtle)'}`,
                      color: preferences.launcherMode === mode ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
                      fontSize: 'var(--nova-text-xs)',
                      fontWeight: 600,
                      cursor: 'pointer',
                    }}
                  >
                    {mode === 'dock' ? 'Floating Dock (macOS)' : mode === 'taskbar' ? 'Classic Taskbar (Windows)' : 'Top Panel (Linux)'}
                  </button>
                ))}
              </div>
            </div>

            {/* Accessibility Toggles */}
            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)', display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 500 }}>Reduced Motion</div>
                  <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>Disables animations and transitions for accessibility</div>
                </div>
                <Button size="sm" variant={preferences.motionReduced ? 'primary' : 'secondary'} onClick={onToggleReducedMotion}>
                  {preferences.motionReduced ? 'Enabled' : 'Disabled'}
                </Button>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 500 }}>Reduced Transparency</div>
                  <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>Uses solid surfaces instead of blurred glass effects</div>
                </div>
                <Button size="sm" variant={preferences.transparencyReduced ? 'primary' : 'secondary'} onClick={onToggleReducedTransparency}>
                  {preferences.transparencyReduced ? 'Enabled' : 'Disabled'}
                </Button>
              </div>
            </div>
          </div>
        )}

        {/* WORKSPACES */}
        {activeSection === 'workspaces' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '600px' }}>
            <div>
              <h2 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 600, marginBottom: '4px' }}>Workspaces & Multitasking</h2>
              <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
                Configure virtual desktops and window snapping sensitivity.
              </p>
            </div>
            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <div style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 500, marginBottom: '8px' }}>Active Workspaces</div>
              <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)', lineHeight: 1.5 }}>
                NovaOS isolates window state per workspace. You can switch between Workspace 1, Workspace 2, and Workspace 3 using the desktop indicator pill or the Global Command Palette (Ctrl+Space).
              </p>
            </div>
          </div>
        )}

        {/* PERMISSIONS & SECURITY */}
        {activeSection === 'permissions' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '600px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div>
                <h2 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 600, marginBottom: '4px' }}>Permission Broker</h2>
                <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
                  Active capability grants and access control policies.
                </p>
              </div>
              <Badge tier="managed">Managed Security</Badge>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {grants.length === 0 ? (
                <div style={{ padding: '24px', textAlign: 'center', backgroundColor: 'var(--nova-surface-base)', borderRadius: 'var(--nova-radius-md)', color: 'var(--nova-text-muted)', fontSize: 'var(--nova-text-xs)' }}>
                  No active capability grants
                </div>
              ) : (
                grants.map((grant) => (
                  <div
                    key={grant.id}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '12px 16px',
                      borderRadius: 'var(--nova-radius-md)',
                      backgroundColor: 'var(--nova-surface-base)',
                      border: '1px solid var(--nova-border-subtle)',
                    }}
                  >
                    <div>
                      <div style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600, color: 'var(--nova-text-primary)' }}>
                        {grant.capability}
                      </div>
                      <div style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>
                        App: {grant.appId} {grant.resourcePath ? `• Path: ${grant.resourcePath}` : ''}
                      </div>
                    </div>
                    <Button size="sm" variant="danger" icon={<Trash2 size={12} />} onClick={() => handleRevoke(grant.id)}>
                      Revoke
                    </Button>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {/* KEYBOARD SHORTCUTS */}
        {activeSection === 'shortcuts' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '600px' }}>
            <div>
              <h2 style={{ fontSize: 'var(--nova-text-lg)', fontWeight: 600, marginBottom: '4px' }}>Keyboard Shortcuts</h2>
              <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
                Keyboard-first productivity shortcuts across NovaOS.
              </p>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {[
                { key: 'Ctrl + Space / Cmd + Space', action: 'Open Global Command Palette' },
                { key: 'Alt + Tab / Option + Tab', action: 'Switch Active Applications' },
                { key: 'Alt + Left Arrow', action: 'Snap Window to Left Half' },
                { key: 'Alt + Right Arrow', action: 'Snap Window to Right Half' },
                { key: 'Alt + Up Arrow', action: 'Maximize Window' },
                { key: 'Escape', action: 'Close Dialogs / Dismiss Palette' },
              ].map((sc) => (
                <div
                  key={sc.key}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '10px 16px',
                    borderRadius: 'var(--nova-radius-md)',
                    backgroundColor: 'var(--nova-surface-base)',
                    border: '1px solid var(--nova-border-subtle)',
                  }}
                >
                  <span style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-primary)' }}>{sc.action}</span>
                  <kbd style={{ padding: '4px 8px', borderRadius: '4px', backgroundColor: 'var(--nova-surface-raised)', border: '1px solid var(--nova-border-strong)', fontSize: '11px', fontFamily: 'var(--nova-font-mono)' }}>
                    {sc.key}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ABOUT NOVAOS */}
        {activeSection === 'about' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '18px', maxWidth: '600px' }}>
            <div>
              <h2 style={{ fontSize: 'var(--nova-text-xl)', fontWeight: 700, color: 'var(--nova-accent-primary)' }}>
                NovaOS
              </h2>
              <div style={{ fontSize: 'var(--nova-text-sm)', color: 'var(--nova-text-secondary)' }}>
                One Desktop. Three Worlds. One Experience.
              </div>
            </div>

            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)', lineHeight: 1.6, fontSize: 'var(--nova-text-xs)' }}>
              <p style={{ marginBottom: '8px' }}>
                <strong>Product Version:</strong> v0.1.0 (Production Candidate)
              </p>
              <p style={{ marginBottom: '8px' }}>
                <strong>Architecture:</strong> Electron + Chromium Sandbox + Node.js Main Process + SQLite
              </p>
              <p>
                <strong>Security Boundary:</strong> NovaOS is a desktop environment application running inside an operating system. It provides a cohesive desktop experience without pretending to be a host bootable kernel or driver stack.
              </p>
            </div>

            <div>
              <h3 style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, marginBottom: '8px' }}>Capability Tier Legend</h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: 'var(--nova-text-xs)' }}>
                  <Badge tier="native">Native Integration</Badge>
                  <span style={{ color: 'var(--nova-text-secondary)' }}>Real host-supported action (pickers, notifications, system metrics, terminal)</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: 'var(--nova-text-xs)' }}>
                  <Badge tier="managed">Managed Integration</Badge>
                  <span style={{ color: 'var(--nova-text-secondary)' }}>Audited operation governed by permission broker and path confinement</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: 'var(--nova-text-xs)' }}>
                  <Badge tier="simulated">Simulated Environment</Badge>
                  <span style={{ color: 'var(--nova-text-secondary)' }}>Desktop canvas, window manager, virtual files (Nova Home), themes</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: 'var(--nova-text-xs)' }}>
                  <Badge tier="unavailable">Unavailable</Badge>
                  <span style={{ color: 'var(--nova-text-secondary)' }}>Host kernel, global OS window manager interception, hardware drivers</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
