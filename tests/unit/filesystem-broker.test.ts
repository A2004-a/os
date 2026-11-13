import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { DatabaseSync } from 'node:sqlite';
import { FilesystemBroker } from '../../src/main/services/filesystem-broker';
import { PermissionBroker } from '../../src/main/services/permission-broker';

describe('FilesystemBroker & Path Confinement', () => {
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

    // Seed root and documents
    const now = Date.now();
    db.exec(`
      INSERT INTO virtual_files (id, parent_id, name, path, is_directory, size_bytes, content, created_at, modified_at, is_trashed)
      VALUES
        ('root', null, 'Nova Home', 'nova://', 1, 0, null, ${now}, ${now}, 0),
        ('dir-docs', 'root', 'Documents', 'nova://documents', 1, 0, null, ${now}, ${now}, 0);
    `);

    permBroker = new PermissionBroker(db);
    fsBroker = new FilesystemBroker(db, permBroker);
  });

  afterEach(() => {
    db.close();
  });

  describe('Virtual Filesystem (Nova Home)', () => {
    it('lists seeded directories', async () => {
      const items = await fsBroker.listDirectory('io.novaos.files', 'nova://');
      expect(items.length).toBe(1);
      expect(items[0].name).toBe('Documents');
      expect(items[0].isVirtual).toBe(true);
      expect(items[0].isDirectory).toBe(true);
    });

    it('writes and reads virtual files', async () => {
      await fsBroker.writeFile('io.novaos.files', 'nova://documents/hello.txt', 'Hello NovaOS!');
      const content = await fsBroker.readFile('io.novaos.files', 'nova://documents/hello.txt');
      expect(content).toBe('Hello NovaOS!');
    });

    it('soft deletes virtual files to trash', async () => {
      await fsBroker.writeFile('io.novaos.files', 'nova://documents/temp.txt', 'Temporary data');
      await fsBroker.deleteFile('io.novaos.files', 'nova://documents/temp.txt', false);

      const items = await fsBroker.listDirectory('io.novaos.files', 'nova://documents');
      expect(items.find((i) => i.name === 'temp.txt')).toBeUndefined();
    });
  });

  describe('Path Confinement & Traversal Prevention', () => {
    it('blocks host access if no grant exists', () => {
      expect(() => {
        fsBroker.resolveAndVerifyHostPath('io.novaos.files', 'C:\\Windows\\System32', 'host-filesystem:read');
      }).toThrow(/Access denied/);
    });

    it('blocks directory traversal attempts escaping granted roots', () => {
      // Grant access only to C:\SafeZone
      permBroker.grantPermission('io.novaos.files', 'host-filesystem:read', 'C:\\SafeZone');

      expect(() => {
        // Attempt traversal out of C:\SafeZone
        fsBroker.resolveAndVerifyHostPath('io.novaos.files', 'C:\\SafeZone\\..\\Windows', 'host-filesystem:read');
      }).toThrow(/Access denied/);
    });
  });
});

