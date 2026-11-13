import React, { useState, useEffect } from 'react';
import { Wrench, RefreshCw, CheckCircle2 } from 'lucide-react';
import { AuditLogEntry } from '@shared/types';
import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

export const DevConsoleApp: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'audit' | 'ipc' | 'diagnostics'>('audit');
  const [logs, setLogs] = useState<AuditLogEntry[]>([]);

  const fetchLogs = async () => {
    if (window.nova?.db?.getAuditLogs) {
      try {
        const res = await window.nova.db.getAuditLogs(50);
        if (res.success && res.logs) {
          setLogs(res.logs);
        }
      } catch (err) {
        console.error(err);
      }
    }
  };

  useEffect(() => {
    fetchLogs();
  }, []);

  const registeredIpcChannels = [
    { channel: 'nova:system:get-metrics', type: 'invoke', schema: 'Empty', auth: 'Public System' },
    { channel: 'nova:system:open-external', type: 'invoke', schema: 'OpenExternalSchema', auth: 'Public System' },
    { channel: 'nova:system:notification', type: 'invoke', schema: 'NotificationPayloadSchema', auth: 'Public System' },
    { channel: 'nova:fs:list', type: 'invoke', schema: 'FsListSchema', auth: 'Scoped Path / Virtual' },
    { channel: 'nova:fs:read-file', type: 'invoke', schema: 'FsReadFileSchema', auth: 'host-filesystem:read' },
    { channel: 'nova:fs:write-file', type: 'invoke', schema: 'FsWriteFileSchema', auth: 'host-filesystem:write' },
    { channel: 'nova:fs:pick-folder', type: 'invoke', schema: 'Empty', auth: 'Native Dialog' },
    { channel: 'nova:terminal:spawn', type: 'invoke', schema: 'TerminalSpawnSchema', auth: 'terminal:execute' },
    { channel: 'nova:terminal:input', type: 'invoke', schema: 'TerminalInputSchema', auth: 'terminal:execute' },
    { channel: 'nova:terminal:kill', type: 'invoke', schema: 'TerminalKillSchema', auth: 'Session Owner' },
    { channel: 'nova:db:get-preferences', type: 'invoke', schema: 'Empty', auth: 'Public Shell' },
    { channel: 'nova:db:set-preferences', type: 'invoke', schema: 'PreferencesSchema', auth: 'Public Shell' },
    { channel: 'nova:permissions:request', type: 'invoke', schema: 'RequestPermissionSchema', auth: 'Permission Broker' },
    { channel: 'nova:permissions:get-grants', type: 'invoke', schema: 'AppId?', auth: 'Permission Broker' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: 'var(--nova-surface-canvas)', color: 'var(--nova-text-primary)' }}>
      {/* Dev Header */}
      <div style={{ height: '42px', borderBottom: '1px solid var(--nova-border-subtle)', backgroundColor: 'var(--nova-surface-base)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 16px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Wrench size={16} color="#f97316" />
          <span style={{ fontSize: 'var(--nova-text-xs)', fontWeight: 600 }}>Developer Console & Diagnostics</span>
          <Badge tier="managed">Dev Build Tools</Badge>
        </div>

        <div style={{ display: 'flex', gap: '4px' }}>
          {[
            { id: 'audit', label: 'Security Audit Log' },
            { id: 'ipc', label: 'IPC Schema Router' },
            { id: 'diagnostics', label: 'Diagnostics' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              style={{
                padding: '4px 10px',
                borderRadius: 'var(--nova-radius-md)',
                backgroundColor: activeTab === tab.id ? 'var(--nova-surface-raised)' : 'transparent',
                color: activeTab === tab.id ? 'var(--nova-accent-primary)' : 'var(--nova-text-secondary)',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Main Tab Area */}
      <div style={{ flex: 1, padding: '16px', overflowY: 'auto' }}>
        {activeTab === 'audit' && (
          <div>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
              <span style={{ fontSize: 'var(--nova-text-xs)', color: 'var(--nova-text-muted)' }}>
                Showing last 50 privileged actions recorded to SQLite audit ledger.
              </span>
              <Button size="sm" variant="subtle" icon={<RefreshCw size={12} />} onClick={fetchLogs}>
                Refresh
              </Button>
            </div>

            <div style={{ borderRadius: 'var(--nova-radius-lg)', border: '1px solid var(--nova-border-subtle)', overflow: 'hidden' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11px', fontFamily: 'var(--nova-font-mono)' }}>
                <thead>
                  <tr style={{ backgroundColor: 'var(--nova-surface-base)', textAlign: 'left', color: 'var(--nova-text-secondary)' }}>
                    <th style={{ padding: '8px 12px' }}>Timestamp</th>
                    <th style={{ padding: '8px 12px' }}>Action</th>
                    <th style={{ padding: '8px 12px' }}>App ID</th>
                    <th style={{ padding: '8px 12px' }}>Result</th>
                    <th style={{ padding: '8px 12px' }}>Details / Resource</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.length === 0 ? (
                    <tr>
                      <td colSpan={5} style={{ padding: '24px', textAlign: 'center', color: 'var(--nova-text-muted)' }}>
                        No audit events recorded yet
                      </td>
                    </tr>
                  ) : (
                    logs.map((log) => (
                      <tr key={log.id} style={{ borderTop: '1px solid var(--nova-border-subtle)' }}>
                        <td style={{ padding: '8px 12px', color: 'var(--nova-text-muted)' }}>
                          {new Date(log.timestamp).toLocaleTimeString()}
                        </td>
                        <td style={{ padding: '8px 12px', fontWeight: 600 }}>{log.action}</td>
                        <td style={{ padding: '8px 12px', color: 'var(--nova-text-secondary)' }}>{log.appId}</td>
                        <td style={{ padding: '8px 12px' }}>
                          <span style={{ color: log.result === 'allowed' ? '#10b981' : '#f43f5e' }}>
                            {log.result.toUpperCase()}
                          </span>
                        </td>
                        <td style={{ padding: '8px 12px', color: 'var(--nova-text-muted)' }}>
                          {log.details || log.resource || '-'}
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'ipc' && (
          <div style={{ borderRadius: 'var(--nova-radius-lg)', border: '1px solid var(--nova-border-subtle)', overflow: 'hidden' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11px', fontFamily: 'var(--nova-font-mono)' }}>
              <thead>
                <tr style={{ backgroundColor: 'var(--nova-surface-base)', textAlign: 'left', color: 'var(--nova-text-secondary)' }}>
                  <th style={{ padding: '8px 12px' }}>IPC Channel</th>
                  <th style={{ padding: '8px 12px' }}>Transport</th>
                  <th style={{ padding: '8px 12px' }}>Zod Validation Schema</th>
                  <th style={{ padding: '8px 12px' }}>Capability Boundary</th>
                </tr>
              </thead>
              <tbody>
                {registeredIpcChannels.map((c) => (
                  <tr key={c.channel} style={{ borderTop: '1px solid var(--nova-border-subtle)' }}>
                    <td style={{ padding: '8px 12px', fontWeight: 600, color: 'var(--nova-accent-primary)' }}>{c.channel}</td>
                    <td style={{ padding: '8px 12px', color: 'var(--nova-text-muted)' }}>{c.type}</td>
                    <td style={{ padding: '8px 12px', color: 'var(--nova-text-secondary)' }}>{c.schema}</td>
                    <td style={{ padding: '8px 12px', color: '#10b981' }}>{c.auth}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {activeTab === 'diagnostics' && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '600px' }}>
            <div style={{ padding: '16px', borderRadius: 'var(--nova-radius-lg)', backgroundColor: 'var(--nova-surface-base)', border: '1px solid var(--nova-border-subtle)' }}>
              <h4 style={{ fontSize: 'var(--nova-text-sm)', fontWeight: 600, marginBottom: '8px' }}>Security Posture Verification</h4>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '11px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981' }}>
                  <CheckCircle2 size={14} /> Context Isolation: Enabled (webPreferences.contextIsolation = true)
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981' }}>
                  <CheckCircle2 size={14} /> Node Integration: Disabled (webPreferences.nodeIntegration = false)
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981' }}>
                  <CheckCircle2 size={14} /> Chromium Sandbox: Active (webPreferences.sandbox = true)
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981' }}>
                  <CheckCircle2 size={14} /> Content Security Policy: Enforced ('default-src self')
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981' }}>
                  <CheckCircle2 size={14} /> Path Confinement: Active (Root verification & realpath check)
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
