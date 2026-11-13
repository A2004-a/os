/**
 * NovaOS Permission Broker & Audit Logger
 * Manages fine-grained capability checks, user consents, and audit trails
 */

import type { ISqliteDatabase } from '../db/database';
import { Capability, PermissionGrant, AuditLogEntry } from '../../shared/types';
import * as crypto from 'node:crypto';

export class PermissionBroker {
  constructor(private db: ISqliteDatabase) {}

  public hasPermission(appId: string, capability: Capability, resourcePath?: string): boolean {
    // Built-in system apps have core permissions for metrics and notifications
    if (appId.startsWith('io.novaos.system') && (capability === 'system:metrics' || capability === 'notifications:send')) {
      return true;
    }

    const now = Date.now();
    let query = 'SELECT * FROM permissions WHERE app_id = ? AND capability = ? AND revoked = 0 AND (expires_at IS NULL OR expires_at > ?)';
    const params: (string | number)[] = [appId, capability, now];

    if (resourcePath) {
      query += ' AND (resource_path IS NULL OR resource_path = ?)';
      params.push(resourcePath);
    }

    const stmt = this.db.prepare(query);
    const row = stmt.get(...params);
    return Boolean(row);
  }

  public grantPermission(appId: string, capability: Capability, resourcePath?: string, durationMs?: number): PermissionGrant {
    const id = crypto.randomUUID();
    const now = Date.now();
    const expiresAt = durationMs ? now + durationMs : null;

    const stmt = this.db.prepare(`
      INSERT INTO permissions (id, app_id, capability, resource_path, granted_at, expires_at, revoked)
      VALUES (?, ?, ?, ?, ?, ?, 0)
    `);

    stmt.run(id, appId, capability, resourcePath || null, now, expiresAt);

    this.logAudit({
      appId,
      action: `GRANT_${capability}`,
      resource: resourcePath,
      result: 'allowed',
      details: `Granted capability ${capability} to ${appId}`,
    });

    return {
      id,
      appId,
      capability,
      resourcePath,
      grantedAt: now,
      expiresAt: expiresAt || undefined,
      revoked: false,
    };
  }

  public revokePermission(grantId: string): boolean {
    const stmt = this.db.prepare('UPDATE permissions SET revoked = 1 WHERE id = ?');
    const result = stmt.run(grantId);

    this.logAudit({
      appId: 'system',
      action: 'REVOKE_PERMISSION',
      resource: grantId,
      result: 'allowed',
      details: `Revoked grant ${grantId}`,
    });

    return Boolean(result.changes && result.changes > 0);
  }

  public getGrants(appId?: string): PermissionGrant[] {
    const now = Date.now();
    let query = 'SELECT * FROM permissions WHERE revoked = 0 AND (expires_at IS NULL OR expires_at > ?)';
    const params: (string | number)[] = [now];

    if (appId) {
      query += ' AND app_id = ?';
      params.push(appId);
    }

    const stmt = this.db.prepare(query);
    const rows = stmt.all(...params) as Array<{
      id: string;
      app_id: string;
      capability: string;
      resource_path: string | null;
      granted_at: number;
      expires_at: number | null;
      revoked: number;
    }>;

    return rows.map((r) => ({
      id: r.id,
      appId: r.app_id,
      capability: r.capability as Capability,
      resourcePath: r.resource_path || undefined,
      grantedAt: r.granted_at,
      expiresAt: r.expires_at || undefined,
      revoked: Boolean(r.revoked),
    }));
  }

  public logAudit(entry: Omit<AuditLogEntry, 'id' | 'timestamp'>): void {
    const id = crypto.randomUUID();
    const now = Date.now();

    const stmt = this.db.prepare(`
      INSERT INTO audit_log (id, timestamp, app_id, action, resource, result, details)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `);

    stmt.run(
      id,
      now,
      entry.appId,
      entry.action,
      entry.resource || null,
      entry.result,
      entry.details || null
    );
  }

  public getAuditLogs(limit: number = 100): AuditLogEntry[] {
    const stmt = this.db.prepare('SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ?');
    const rows = stmt.all(limit) as Array<{
      id: string;
      timestamp: number;
      app_id: string;
      action: string;
      resource: string | null;
      result: string;
      details: string | null;
    }>;

    return rows.map((r) => ({
      id: r.id,
      timestamp: r.timestamp,
      appId: r.app_id,
      action: r.action,
      resource: r.resource || undefined,
      result: r.result as 'allowed' | 'denied' | 'error',
      details: r.details || undefined,
    }));
  }
}

