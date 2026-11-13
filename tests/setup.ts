import { vi } from 'vitest';

// Mock electron in unit test environment if not present
vi.mock('electron', () => {
  return {
    app: {
      getPath: () => './.novaos-test-data',
      requestSingleInstanceLock: () => true,
      quit: vi.fn(),
      whenReady: () => Promise.resolve(),
      on: vi.fn(),
    },
    ipcMain: {
      handle: vi.fn(),
    },
    dialog: {
      showOpenDialog: vi.fn(),
    },
    Notification: {
      isSupported: () => true,
    },
    shell: {
      openExternal: vi.fn(),
    },
  };
});
