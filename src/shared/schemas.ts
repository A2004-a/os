/**
 * NovaOS Runtime Zod Validation Schemas
 * Used by the Secure IPC Router to validate all inbound IPC payloads
 */

import { z } from 'zod';

export const CapabilitySchema = z.enum([
  'host-filesystem:read',
  'host-filesystem:write',
  'terminal:execute',
  'clipboard:read',
  'system:metrics',
  'notifications:send',
]);

export const BoundsSchema = z.object({
  x: z.number(),
  y: z.number(),
  width: z.number().min(100),
  height: z.number().min(100),
});

export const SnapZoneSchema = z.enum([
  'left-half',
  'right-half',
  'top-half',
  'bottom-half',
  'top-left',
  'top-right',
  'bottom-left',
  'bottom-right',
  'center-third',
  'maximize',
]);

export const WindowStateSchema = z.enum(['normal', 'minimized', 'maximized', 'snapped']);

export const ShellWindowSchema = z.object({
  id: z.string().min(1),
  appId: z.string().min(1),
  title: z.string(),
  workspaceId: z.string(),
  bounds: BoundsSchema,
  state: WindowStateSchema,
  snapZone: SnapZoneSchema.optional(),
  zIndex: z.number().int().min(0),
  focused: z.boolean(),
  restoreBounds: BoundsSchema.optional(),
  permissions: z.array(CapabilitySchema),
  icon: z.string().optional(),
});

export const OpenExternalSchema = z.object({
  url: z.string().url().refine((url) => {
    try {
      const parsed = new URL(url);
      return ['http:', 'https:'].includes(parsed.protocol);
    } catch {
      return false;
    }
  }, { message: 'Only secure HTTP/HTTPS protocols are permitted for external links' }),
});

export const NotificationPayloadSchema = z.object({
  title: z.string().min(1).max(100),
  message: z.string().max(500),
  urgency: z.enum(['low', 'normal', 'critical']).default('normal'),
});

export const FsListSchema = z.object({
  path: z.string().min(1),
});

export const FsReadFileSchema = z.object({
  path: z.string().min(1),
});

export const FsWriteFileSchema = z.object({
  path: z.string().min(1),
  content: z.string(),
});

export const FsCreateDirSchema = z.object({
  path: z.string().min(1),
});

export const FsDeleteFileSchema = z.object({
  path: z.string().min(1),
  permanent: z.boolean().default(false),
});

export const FsRestoreFileSchema = z.object({
  id: z.string().min(1),
});

export const TerminalSpawnSchema = z.object({
  sessionId: z.string().min(1),
  cwd: z.string().optional(),
});

export const TerminalInputSchema = z.object({
  sessionId: z.string().min(1),
  data: z.string(),
});

export const TerminalKillSchema = z.object({
  sessionId: z.string().min(1),
});

export const RequestPermissionSchema = z.object({
  appId: z.string().min(1),
  capability: CapabilitySchema,
  resourcePath: z.string().optional(),
});

export const RevokePermissionSchema = z.object({
  grantId: z.string().min(1),
});

export const PreferencesSchema = z.object({
  theme: z.enum(['dark', 'light', 'high-contrast']),
  accent: z.enum(['cyan', 'blue', 'purple', 'amber', 'emerald', 'rose']),
  launcherMode: z.enum(['dock', 'taskbar', 'panel']),
  motionReduced: z.boolean(),
  transparencyReduced: z.boolean(),
  scaleFactor: z.number().min(0.5).max(2.0),
  activeWorkspaceId: z.string(),
  autoSnapThreshold: z.number().min(5).max(50),
});

export const SaveWindowStateSchema = z.object({
  windows: z.array(ShellWindowSchema),
});

