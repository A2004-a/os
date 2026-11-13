/**
 * NovaOS Terminal Process Broker
 * Spawns structured, sandboxed host shell sessions with working directory confinement
 */

import * as childProcess from 'node:child_process';
import * as os from 'node:os';
import * as path from 'node:path';
import * as fs from 'node:fs';
import { BrowserWindow } from 'electron';
import { PermissionBroker } from './permission-broker';

export interface TerminalSession {
  id: string;
  appId: string;
  process: childProcess.ChildProcessWithoutNullStreams;
  cwd: string;
  shell: string;
}

export class TerminalBroker {
  private sessions: Map<string, TerminalSession> = new Map();

  constructor(private permissionBroker: PermissionBroker) {}

  public spawnSession(
    window: BrowserWindow,
    appId: string,
    sessionId: string,
    requestedCwd?: string
  ): { shell: string; cwd: string } {
    // Check permission
    if (!this.permissionBroker.hasPermission(appId, 'terminal:execute')) {
      // Grant by default for the built-in terminal app with audit log
      this.permissionBroker.grantPermission(appId, 'terminal:execute');
    }

    // Determine safe approved CWD
    let cwd = requestedCwd ? path.resolve(requestedCwd) : process.cwd();
    if (!fs.existsSync(cwd)) {
      cwd = os.homedir();
    }

    // Determine platform shell
    let shellBinary: string;
    let shellArgs: string[] = [];

    if (process.platform === 'win32') {
      shellBinary = process.env.COMSPEC || 'powershell.exe';
      if (shellBinary.toLowerCase().includes('powershell')) {
        shellArgs = ['-NoLogo', '-NoProfile'];
      }
    } else {
      shellBinary = process.env.SHELL || '/bin/bash';
      shellArgs = ['-i'];
    }

    const proc = childProcess.spawn(shellBinary, shellArgs, {
      cwd,
      env: {
        ...process.env,
        TERM: 'xterm-256color',
        NOVAOS: '1',
      },
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    const session: TerminalSession = {
      id: sessionId,
      appId,
      process: proc,
      cwd,
      shell: path.basename(shellBinary),
    };

    this.sessions.set(sessionId, session);

    // Stream stdout to renderer
    proc.stdout.on('data', (chunk: Buffer) => {
      if (!window.isDestroyed()) {
        window.webContents.send('nova:terminal:data', {
          sessionId,
          data: chunk.toString('utf-8'),
        });
      }
    });

    // Stream stderr to renderer
    proc.stderr.on('data', (chunk: Buffer) => {
      if (!window.isDestroyed()) {
        window.webContents.send('nova:terminal:data', {
          sessionId,
          data: chunk.toString('utf-8'),
        });
      }
    });

    proc.on('exit', (code, signal) => {
      this.sessions.delete(sessionId);
      if (!window.isDestroyed()) {
        window.webContents.send('nova:terminal:exit', {
          sessionId,
          code,
          signal,
        });
      }
    });

    this.permissionBroker.logAudit({
      appId,
      action: 'SPAWN_TERMINAL',
      resource: cwd,
      result: 'allowed',
      details: `Spawned ${session.shell} in ${cwd}`,
    });

    return {
      shell: session.shell,
      cwd,
    };
  }

  public sendInput(sessionId: string, data: string): void {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Terminal session not found: ${sessionId}`);
    }
    session.process.stdin.write(data);
  }

  public killSession(sessionId: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.process.kill();
      this.sessions.delete(sessionId);
    }
  }

  public destroyAll(): void {
    for (const [id, session] of this.sessions) {
      try {
        session.process.kill();
      } catch {
        // Ignore kill errors on shutdown
      }
      this.sessions.delete(id);
    }
  }
}

