# NovaOS Quality Gates & Test Plan

**Document ID:** NOVA-TEST-001  
**Version:** 0.1.0  
**Status:** Approved  

---

## 1. Testing Philosophy & Verification Strategy

In accordance with NovaOS engineering principles, **no feature is complete simply because its UI exists**. A feature is accepted into the codebase only when it is backed by:
1. Typed schema validation.
2. Unit tests covering edge cases, state mutations, and error paths.
3. Security boundary verification (path traversal rejection, permission enforcement).
4. Accessibility compliance verification (keyboard navigation, ARIA attributes).
5. Manual verification on the host platform.

---

## 2. Test Execution Architecture

```text
tests/
├── unit/
│   ├── window-manager.test.ts      # Window creation, bounds, snapping math, z-order
│   ├── permission-broker.test.ts   # Capability grants, denials, and audit persistence
│   ├── filesystem-broker.test.ts   # Path canonicalization, traversal blocks, safe operations
│   ├── ipc-schemas.test.ts         # Zod validation schemas for all channels
│   └── store.test.ts               # State transitions and session persistence
├── components/
│   ├── command-palette.test.tsx    # Filter matching, keyboard focus trap, item execution
│   ├── launcher.test.tsx           # Pinned apps, badges, mode switching (dock/taskbar/panel)
│   └── desktop-canvas.test.tsx     # Icon grid, drag selection, wallpaper rendering
├── security/
│   ├── traversal-exploit.test.ts   # Directory escape attempts (../../)
│   └── permission-bypass.test.ts   # Unpermitted capability execution blocks
└── setup.ts                        # Vitest environment setup and mocks
```

---

## 3. Test Suites & Pass Criteria

| Suite | Scope | Target Pass Rate | Run Command |
|---|---|---|---|
| **TypeScript Typecheck** | Complete repository (`src/`, `tests/`) | 100% (0 errors) | `npm run typecheck` |
| **Unit Tests** | Math, logic, reducers, stores, Zod schemas | 100% | `npm test` |
| **Security Tests** | Path traversal, permission enforcement | 100% | `npm test` |
| **Accessibility Tests** | Keyboard navigation, focus trapping | 100% | `npm test` |
| **Production Build** | Renderer bundle & main process compilation | 0 build warnings | `npm run build` |

---

## 4. Manual Verification Scenarios

1. **Desktop Launch & Compositor**: Launch app, verify frameless window styling, wallpaper rendering, and responsive grid layout.
2. **Window Manager Operations**:
   - Spawn multiple windows.
   - Drag to snap to Left Half, Right Half, and Four Quadrants.
   - Maximize, minimize, and restore windows.
   - Switch active windows with Alt+Tab and verify Z-index focus.
3. **Virtual Workspaces**: Switch between Desktop 1 and Desktop 2; verify windows belong to designated workspaces.
4. **Command Palette**: Press `Ctrl+Space` / `Cmd+Space`, search for "Files", press `Enter`, and verify the Files app opens.
5. **Filesystem Confinement**:
   - Browse *Nova Home* virtual folders.
   - Attempt to read a host path outside granted folders; verify permission dialog blocks access.
6. **Terminal Execution**: Open Terminal app, run `dir` or `ls`, verify structured execution and output streaming.

