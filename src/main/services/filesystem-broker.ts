/**
 * NovaOS Filesystem Broker
 * Enforces path confinement, manages virtual Nova Home, and gates host folder operations
 */

import { DatabaseSync } from 'node:sqlite';
import * as path from 'node:path';
import * as fs from 'node:fs';
import * as crypto from 'node:crypto';
import { dialog, BrowserWindow } from 'electron';
import { FileItem } from '@shared/types';
import { PermissionBroker } from './permission-broker';

export class FilesystemBroker {
  constructor(
    private db: DatabaseSync,
    private permissionBroker: PermissionBroker
  ) {}

  public isVirtualPath(targetPath: string): boolean {
    return targetPath.startsWith('nova://');
  }

  public async listDirectory(appId: string, dirPath: string): Promise<FileItem[]> {
    if (this.isVirtualPath(dirPath)) {
      return this.listVirtualDirectory(dirPath);
    }

    return this.listHostDirectory(appId, dirPath);
  }

  private listVirtualDirectory(dirPath: string): FileItem[] {
    // Normalize path e.g. 'nova://documents'
    const normalized = dirPath.endsWith('/') && dirPath !== 'nova://' ? dirPath.slice(0, -1) : dirPath;

    // Find directory record
    const parentStmt = this.db.prepare('SELECT id FROM virtual_files WHERE path = ? AND is_directory = 1 AND is_trashed = 0');
    const parent = parentStmt.get(normalized) as { id: string } | undefined;

    if (!parent && normalized !== 'nova://') {
      throw new Error(`Directory not found: ${dirPath}`);
    }

    const parentId = parent ? parent.id : 'root';
    const stmt = this.db.prepare(`
      SELECT id, name, path, is_directory, size_bytes, modified_at, is_trashed
      FROM virtual_files
      WHERE parent_id = ? AND is_trashed = 0
      ORDER BY is_directory DESC, name ASC
    `);

    const rows = stmt.all(parentId) as Array<{
      id: string;
      name: string;
      path: string;
      is_directory: number;
      size_bytes: number;
      modified_at: number;
      is_trashed: number;
    }>;

    return rows.map((r) => ({
      id: r.id,
      name: r.name,
      path: r.path,
      isVirtual: true,
      isDirectory: Boolean(r.is_directory),
      sizeBytes: r.size_bytes,
      modifiedMs: r.modified_at,
      isTrashed: Boolean(r.is_trashed),
    }));
  }

  private async listHostDirectory(appId: string, hostPath: string): Promise<FileItem[]> {
    const canonicalPath = this.resolveAndVerifyHostPath(appId, hostPath, 'host-filesystem:read');

    const stat = fs.statSync(canonicalPath);
    if (!stat.isDirectory()) {
      throw new Error(`Path is not a directory: ${hostPath}`);
    }

    const entries = fs.readdirSync(canonicalPath, { withFileTypes: true });
    const items: FileItem[] = [];

    for (const entry of entries) {
      try {
        const itemPath = path.join(canonicalPath, entry.name);
        const itemStat = fs.statSync(itemPath);
        items.push({
          id: crypto.createHash('md5').update(itemPath).digest('hex'),
          name: entry.name,
          path: itemPath,
          isVirtual: false,
          isDirectory: entry.isDirectory(),
          sizeBytes: itemStat.size,
          modifiedMs: itemStat.mtimeMs,
        });
      } catch {
        // Skip unreadable files (permission errors)
      }
    }

    return items;
  }

  public async readFile(appId: string, filePath: string): Promise<string> {
    if (this.isVirtualPath(filePath)) {
      const stmt = this.db.prepare('SELECT content FROM virtual_files WHERE path = ? AND is_directory = 0 AND is_trashed = 0');
      const row = stmt.get(filePath) as { content: string | null } | undefined;
      if (!row || row.content === null) {
        throw new Error(`Virtual file not found: ${filePath}`);
      }
      return row.content;
    }

    const canonicalPath = this.resolveAndVerifyHostPath(appId, filePath, 'host-filesystem:read');
    return fs.readFileSync(canonicalPath, 'utf-8');
  }

  public async writeFile(appId: string, filePath: string, content: string): Promise<void> {
    const now = Date.now();

    if (this.isVirtualPath(filePath)) {
      const existingStmt = this.db.prepare('SELECT id FROM virtual_files WHERE path = ?');
      const existing = existingStmt.get(filePath) as { id: string } | undefined;

      if (existing) {
        const updateStmt = this.db.prepare(`
          UPDATE virtual_files
          SET content = ?, size_bytes = ?, modified_at = ?, is_trashed = 0
          WHERE id = ?
        `);
        updateStmt.run(content, Buffer.byteLength(content), now, existing.id);
      } else {
        const parts = filePath.replace('nova://', '').split('/');
        const fileName = parts.pop() || 'untitled.txt';
        const parentPath = parts.length > 0 ? `nova://${parts.join('/')}` : 'nova://';

        let parentId = 'root';
        if (parentPath !== 'nova://') {
          const pStmt = this.db.prepare('SELECT id FROM virtual_files WHERE path = ? AND is_directory = 1');
          const pRow = pStmt.get(parentPath) as { id: string } | undefined;
          if (pRow) {
            parentId = pRow.id;
          }
        }

        const id = crypto.randomUUID();
        const insertStmt = this.db.prepare(`
          INSERT INTO virtual_files (id, parent_id, name, path, is_directory, size_bytes, content, created_at, modified_at, is_trashed)
          VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?, 0)
        `);
        insertStmt.run(id, parentId, fileName, filePath, Buffer.byteLength(content), content, now, now);
      }
      return;
    }

    const canonicalPath = this.resolveAndVerifyHostPath(appId, filePath, 'host-filesystem:write');
    fs.writeFileSync(canonicalPath, content, 'utf-8');
  }

