import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { DatabaseSync } from 'node:sqlite';
import { FilesystemBroker } from '../../src/main/services/filesystem-broker';
import { PermissionBroker } from '../../src/main/services/permission-broker';
import * as path from 'node:path';

describe('Security: Path Traversal Defenses', () => {
  let db: DatabaseSync;
  let permBroker: PermissionBroker;
  let fsBroker: FilesystemBroker;

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

      CREATE TABLE virtual_files (
        id TEXT PRIMARY KEY,
        parent_id TEXT,
        name TEXT NOT NULL,
        path TEXT UNIQUE NOT NULL,
        is_directory INTEGER NOT NULL,
        size_bytes INTEGER NOT NULL DEFAULT 0,
        content TEXT,
        created_at INTEGER NOT NULL,
        modified_at INTEGER NOT NULL,
        is_trashed INTEGER NOT NULL DEFAULT 0
      );
    `);

    permBroker = new PermissionBroker(db);
    fsBroker = new FilesystemBroker(db, permBroker);
  });

  afterEach(() => {
    db.close();
  });

  it('rejects relative path traversal attempting to escape permitted boundary', () => {
    // Grant access to current working directory
    const approvedDir = process.cwd();
    permBroker.grantPermission('io.novaos.files', 'host-filesystem:read', approvedDir);

    // Attempt to access parent directory using relative path ..
    const escapePath = path.join(approvedDir, '..', '..', 'Windows');

    expect(() => {
      fsBroker.resolveAndVerifyHostPath('io.novaos.files', escapePath, 'host-filesystem:read');
    }).toThrow(/Access denied/);
  });

  it('logs security audit denial on traversal attempts', () => {
    const approvedDir = process.cwd();
    permBroker.grantPermission('io.novaos.malicious', 'host-filesystem:read', approvedDir);

    const escapePath = path.join(approvedDir, '..', 'secret.txt');

    try {
      fsBroker.resolveAndVerifyHostPath('io.novaos.malicious', escapePath, 'host-filesystem:read');
    } catch {
      // Expected denial
    }

    const auditLogs = permBroker.getAuditLogs();
    const denialLog = auditLogs.find((l) => l.action === 'BLOCKED_host-filesystem:read');
    expect(denialLog).toBeDefined();
    expect(denialLog?.result).toBe('denied');
    expect(denialLog?.appId).toBe('io.novaos.malicious');
  });
});
