/**
 * NovaOS Secure IPC Router
 * Central dispatch for all IPC channels with schema validation and origin verification
 */

import { ipcMain, BrowserWindow, IpcMainInvokeEvent } from 'electron';
import {
  FsListSchema,
  FsReadFileSchema,
  FsWriteFileSchema,
  FsCreateDirSchema,
  FsDeleteFileSchema,
  FsRestoreFileSchema,
  TerminalSpawnSchema,
  TerminalInputSchema,
  TerminalKillSchema,
  OpenExternalSchema,
  NotificationPayloadSchema,
  PreferencesSchema,
  SaveWindowStateSchema,
  RequestPermissionSchema,
  RevokePermissionSchema,
} from '../../shared/schemas';
import { PermissionBroker } from '../services/permission-broker';
import { FilesystemBroker } from '../services/filesystem-broker';
import { TerminalBroker } from '../services/terminal-broker';
import { SystemBroker } from '../services/system-broker';
import { NovaDatabase } from '../db/database';

export class SecureIpcRouter {
  constructor(
    private mainWindow: BrowserWindow,
    private database: NovaDatabase,
    private permissionBroker: PermissionBroker,
    private filesystemBroker: FilesystemBroker,
    private terminalBroker: TerminalBroker,
    private systemBroker: SystemBroker
  ) {}

  public registerHandlers(): void {
    // Helper to verify event sender is legitimate
    const verifySender = (event: IpcMainInvokeEvent): boolean => {
      return (
        !this.mainWindow.isDestroyed() &&
        event.sender.id === this.mainWindow.webContents.id
      );
    };

    // SYSTEM METRICS
    ipcMain.handle('nova:system:get-metrics', async (event) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      return { success: true, metrics: this.systemBroker.getSystemMetrics() };
    });

