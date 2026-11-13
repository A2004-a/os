import React, { useState } from 'react';
import { AppManifest } from '@shared/types';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

export const CATALOG_APPS: AppManifest[] = [
  {
    id: 'io.novaos.files',
    name: 'Files',
    version: '0.1.0',
    description: 'Virtual Nova Home and permitted host folder explorer with safe trash management.',
    icon: '📁',
    category: 'productivity',
    defaultBounds: { width: 880, height: 580 },
    requiredPermissions: ['host-filesystem:read', 'host-filesystem:write'],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.notes',
    name: 'Notes',
    version: '0.1.0',
    description: 'Local markdown and plain-text note editor with SQLite autosave and export.',
    icon: '📝',
    category: 'productivity',
    defaultBounds: { width: 780, height: 520 },
    requiredPermissions: [],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.terminal',
    name: 'Terminal',
    version: '0.1.0',
    description: 'Secure, host-aware shell terminal with working directory confinement.',
    icon: '💻',
    category: 'development',
    defaultBounds: { width: 740, height: 480 },
    requiredPermissions: ['terminal:execute'],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.sysmon',
    name: 'System Monitor',
    version: '0.1.0',
    description: 'Hardware telemetry displaying real-time CPU, RAM, and platform metrics.',
    icon: '📊',
    category: 'system',
    defaultBounds: { width: 720, height: 500 },
    requiredPermissions: ['system:metrics'],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.calc',
    name: 'Calculator',
    version: '0.1.0',
    description: 'Accessible standard and scientific mathematics engine.',
    icon: '🔢',
    category: 'utilities',
    defaultBounds: { width: 440, height: 520 },
    requiredPermissions: [],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.clock',
    name: 'Clock',
    version: '0.1.0',
    description: 'World clocks, countdown timers, stopwatch, and alarms.',
    icon: '⏱️',
    category: 'utilities',
    defaultBounds: { width: 560, height: 460 },
    requiredPermissions: ['notifications:send'],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.media',
    name: 'Media Viewer',
    version: '0.1.0',
    description: 'Safe image and graphics viewer with zoom and rotation tools.',
    icon: '🖼️',
    category: 'media',
    defaultBounds: { width: 680, height: 480 },
    requiredPermissions: [],
    isSystemApp: true,
  },
  {
    id: 'io.novaos.devconsole',
    name: 'Developer Console',
    version: '0.1.0',
    description: 'Inspect live IPC channels, permission grants, and platform diagnostics.',
    icon: '🛠️',
    category: 'development',
    defaultBounds: { width: 840, height: 540 },
    requiredPermissions: ['system:metrics'],
    isSystemApp: true,
  },
];

interface SoftwareCenterAppProps {
  onOpenApp: (appId: string, name: string) => void;
}

export const SoftwareCenterApp: React.FC<SoftwareCenterAppProps> = ({ onOpenApp }) => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  const filtered = selectedCategory === 'all'
    ? CATALOG_APPS
    : CATALOG_APPS.filter((a) => a.category === selectedCategory);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Disclaimer Banner */}
      <div style={{ padding: '8px 16px', backgroundColor: 'var(--nova-surface-base)', borderBottom: '1px solid var(--nova-border-subtle)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)' }}>
          Nova Software Center manages curated, sandboxed NovaOS application manifests. It is not a replacement for native package managers (apt, dnf, Homebrew, Winget).
        </span>
        <Badge tier="simulated">Curated Catalog</Badge>
      </div>

      {/* Category Filter Bar */}
      <div style={{ padding: '12px 16px', display: 'flex', gap: '8px', borderBottom: '1px solid var(--nova-border-subtle)' }}>
        {['all', 'productivity', 'development', 'system', 'utilities', 'media'].map((cat) => (
          <button
            key={cat}
            onClick={() => setSelectedCategory(cat)}
            style={{
              padding: '6px 12px',
              borderRadius: 'var(--nova-radius-md)',
              backgroundColor: selectedCategory === cat ? 'var(--nova-accent-primary)' : 'var(--nova-surface-base)',
              color: selectedCategory === cat ? 'var(--nova-text-inverse)' : 'var(--nova-text-secondary)',
              fontSize: 'var(--nova-text-xs)',
              fontWeight: 600,
              textTransform: 'capitalize',
              cursor: 'pointer',
            }}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* App Grid */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: '16px', alignContent: 'flex-start' }}>
        {filtered.map((app) => {
          return (
            <div
              key={app.id}
              style={{
                padding: '16px',
                borderRadius: 'var(--nova-radius-lg)',
                backgroundColor: 'var(--nova-surface-base)',
                border: '1px solid var(--nova-border-subtle)',
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'space-between',
                gap: '12px',
              }}
            >
              <div>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontSize: '24px' }}>{app.icon}</span>
                    <div>
                      <h4 style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600 }}>{app.name}</h4>
                      <span style={{ fontSize: '11px', color: 'var(--nova-text-muted)' }}>v{app.version} • {app.category}</span>
                    </div>
                  </div>
                  {app.isSystemApp && <Badge tier="neutral">Core App</Badge>}
                </div>
                <p style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-secondary)', lineHeight: 1.4 }}>
                  {app.description}
                </p>
              </div>

              {/* Permissions & Actions */}
              <div style={{ borderTop: '1px solid var(--nova-border-subtle)', paddingTop: '10px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                  {app.requiredPermissions.length === 0 ? (
                    <span style={{ fontSize: '10px', color: 'var(--nova-text-muted)' }}>No host permissions required</span>
                  ) : (
                    app.requiredPermissions.map((p) => (
                      <span key={p} style={{ fontSize: '10px', padding: '2px 6px', borderRadius: '4px', backgroundColor: 'var(--nova-surface-raised)', color: '#f59e0b' }}>
                        {p}
                      </span>
                    ))
                  )}
                </div>

                <Button size="sm" variant="primary" onClick={() => onOpenApp(app.id, app.name)}>
                  Launch
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
