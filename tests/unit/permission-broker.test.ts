import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { DatabaseSync } from 'node:sqlite';
import { PermissionBroker } from '../../src/main/services/permission-broker';

describe('PermissionBroker', () => {
  let db: DatabaseSync;
  let broker: PermissionBroker;

  beforeEach(() => {
    db = new DatabaseSync(':memory:');
    db.exec(`
      CREATE TABLE permissions (
        id TEXT PRIMARY KEY,
        app_id TEXT NOT NULL,
        capability TEXT NOT NULL,
        resource_path TEXT,
        granted_at INTEGER NOT NULL,
        expires_at INTEGER,
        revoked INTEGER NOT NULL DEFAULT 0
      );

      CREATE TABLE audit_log (
        id TEXT PRIMARY KEY,
        timestamp INTEGER NOT NULL,
        app_id TEXT NOT NULL,
        action TEXT NOT NULL,
        resource TEXT,
        result TEXT NOT NULL,
        details TEXT
      );
    `);
    broker = new PermissionBroker(db);
  });

  afterEach(() => {
    db.close();
  });

  it('denies permissions by default for untrusted apps', () => {
    const granted = broker.hasPermission('com.thirdparty.app', 'host-filesystem:read');
    expect(granted).toBe(false);
  });

  it('auto-allows system apps for safe capabilities (system:metrics)', () => {
    const granted = broker.hasPermission('io.novaos.system.monitor', 'system:metrics');
    expect(granted).toBe(true);
  });

  it('grants capabilities and logs to audit trail', () => {
    const grant = broker.grantPermission('io.novaos.files', 'host-filesystem:read', 'C:\\PermittedFolder');
    expect(grant.id).toBeDefined();
    expect(grant.appId).toBe('io.novaos.files');
    expect(grant.capability).toBe('host-filesystem:read');
    expect(grant.resourcePath).toBe('C:\\PermittedFolder');

    // Check permission evaluates to true
    expect(broker.hasPermission('io.novaos.files', 'host-filesystem:read', 'C:\\PermittedFolder')).toBe(true);

    // Check audit logs
    const logs = broker.getAuditLogs();
    expect(logs.length).toBeGreaterThan(0);
    expect(logs[0].action).toBe('GRANT_host-filesystem:read');
    expect(logs[0].result).toBe('allowed');
  });

  it('revokes permissions successfully', () => {
    const grant = broker.grantPermission('io.novaos.terminal', 'terminal:execute');
    expect(broker.hasPermission('io.novaos.terminal', 'terminal:execute')).toBe(true);

    const revoked = broker.revokePermission(grant.id);
    expect(revoked).toBe(true);
    expect(broker.hasPermission('io.novaos.terminal', 'terminal:execute')).toBe(false);
  });

  it('handles expired permissions correctly', () => {
    // Grant with duration -1000ms (already expired)
    broker.grantPermission('io.novaos.temp', 'clipboard:read', undefined, -1000);
    expect(broker.hasPermission('io.novaos.temp', 'clipboard:read')).toBe(false);
  });
});