    // OPEN EXTERNAL LINK
    ipcMain.handle('nova:system:open-external', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = OpenExternalSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      await this.systemBroker.openExternal(parsed.data.url);
      return { success: true };
    });

    // NATIVE NOTIFICATION
    ipcMain.handle('nova:system:notification', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = NotificationPayloadSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      const shown = this.systemBroker.showNotification(
        parsed.data.title,
        parsed.data.message,
        parsed.data.urgency
      );
      return { success: true, shown };
    });

    // FILESYSTEM: LIST
    ipcMain.handle('nova:fs:list', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = FsListSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        const items = await this.filesystemBroker.listDirectory('io.novaos.files', parsed.data.path);
        return { success: true, items };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // FILESYSTEM: READ
    ipcMain.handle('nova:fs:read-file', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = FsReadFileSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        const content = await this.filesystemBroker.readFile('io.novaos.files', parsed.data.path);
        return { success: true, content };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // FILESYSTEM: WRITE
    ipcMain.handle('nova:fs:write-file', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = FsWriteFileSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        await this.filesystemBroker.writeFile('io.novaos.files', parsed.data.path, parsed.data.content);
        return { success: true };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // FILESYSTEM: CREATE DIRECTORY
    ipcMain.handle('nova:fs:create-dir', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = FsCreateDirSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        await this.filesystemBroker.createDirectory('io.novaos.files', parsed.data.path);
        return { success: true };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // FILESYSTEM: DELETE
    ipcMain.handle('nova:fs:delete-file', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = FsDeleteFileSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        await this.filesystemBroker.deleteFile('io.novaos.files', parsed.data.path, parsed.data.permanent);
        return { success: true };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // FILESYSTEM: PICK HOST FOLDER
    ipcMain.handle('nova:fs:pick-folder', async (event) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      try {
        const folder = await this.filesystemBroker.pickHostFolder(this.mainWindow, 'io.novaos.files');
        return { success: true, folder };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // TERMINAL: SPAWN
    ipcMain.handle('nova:terminal:spawn', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = TerminalSpawnSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        const result = this.terminalBroker.spawnSession(
          this.mainWindow,
          'io.novaos.terminal',
          parsed.data.sessionId,
          parsed.data.cwd
        );
        return { success: true, ...result };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // TERMINAL: INPUT
    ipcMain.handle('nova:terminal:input', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = TerminalInputSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      try {
        this.terminalBroker.sendInput(parsed.data.sessionId, parsed.data.data);
        return { success: true };
      } catch (err: unknown) {
        return { success: false, error: (err as Error).message };
      }
    });

    // TERMINAL: KILL
    ipcMain.handle('nova:terminal:kill', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = TerminalKillSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      this.terminalBroker.killSession(parsed.data.sessionId);
      return { success: true };
    });

    // PREFERENCES: GET
    ipcMain.handle('nova:db:get-preferences', async (event) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const db = this.database.getRawDb();
      const stmt = db.prepare('SELECT value FROM preferences WHERE key = ?');
      const row = stmt.get('user_preferences') as { value: string } | undefined;
      return { success: true, preferences: row ? JSON.parse(row.value) : null };
    });

    // PREFERENCES: SET
    ipcMain.handle('nova:db:set-preferences', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = PreferencesSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      const db = this.database.getRawDb();
      const stmt = db.prepare(`
        INSERT INTO preferences (key, value) VALUES ('user_preferences', ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value
      `);
      stmt.run(JSON.stringify(parsed.data));
      return { success: true };
    });

    // WINDOW STATE: GET
    ipcMain.handle('nova:db:get-window-state', async (event) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const db = this.database.getRawDb();
      const stmt = db.prepare('SELECT * FROM window_state ORDER BY z_index ASC');
      const rows = stmt.all() as Array<{
        id: string;
        app_id: string;
        workspace_id: string;
        title: string;
        bounds_json: string;
        state: string;
        snap_zone: string | null;
        z_index: number;
        focused: number;
        restore_bounds_json: string | null;
        permissions_json: string;
      }>;

      const windows = rows.map((r) => ({
        id: r.id,
        appId: r.app_id,
        workspaceId: r.workspace_id,
        title: r.title,
        bounds: JSON.parse(r.bounds_json),
        state: r.state as any,
        snapZone: r.snap_zone || undefined,
        zIndex: r.z_index,
        focused: Boolean(r.focused),
        restoreBounds: r.restore_bounds_json ? JSON.parse(r.restore_bounds_json) : undefined,
        permissions: JSON.parse(r.permissions_json),
      }));

      return { success: true, windows };
    });

    // WINDOW STATE: SAVE
    ipcMain.handle('nova:db:save-window-state', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = SaveWindowStateSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      const db = this.database.getRawDb();
      db.exec('DELETE FROM window_state');
      const insert = db.prepare(`
        INSERT INTO window_state (id, app_id, workspace_id, title, bounds_json, state, snap_zone, z_index, focused, restore_bounds_json, permissions_json, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `);
      const now = Date.now();
      for (const w of parsed.data.windows) {
        insert.run(
          w.id,
          w.appId,
          w.workspaceId,
          w.title,
          JSON.stringify(w.bounds),
          w.state,
          w.snapZone || null,
          w.zIndex,
          w.focused ? 1 : 0,
          w.restoreBounds ? JSON.stringify(w.restoreBounds) : null,
          JSON.stringify(w.permissions),
          now
        );
      }
      return { success: true };
    });

    // PERMISSIONS: REQUEST
    ipcMain.handle('nova:permissions:request', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = RequestPermissionSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      const grant = this.permissionBroker.grantPermission(
        parsed.data.appId,
        parsed.data.capability,
        parsed.data.resourcePath
      );
      return { success: true, grant };
    });

    // PERMISSIONS: GET GRANTS
    ipcMain.handle('nova:permissions:get-grants', async (event, appId?: string) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const grants = this.permissionBroker.getGrants(appId);
      return { success: true, grants };
    });

    // PERMISSIONS: REVOKE
    ipcMain.handle('nova:permissions:revoke', async (event, rawPayload) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const parsed = RevokePermissionSchema.safeParse(rawPayload);
      if (!parsed.success) {
        return { success: false, error: parsed.error.issues[0].message };
      }
      const revoked = this.permissionBroker.revokePermission(parsed.data.grantId);
      return { success: true, revoked };
    });

    // AUDIT LOGS: GET
    ipcMain.handle('nova:db:get-audit-logs', async (event, limit?: number) => {
      if (!verifySender(event)) throw new Error('Unauthorized IPC sender');
      const logs = this.permissionBroker.getAuditLogs(limit || 100);
      return { success: true, logs };
    });
  }
}

