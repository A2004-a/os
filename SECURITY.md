# NovaOS Security Policy & Hardening Specification

**Document ID:** NOVA-SEC-001  
**Lead Security Engineer:** Principal Electron Security Specialist  
**Version:** 0.1.0  
**Classification:** Public Engineering Standard  

---

## 1. Threat Model & Trust Boundaries

The primary threat model assumes that untrusted or malicious content could enter the renderer process via:
1. Malicious or compromised web content or Markdown rendered in an app.
2. Malicious user-imported files or scripts.
3. Exploitation of Chromium or Electron zero-day vulnerabilities.
4. Path traversal attacks attempting to exfiltrate sensitive host files (e.g. `~/.ssh`, Windows SAM/Registry, system files).
5. Unauthorized privilege escalation attempting to execute arbitrary host commands without user consent.

### Core Mitigations
- **Principle of Least Privilege**: The renderer operates with zero Node.js capabilities and cannot access disk, processes, or operating system APIs directly.
- **Strict Capability Boundary**: All native interactions must pass through the `PermissionBroker` and `SecureRouter` in the Main Process.
- **Fail-Secure Defaults**: Any unhandled IPC message, malformed payload, or path outside permitted roots results in an immediate failure and audit log entry.

---

## 2. Electron Hardening Configuration

NovaOS enforces the following flags on all `BrowserWindow` and `webContents` instances:

| Configuration Setting | Required Value | Purpose |
|---|---|---|
| `contextIsolation` | `true` | Prevents renderer scripts from accessing the preload context or Node internals. |
| `nodeIntegration` | `false` | Completely disables Node.js APIs in the renderer execution context. |
| `nodeIntegrationInWorker` | `false` | Prevents Web Workers from obtaining Node privileges. |
| `nodeIntegrationInSubFrames`| `false` | Prevents iframes from inheriting Node capabilities. |
| `sandbox` | `true` | Enables Chromium's multi-process sandbox for renderer processes. |
| `webSecurity` | `true` | Enforces Same-Origin Policy and prevents local file access from web contexts. |
| `allowRunningInsecureContent`| `false` | Blocks mixed HTTP/HTTPS content. |
| `experimentalFeatures` | `false` | Disables unstable or risky Chromium features. |
| `enableRemoteModule` | `false` | Eliminates the deprecated remote module attack surface. |

### Web Navigation & Popups Protection
```typescript
// Block all arbitrary new window creation and open safe links in external browser
contents.setWindowOpenHandler(({ url }) => {
  if (isSafeExternalUrl(url)) {
    shell.openExternal(url);
  }
  return { action: 'deny' };
});

// Block unauthorized navigations within the window
contents.on('will-navigate', (event, navigationUrl) => {
  const parsedUrl = new URL(navigationUrl);
  if (parsedUrl.protocol !== 'file:' && parsedUrl.origin !== 'http://localhost:5173') {
    event.preventDefault();
  }
});
```

---

## 3. Content Security Policy (CSP)

A strict Content Security Policy is delivered via HTTP headers and HTML meta tags:

```http
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: nova:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none';
```

---

## 4. IPC Router & Payload Validation

Every IPC channel is strictly registered and backed by a typed `Zod` validation schema:
- **No Wildcard Channels**: Generic `ipcRenderer.invoke(channel, data)` bridges are prohibited.
- **Schema Validation**: Payloads failing schema checks are dropped before any native code is reached.
- **Origin Validation**: The sender `webContents.id` is verified on every request to ensure requests originate from legitimate NovaOS window frames.

---

## 5. Filesystem Safety & Path Confinement

1. **Root Confinement**: All file paths are resolved to their canonical realpaths using `fs.realpathSync`.
2. **Traversal Prevention**: Relative components (`..`, `.` ) are resolved. If the resolved path does not begin with an approved root (`Nova Home` or an explicitly granted host directory), the operation is aborted with a `SECURITY_ERR_PATH_TRAVERSAL` code.
3. **Safe Trash & Deletion**: Deletions in Nova Home are routed to a virtual trash bin with restore capability. Deletion in host directories is never permanent without explicit modal confirmation.

---

## 6. Permission Broker & Audit Logging

Permissions are managed via fine-grained capabilities:
- `host-filesystem:read` — Read permitted host folder.
- `host-filesystem:write` — Write to permitted host folder.
- `terminal:execute` — Launch sandboxed terminal command.
- `clipboard:read` — Access host system clipboard.
- `system:metrics` — Read CPU, memory, and platform stats.

All permission prompts require explicit user action (no automatic approvals). Every grant, denial, and executed action is recorded in the SQLite audit log with timestamps, requested resources, and outcomes.

---

## 7. Vulnerability Disclosure

If you discover a security vulnerability within NovaOS, please submit a detailed report to `security@novaos.internal` or open a confidential security advisory. Do not disclose vulnerabilities publicly until patched.

