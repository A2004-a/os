import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { DatabaseSync } from 'node:sqlite';
import { PermissionBroker } from '../../src/main/services/permission-broker';

describe('Security: Permission Elevation Defenses', () => {
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

  it('prevents third-party apps from claiming system privileges', () => {
    const isAllowed = broker.hasPermission('com.malicious.script', 'system:metrics');
    expect(isAllowed).toBe(false);
  });

  it('verifies that terminal execution is denied without explicit capability grant', () => {
    expect(broker.hasPermission('io.novaos.unauthorized', 'terminal:execute')).toBe(false);

    // Grant permission
    broker.grantPermission('io.novaos.unauthorized', 'terminal:execute');
    expect(broker.hasPermission('io.novaos.unauthorized', 'terminal:execute')).toBe(true);
  });

  it('verifies revoked permissions take effect immediately', () => {
    const grant = broker.grantPermission('com.example.app', 'clipboard:read');
    expect(broker.hasPermission('com.example.app', 'clipboard:read')).toBe(true);

    broker.revokePermission(grant.id);
    expect(broker.hasPermission('com.example.app', 'clipboard:read')).toBe(false);
  });
});
