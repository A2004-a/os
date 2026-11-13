import { vi } from 'vitest';

// Polyfill localStorage if needed
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(global, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'localStorage', {
    value: localStorageMock,
    writable: true,
  });
}

// Mock electron in unit test environment
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
