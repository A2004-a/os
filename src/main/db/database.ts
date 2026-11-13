/**
 * NovaOS SQLite Database & Migration Engine
 * Dual-engine architecture:
 * 1. Native `node:sqlite` (Node.js >= 22.5) when available
 * 2. High-performance WebAssembly SQLite via `sql.js` (for Electron / Node.js 20)
 */

import * as path from 'node:path';
import * as fs from 'node:fs';
import { app } from 'electron';

export interface ISqliteStatement {
  run(...params: any[]): { changes?: number | bigint };
  get(...params: any[]): any;
  all(...params: any[]): any[];
}

export interface ISqliteDatabase {
  exec(sql: string): void;
  prepare(sql: string): ISqliteStatement;
  close(): void;
}

class SqlJsDatabaseWrapper implements ISqliteDatabase {
  constructor(private sqlDb: any, private filePath?: string) {}

  private persist(): void {
    if (this.filePath && !this.filePath.startsWith(':memory:')) {
      try {
        const data = this.sqlDb.export();
        const buffer = Buffer.from(data);
        fs.writeFileSync(this.filePath, buffer);
      } catch (err) {
        console.error('[NovaDatabase] Failed to persist sql.js database to disk:', err);
      }
    }
  }

  exec(sql: string): void {
    this.sqlDb.exec(sql);
    this.persist();
  }

  prepare(sql: string): ISqliteStatement {
    return {
      run: (...params: any[]) => {
        const stmt = this.sqlDb.prepare(sql);
        if (params.length > 0) {
          stmt.bind(params);
        }
        stmt.step();
        stmt.free();
        const changes = this.sqlDb.getRowsModified();
        this.persist();
        return { changes };
      },
      get: (...params: any[]) => {
        const stmt = this.sqlDb.prepare(sql);
        if (params.length > 0) {
          stmt.bind(params);
        }
        let row: any = undefined;
        if (stmt.step()) {
          row = stmt.getAsObject();
        }
        stmt.free();
        return row;
      },
      all: (...params: any[]) => {
        const stmt = this.sqlDb.prepare(sql);
        if (params.length > 0) {
          stmt.bind(params);
        }
        const rows: any[] = [];
        while (stmt.step()) {
          rows.push(stmt.getAsObject());
        }
        stmt.free();
        return rows;
      },
    };
  }

  close(): void {
    this.persist();
    this.sqlDb.close();
  }
}

export class NovaDatabase {
  private db: ISqliteDatabase;

  public constructor(db: ISqliteDatabase) {
    this.db = db;
  }

  public static async create(customPath?: string): Promise<NovaDatabase> {
    let dbPath = customPath;
    if (!dbPath) {
      const userDataDir = app ? app.getPath('userData') : path.join(process.cwd(), '.novaos-data');
      if (!fs.existsSync(userDataDir)) {
        fs.mkdirSync(userDataDir, { recursive: true });
      }
      dbPath = path.join(userDataDir, 'novaos.sqlite');
    }

    let rawDb: ISqliteDatabase;

    // Check if node:sqlite is natively available
    let nativeSqliteModule: any = null;
    try {
      const moduleName = 'node:sqlite';
      nativeSqliteModule = module.require(moduleName);
    } catch {
      nativeSqliteModule = null;
    }

    if (nativeSqliteModule && nativeSqliteModule.DatabaseSync) {
      rawDb = new nativeSqliteModule.DatabaseSync(dbPath) as ISqliteDatabase;
    } else {
      // Fallback to WebAssembly sql.js
      const initSqlJs = require('sql.js');

      let wasmPath = path.join(__dirname, 'sql-wasm.wasm');
      if (!fs.existsSync(wasmPath)) {
        try {
          wasmPath = require.resolve('sql.js/dist/sql-wasm.wasm');
        } catch {
          wasmPath = path.join(process.cwd(), 'node_modules/sql.js/dist/sql-wasm.wasm');
        }
      }

      let wasmBinary: Buffer | undefined;
      if (fs.existsSync(wasmPath)) {
        wasmBinary = fs.readFileSync(wasmPath);
      }

      const SQL = await initSqlJs(wasmBinary ? { wasmBinary } : undefined);

      let initialBuffer: Buffer | null = null;
      if (dbPath && !dbPath.startsWith(':memory:') && fs.existsSync(dbPath)) {
        try {
          initialBuffer = fs.readFileSync(dbPath);
        } catch {
          initialBuffer = null;
        }
      }

      const sqlDb = initialBuffer ? new SQL.Database(initialBuffer) : new SQL.Database();
      rawDb = new SqlJsDatabaseWrapper(sqlDb, dbPath);
    }

    const instance = new NovaDatabase(rawDb);
    instance.initPragmas();
    instance.runMigrations();
    return instance;
  }

  private initPragmas(): void {
    try {
      this.db.exec('PRAGMA foreign_keys = ON;');
      this.db.exec('PRAGMA journal_mode = WAL;');
      this.db.exec('PRAGMA synchronous = NORMAL;');
    } catch (e) {
      // Some pragmas might be no-ops in memory/wasm engines
    }
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
    const result = countStmt.get() as { count: number } | undefined;
    if (!result || Number(result.count) === 0) {
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

  public getRawDb(): ISqliteDatabase {
    return this.db;
  }

  public close(): void {
    this.db.close();
  }
}
