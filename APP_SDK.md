# NovaOS App SDK & Manifest Specification

**Document ID:** NOVA-SDK-001  
**Version:** 0.1.0  
**Status:** Approved  

---

## 1. Overview

The NovaOS Application SDK provides a standardized, sandboxed runtime contract for all built-in applications and third-party Nova packages. Applications run inside the sandboxed renderer shell and access platform capabilities exclusively through declared manifests and the `window.nova` API bridge.

---

## 2. App Manifest Schema

Every application must provide an `AppManifest` definition conforming to the following TypeScript interface:

```typescript
export type AppCapability =
  | 'host-filesystem:read'
  | 'host-filesystem:write'
  | 'terminal:execute'
  | 'clipboard:read'
  | 'system:metrics'
  | 'notifications:send';

export interface AppManifest {
  /** Unique reverse-DNS identifier (e.g., 'io.novaos.files') */
  id: string;
  /** Human-readable display name */
  name: string;
  /** SemVer-compliant version string */
  version: string;
  /** Short description of application purpose */
  description: string;
  /** Icon identifier from Nova Surface icon set */
  icon: string;
  /** Category for launcher grouping */
  category: 'productivity' | 'utilities' | 'system' | 'development' | 'media';
  /** Default window geometry */
  defaultBounds: {
    width: number;
    height: number;
    minWidth?: number;
    minHeight?: number;
  };
  /** Requested capabilities */
  requiredPermissions: AppCapability[];
  /** Supported file extensions if registered as a file handler */
  supportedMimeTypes?: string[];
  /** Flag indicating whether the app is a core system app */
  isSystemApp?: boolean;
}
```

---

## 3. Application Lifecycle

Applications implement standard lifecycle callbacks to maintain performance and session continuity:

```typescript
export interface NovaAppInstance {
  /** Called when window is instantiated */
  onMount?: () => void | Promise<void>;
  /** Called when app loses focus or is minimized */
  onPause?: () => void;
  /** Called when app receives focus */
  onResume?: () => void;
  /** Called before window close to serialize state for session restore */
  onSaveState?: () => Record<string, unknown>;
  /** Called to restore previous state on session reload */
  onRestoreState?: (state: Record<string, unknown>) => void;
  /** Called on window destruction for cleanup */
  onDestroy?: () => void;
}
```

---

## 4. Sandboxed Runtime APIs

Applications access platform services via `window.nova`:

```typescript
// Example: Checking permission and listing files
async function loadFiles() {
  const result = await window.nova.fs.list({ path: 'nova://documents' });
  if (result.success) {
    console.log('Files:', result.items);
  } else {
    console.error('Error:', result.error);
  }
}
```

### Constraints
1. **No Node Primitives**: Calling `require()`, `process.exit()`, or accessing `Buffer` is strictly impossible in renderer code.
2. **Path Confinement**: All file paths must be prefixed with `nova://` or reside within a user-approved host folder path.
3. **No Direct DOM Injection**: Apps cannot inject scripts into other app windows or the shell UI.