  public async deleteFile(appId: string, targetPath: string, permanent: boolean = false): Promise<void> {
    if (this.isVirtualPath(targetPath)) {
      if (permanent) {
        const stmt = this.db.prepare('DELETE FROM virtual_files WHERE path = ?');
        stmt.run(targetPath);
      } else {
        const stmt = this.db.prepare('UPDATE virtual_files SET is_trashed = 1, modified_at = ? WHERE path = ?');
        stmt.run(Date.now(), targetPath);
      }
      return;
    }

    const canonicalPath = this.resolveAndVerifyHostPath(appId, targetPath, 'host-filesystem:write');
    if (!permanent) {
      throw new Error('Host file trash operation must be handled with explicit confirmation');
    }
    fs.unlinkSync(canonicalPath);
  }

  public async createDirectory(appId: string, dirPath: string): Promise<void> {
    const now = Date.now();

    if (this.isVirtualPath(dirPath)) {
      const parts = dirPath.replace('nova://', '').split('/').filter(Boolean);
      const dirName = parts[parts.length - 1];
      const parentParts = parts.slice(0, -1);
      const parentPath = parentParts.length > 0 ? `nova://${parentParts.join('/')}` : 'nova://';

      let parentId = 'root';
      if (parentPath !== 'nova://') {
        const pStmt = this.db.prepare('SELECT id FROM virtual_files WHERE path = ? AND is_directory = 1');
        const pRow = pStmt.get(parentPath) as { id: string } | undefined;
        if (pRow) {
          parentId = pRow.id;
        }
      }

      const id = crypto.randomUUID();
      const insert = this.db.prepare(`
        INSERT INTO virtual_files (id, parent_id, name, path, is_directory, size_bytes, content, created_at, modified_at, is_trashed)
        VALUES (?, ?, ?, ?, 1, 0, null, ?, ?, 0)
      `);
      insert.run(id, parentId, dirName, dirPath, now, now);
      return;
    }

    const canonicalPath = this.resolveAndVerifyHostPath(appId, dirPath, 'host-filesystem:write');
    fs.mkdirSync(canonicalPath, { recursive: true });
  }

  public async pickHostFolder(window: BrowserWindow, appId: string): Promise<string | null> {
    const result = await dialog.showOpenDialog(window, {
      title: 'Authorize Folder Access in NovaOS',
      properties: ['openDirectory', 'dontAddToRecent'],
      buttonLabel: 'Authorize Access',
    });

    if (result.canceled || result.filePaths.length === 0) {
      return null;
    }

    const selectedFolder = path.resolve(result.filePaths[0]);

    // Grant read & write permissions for this specific host directory
    this.permissionBroker.grantPermission(appId, 'host-filesystem:read', selectedFolder);
    this.permissionBroker.grantPermission(appId, 'host-filesystem:write', selectedFolder);

    return selectedFolder;
  }

  public resolveAndVerifyHostPath(appId: string, rawPath: string, capability: 'host-filesystem:read' | 'host-filesystem:write'): string {
    const resolvedPath = path.resolve(rawPath);

    // Verify canonical path exists or parent exists
    let canonicalPath: string;
    try {
      canonicalPath = fs.realpathSync(resolvedPath);
    } catch {
      // If file doesn't exist yet, check its existing parent
      const parentDir = path.dirname(resolvedPath);
      const canonicalParent = fs.realpathSync(parentDir);
      canonicalPath = path.join(canonicalParent, path.basename(resolvedPath));
    }

    // Check if the canonical path is covered by any granted folder for this appId
    const grants = this.permissionBroker.getGrants(appId);
    const activeGrant = grants.find((g) => {
      if (g.capability !== capability && g.capability !== 'host-filesystem:write') {
        return false;
      }
      if (!g.resourcePath) return false;
      const grantRoot = path.resolve(g.resourcePath);
      return canonicalPath.startsWith(grantRoot);
    });

    if (!activeGrant) {
      this.permissionBroker.logAudit({
        appId,
        action: `BLOCKED_${capability}`,
        resource: rawPath,
        result: 'denied',
        details: `Access to ${canonicalPath} denied: Outside permitted roots`,
      });
      throw new Error(`Access denied: Folder not authorized for ${appId}. Use the folder picker to grant permission.`);
    }

    return canonicalPath;
  }
}
