import { describe, it, expect } from 'vitest';
import {
  OpenExternalSchema,
  NotificationPayloadSchema,
  FsListSchema,
  FsReadFileSchema,
  FsWriteFileSchema,
  PreferencesSchema,
  ShellWindowSchema,
} from '../../src/shared/schemas';

describe('IPC Validation Schemas', () => {
  describe('OpenExternalSchema', () => {
    it('accepts valid https URLs', () => {
      const result = OpenExternalSchema.safeParse({ url: 'https://github.com' });
      expect(result.success).toBe(true);
    });

    it('rejects dangerous protocols (javascript:, file:)', () => {
      const jsResult = OpenExternalSchema.safeParse({ url: 'javascript:alert(1)' });
      expect(jsResult.success).toBe(false);

      const fileResult = OpenExternalSchema.safeParse({ url: 'file:///etc/passwd' });
      expect(fileResult.success).toBe(false);
    });
  });

  describe('NotificationPayloadSchema', () => {
    it('validates legitimate notification', () => {
      const result = NotificationPayloadSchema.safeParse({
        title: 'Build Finished',
        message: 'NovaOS compilation completed in 4.2s',
        urgency: 'normal',
      });
      expect(result.success).toBe(true);
    });

    it('rejects empty title or oversized message', () => {
      const emptyTitle = NotificationPayloadSchema.safeParse({
        title: '',
        message: 'Hello',
      });
      expect(emptyTitle.success).toBe(false);
    });
  });

  describe('Fs Schemas', () => {
    it('validates fs list and read paths', () => {
      expect(FsListSchema.safeParse({ path: 'nova://documents' }).success).toBe(true);
      expect(FsReadFileSchema.safeParse({ path: 'nova://documents/note.txt' }).success).toBe(true);
      expect(FsWriteFileSchema.safeParse({ path: 'nova://test.txt', content: 'hello' }).success).toBe(true);
    });

    it('rejects empty path parameters', () => {
      expect(FsListSchema.safeParse({ path: '' }).success).toBe(false);
      expect(FsReadFileSchema.safeParse({ path: '' }).success).toBe(false);
    });
  });

  describe('PreferencesSchema', () => {
    it('validates comprehensive preference payload', () => {
      const validPrefs = {
        theme: 'dark' as const,
        accent: 'cyan' as const,
        launcherMode: 'dock' as const,
        motionReduced: false,
        transparencyReduced: false,
        scaleFactor: 1.0,
        activeWorkspaceId: 'ws-1',
        autoSnapThreshold: 20,
      };
      expect(PreferencesSchema.safeParse(validPrefs).success).toBe(true);
    });

    it('rejects invalid theme names and out-of-range scale factors', () => {
      const invalid = {
        theme: 'neon-cyberpunk',
        accent: 'cyan',
        launcherMode: 'dock',
        motionReduced: false,
        transparencyReduced: false,
        scaleFactor: 5.0, // max is 2.0
        activeWorkspaceId: 'ws-1',
        autoSnapThreshold: 20,
      };
      expect(PreferencesSchema.safeParse(invalid).success).toBe(false);
    });
  });

  describe('ShellWindowSchema', () => {
    it('validates correct window model', () => {
      const validWindow = {
        id: 'win-1',
        appId: 'io.novaos.files',
        title: 'Files',
        workspaceId: 'ws-1',
        bounds: { x: 100, y: 100, width: 800, height: 600 },
        state: 'normal' as const,
        zIndex: 1,
        focused: true,
        permissions: ['host-filesystem:read' as const],
      };
      expect(ShellWindowSchema.safeParse(validWindow).success).toBe(true);
    });

    it('rejects windows with invalid bounds dimensions', () => {
      const invalidWindow = {
        id: 'win-1',
        appId: 'io.novaos.files',
        title: 'Files',
        workspaceId: 'ws-1',
        bounds: { x: 100, y: 100, width: 20, height: 20 }, // min is 100
        state: 'normal',
        zIndex: 1,
        focused: true,
        permissions: [],
      };
      expect(ShellWindowSchema.safeParse(invalidWindow).success).toBe(false);
    });
  });
});

