/**
 * NovaOS Electron Main Process Entry Point
 * Enforces hardware isolation, CSP policies, and IPC route handling
 */

import electron, { BrowserWindow as ElectronBrowserWindow } from 'electron';
import * as childProcess from 'node:child_process';
import * as path from 'node:path';
import { NovaDatabase } from './db/database';
import { PermissionBroker } from './services/permission-broker';
import { FilesystemBroker } from './services/filesystem-broker';
import { TerminalBroker } from './services/terminal-broker';
import { SystemBroker } from './services/system-broker';
import { SecureIpcRouter } from './ipc/router';

// Auto-delegate to Electron binary if invoked directly via `node dist/main/main.js`
if (typeof electron === 'string') {
  const electronExe = electron as string;
  const child = childProcess.spawn(electronExe, [path.resolve(__dirname, 'main.js'), ...process.argv.slice(2)], {
    stdio: 'inherit',
    windowsHide: false,
  });
  child.on('close', (code) => {
    process.exit(code || 0);
  });
} else {
  const { app, BrowserWindow, session, shell } = electron as typeof import('electron');

  let mainWindow: ElectronBrowserWindow | null = null;
  let database: NovaDatabase | null = null;
  let terminalBroker: TerminalBroker | null = null;

  // Enforce single instance lock
  const gotTheLock = app.requestSingleInstanceLock();
  if (!gotTheLock) {
    app.quit();
  } else {
    app.on('second-instance', () => {
      if (mainWindow) {
        if (mainWindow.isMinimized()) mainWindow.restore();
        mainWindow.focus();
      }
    });

    app.whenReady().then(async () => {
      // 1. Initialize SQLite Database & Migrations
      database = await NovaDatabase.create();

      // 2. Initialize Service Brokers
      const permissionBroker = new PermissionBroker(database.getRawDb());
      const filesystemBroker = new FilesystemBroker(database.getRawDb(), permissionBroker);
      terminalBroker = new TerminalBroker(permissionBroker);
      const systemBroker = new SystemBroker();

      // 3. Configure Strict Content Security Policy on the session
      session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
        callback({
          responseHeaders: {
            ...details.responseHeaders,
            'Content-Security-Policy': [
              "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none';",
            ],
          },
        });
      });

      // 4. Create Main Application Window
      mainWindow = new BrowserWindow({
        title: 'NovaOS',
        width: 1440,
        height: 900,
        minWidth: 1024,
        minHeight: 700,
        backgroundColor: '#0a0d14',
        titleBarStyle: 'hidden',
        titleBarOverlay: process.platform === 'win32' ? {
          color: '#0d111a',
          symbolColor: '#94a3b8',
          height: 32,
        } : false,
        webPreferences: {
          contextIsolation: true,
          nodeIntegration: false,
          nodeIntegrationInWorker: false,
          nodeIntegrationInSubFrames: false,
          sandbox: true,
          webSecurity: true,
          allowRunningInsecureContent: false,
          preload: path.join(__dirname, '../preload/preload.js'),
        },
      });

      // 5. Hardened Navigation & External Links
      mainWindow.webContents.setWindowOpenHandler(({ url }: { url: string }) => {
        try {
          const parsed = new URL(url);
          if (['http:', 'https:'].includes(parsed.protocol)) {
            shell.openExternal(url);
          }
        } catch {
          // Drop invalid URL
        }
        return { action: 'deny' };
      });

      mainWindow.webContents.on('will-navigate', (event: any, navigationUrl: string) => {
        try {
          const parsed = new URL(navigationUrl);
          if (parsed.protocol !== 'file:' && !navigationUrl.startsWith('http://localhost:5173')) {
            event.preventDefault();
          }
        } catch {
          event.preventDefault();
        }
      });

      // 6. Initialize & Register Secure IPC Router
      const router = new SecureIpcRouter(
        mainWindow,
        database,
        permissionBroker,
        filesystemBroker,
        terminalBroker,
        systemBroker
      );
      router.registerHandlers();

      // 7. Load Renderer
      const isDev = !app.isPackaged && process.env.NODE_ENV !== 'production';
      if (isDev && process.env.VITE_DEV_SERVER_URL) {
        await mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
      } else {
        const prodPath = path.join(__dirname, '../renderer/index.html');
        await mainWindow.loadFile(prodPath);
      }

      mainWindow.on('closed', () => {
        mainWindow = null;
      });
    });

    app.on('window-all-closed', () => {
      if (terminalBroker) {
        terminalBroker.destroyAll();
      }
      if (database) {
        database.close();
      }
      if (process.platform !== 'darwin') {
        app.quit();
      }
    });
  }
}

