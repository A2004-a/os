/**
 * NovaOS Preload Bridge
 * Establishes typed, narrow, secure boundary between renderer and main process
 */

import { contextBridge, ipcRenderer } from 'electron';
import {
  SystemMetrics,
  FileItem,
  UserPreferences,
  ShellWindow,
  PermissionGrant,
  AuditLogEntry,
  Capability,
} from '../shared/types';

export interface IPCResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

const novaApi = {
  system: {
    getMetrics: async (): Promise<{ success: boolean; metrics?: SystemMetrics; error?: string }> => {
      return ipcRenderer.invoke('nova:system:get-metrics');
    },
    openExternal: async (url: string): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:system:open-external', { url });
    },
    showNotification: async (
      title: string,
      message: string,
      urgency: 'low' | 'normal' | 'critical' = 'normal'
    ): Promise<{ success: boolean; shown?: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:system:notification', { title, message, urgency });
    },
  },

  fs: {
    list: async (path: string): Promise<{ success: boolean; items?: FileItem[]; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:list', { path });
    },
    readFile: async (path: string): Promise<{ success: boolean; content?: string; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:read-file', { path });
    },
    writeFile: async (path: string, content: string): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:write-file', { path, content });
    },
    createDir: async (path: string): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:create-dir', { path });
    },
    deleteFile: async (path: string, permanent: boolean = false): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:delete-file', { path, permanent });
    },
    pickFolder: async (): Promise<{ success: boolean; folder?: string | null; error?: string }> => {
      return ipcRenderer.invoke('nova:fs:pick-folder');
    },
  },

  terminal: {
    spawn: async (
      sessionId: string,
      cwd?: string
    ): Promise<{ success: boolean; shell?: string; cwd?: string; error?: string }> => {
      return ipcRenderer.invoke('nova:terminal:spawn', { sessionId, cwd });
    },
    sendInput: async (sessionId: string, data: string): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:terminal:input', { sessionId, data });
    },
    kill: async (sessionId: string): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:terminal:kill', { sessionId });
    },
    onData: (callback: (payload: { sessionId: string; data: string }) => void) => {
      const subscription = (_event: any, payload: { sessionId: string; data: string }) => callback(payload);
      ipcRenderer.on('nova:terminal:data', subscription);
      return () => {
        ipcRenderer.removeListener('nova:terminal:data', subscription);
      };
    },
    onExit: (callback: (payload: { sessionId: string; code: number | null; signal: string | null }) => void) => {
      const subscription = (
        _event: any,
        payload: { sessionId: string; code: number | null; signal: string | null }
      ) => callback(payload);
      ipcRenderer.on('nova:terminal:exit', subscription);
      return () => {
        ipcRenderer.removeListener('nova:terminal:exit', subscription);
      };
    },
  },

  db: {
    getPreferences: async (): Promise<{ success: boolean; preferences?: UserPreferences | null; error?: string }> => {
      return ipcRenderer.invoke('nova:db:get-preferences');
    },
    setPreferences: async (preferences: UserPreferences): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:db:set-preferences', preferences);
    },
    getWindowState: async (): Promise<{ success: boolean; windows?: ShellWindow[]; error?: string }> => {
      return ipcRenderer.invoke('nova:db:get-window-state');
    },
    saveWindowState: async (windows: ShellWindow[]): Promise<{ success: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:db:save-window-state', { windows });
    },
    getAuditLogs: async (limit?: number): Promise<{ success: boolean; logs?: AuditLogEntry[]; error?: string }> => {
      return ipcRenderer.invoke('nova:db:get-audit-logs', limit);
    },
  },

  permissions: {
    request: async (
      appId: string,
      capability: Capability,
      resourcePath?: string
    ): Promise<{ success: boolean; grant?: PermissionGrant; error?: string }> => {
      return ipcRenderer.invoke('nova:permissions:request', { appId, capability, resourcePath });
    },
    getGrants: async (appId?: string): Promise<{ success: boolean; grants?: PermissionGrant[]; error?: string }> => {
      return ipcRenderer.invoke('nova:permissions:get-grants', appId);
    },
    revoke: async (grantId: string): Promise<{ success: boolean; revoked?: boolean; error?: string }> => {
      return ipcRenderer.invoke('nova:permissions:revoke', { grantId });
    },
  },
};

export type NovaBridge = typeof novaApi;

contextBridge.exposeInMainWorld('nova', novaApi);

