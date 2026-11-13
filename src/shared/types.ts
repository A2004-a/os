/**
 * NovaOS Shared Types & Core Contracts
 * Identity: NovaOS — One Desktop. Three Worlds. One Experience.
 */

export type Capability =
  | 'host-filesystem:read'
  | 'host-filesystem:write'
  | 'terminal:execute'
  | 'clipboard:read'
  | 'system:metrics'
  | 'notifications:send';

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type WindowState = 'normal' | 'minimized' | 'maximized' | 'snapped';

export type SnapZone =
  | 'left-half'
  | 'right-half'
  | 'top-half'
  | 'bottom-half'
  | 'top-left'
  | 'top-right'
  | 'bottom-left'
  | 'bottom-right'
  | 'center-third'
  | 'maximize';

export interface ShellWindow {
  id: string;
  appId: string;
  title: string;
  workspaceId: string;
  bounds: Bounds;
  state: WindowState;
  snapZone?: SnapZone;
  zIndex: number;
  focused: boolean;
  restoreBounds?: Bounds;
  permissions: Capability[];
  icon?: string;
}

export interface Workspace {
  id: string;
  name: string;
  index: number;
  active: boolean;
}

export type LauncherMode = 'dock' | 'taskbar' | 'panel';
export type ThemeMode = 'dark' | 'light' | 'high-contrast';
export type AccentColor = 'cyan' | 'blue' | 'purple' | 'amber' | 'emerald' | 'rose';

export interface UserPreferences {
  theme: ThemeMode;
  accent: AccentColor;
  launcherMode: LauncherMode;
  motionReduced: boolean;
  transparencyReduced: boolean;
  scaleFactor: number;
  activeWorkspaceId: string;
  autoSnapThreshold: number;
}

export interface AppManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  icon: string;
  category: 'productivity' | 'utilities' | 'system' | 'development' | 'media';
  defaultBounds: {
    width: number;
    height: number;
    minWidth?: number;
    minHeight?: number;
  };
  requiredPermissions: Capability[];
  supportedMimeTypes?: string[];
  isSystemApp?: boolean;
}

export interface SystemMetrics {
  platform: 'win32' | 'darwin' | 'linux';
  arch: string;
  osRelease: string;
  cpuModel: string;
  cpuCount: number;
  cpuUsagePercent: number;
  totalMemoryBytes: number;
  freeMemoryBytes: number;
  uptimeSeconds: number;
  battery?: {
    hasBattery: boolean;
    isCharging: boolean;
    percent: number;
  };
}

export interface FileItem {
  id: string;
  name: string;
  path: string;
  isVirtual: boolean;
  isDirectory: boolean;
  sizeBytes: number;
  modifiedMs: number;
  mimeType?: string;
  isTrashed?: boolean;
}

export interface PermissionGrant {
  id: string;
  appId: string;
  capability: Capability;
  resourcePath?: string;
  grantedAt: number;
  expiresAt?: number;
  revoked: boolean;
}

export interface AuditLogEntry {
  id: string;
  timestamp: number;
  appId: string;
  action: string;
  resource?: string;
  result: 'allowed' | 'denied' | 'error';
  details?: string;
}

export interface ShellNotification {
  id: string;
  title: string;
  message: string;
  timestamp: number;
  sourceAppId: string;
  read: boolean;
  urgency: 'low' | 'normal' | 'critical';
}
