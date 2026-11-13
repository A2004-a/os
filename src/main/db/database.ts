/**
 * NovaOS SQLite Database & Migration Engine
 * Powered by Node.js built-in `node:sqlite` (zero-dependency native execution)
 */

import { DatabaseSync } from 'node:sqlite';
import * as path from 'node:path';
import * as fs from 'node:fs';
import { app } from 'electron';

export class NovaDatabase {
  private db: DatabaseSync;

  constructor(customPath?: string) {
    let dbPath = customPath;
    if (!dbPath) {
      const userDataDir = app ? app.getPath('userData') : path.join(process.cwd(), '.novaos-data');
      if (!fs.existsSync(userDataDir)) {
        fs.mkdirSync(userDataDir, { recursive: true });
      }
      dbPath = path.join(userDataDir, 'novaos.sqlite');
    }

    this.db = new DatabaseSync(dbPath);
    this.initPragmas();
    this.runMigrations();
  }

  private initPragmas(): void {
    this.db.exec('PRAGMA journal_mode = WAL;');
    this.db.exec('PRAGMA foreign_keys = ON;');
    this.db.exec('PRAGMA synchronous = NORMAL;');
  }

  private runMigrations(): void {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version INTEGER PRIMARY KEY,
        applied_at INTEGER NOT NULL
      );

      CREATE TABLE IF NOT EXISTS preferences (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS window_state (
        id TEXT PRIMARY KEY,
        app_id TEXT NOT NULL,
        workspace_id TEXT NOT NULL,
        title TEXT NOT NULL,
        bounds_json TEXT NOT NULL,
        state TEXT NOT NULL,
        snap_zone TEXT,
        z_index INTEGER NOT NULL,
        focused INTEGER NOT NULL,
        restore_bounds_json TEXT,
        permissions_json TEXT NOT NULL,
        updated_at INTEGER NOT NULL
      );

      CREATE TABLE IF NOT EXISTS permissions (
        id TEXT PRIMARY KEY,
        app_id TEXT NOT NULL,
        capability TEXT NOT NULL,
        resource_path TEXT,
        granted_at INTEGER NOT NULL,
        expires_at INTEGER,
        revoked INTEGER NOT NULL DEFAULT 0
      );

      CREATE TABLE IF NOT EXISTS audit_log (
        id TEXT PRIMARY KEY,
        timestamp INTEGER NOT NULL,
        app_id TEXT NOT NULL,
        action TEXT NOT NULL,
        resource TEXT,
        result TEXT NOT NULL,
        details TEXT
      );

      CREATE TABLE IF NOT EXISTS virtual_files (
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

      CREATE TABLE IF NOT EXISTS notes (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        category TEXT NOT NULL DEFAULT 'General',
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      );
    `);

    // Seed default Nova Home folder hierarchy if virtual_files is empty
    const countStmt = this.db.prepare('SELECT COUNT(*) as count FROM virtual_files');
    const result = countStmt.get() as { count: number };
    if (result.count === 0) {
      this.seedNovaHome();
    }
  }

  private seedNovaHome(): void {
    const now = Date.now();
    const insert = this.db.prepare(`
      INSERT INTO virtual_files (id, parent_id, name, path, is_directory, size_bytes, content, created_at, modified_at, is_trashed)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
    `);

    // Root virtual directory
    insert.run('root', null, 'Nova Home', 'nova://', 1, 0, null, now, now);
    insert.run('dir-docs', 'root', 'Documents', 'nova://documents', 1, 0, null, now, now);
    insert.run('dir-downloads', 'root', 'Downloads', 'nova://downloads', 1, 0, null, now, now);
    insert.run('dir-projects', 'root', 'Projects', 'nova://projects', 1, 0, null, now, now);
    insert.run('dir-media', 'root', 'Media', 'nova://media', 1, 0, null, now, now);

    // Initial Welcome Guide file
    const welcomeText = `# Welcome to NovaOS
One Desktop. Three Worlds. One Experience.

NovaOS combines:
- Windows-style multitasking and flexible window snapping.
- macOS-style fluid motion, global command palette, and focused ergonomics.
- Linux-style customization, transparent capability permissions, and developer tools.

Explore your new hybrid desktop!`;

    insert.run(
      'file-welcome',
      'dir-docs',
      'Welcome to NovaOS.md',
      'nova://documents/Welcome to NovaOS.md',
      0,
      welcomeText.length,
      welcomeText,
      now,
      now
    );
  }

  public getRawDb(): DatabaseSync {
    return this.db;
  }

  public close(): void {
    this.db.close();
  }
}
