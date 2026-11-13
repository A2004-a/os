# NovaOS — Architecture Audit & Baseline Report

**Document ID:** NOVA-AUDIT-001  
**Project Identity:** NovaOS — One Desktop. Three Worlds. One Experience.  
**Lead Architect:** Principal Systems & Electron Security Architect  
**Date:** November 14, 2026  
**Status:** Approved Baseline  

---

## 1. Executive Summary

NovaOS is conceived as an original, production-quality hybrid desktop environment engineered to combine the productivity and window management of Windows, the visual polish and focused ergonomics of macOS, and the transparency, configurability, and agency of Linux. 

Crucially, **NovaOS is an application running on top of Windows, macOS, and Linux**. It is strictly not a bootable kernel, driver stack, or full host operating system replacement. All capabilities are systematically categorized and labeled into four explicit tiers:
1. **Native integration:** Sandboxed host capabilities (e.g., native file pickers, system notifications, secure shell terminal spawning, display metrics).
2. **Managed integration:** Controlled operations brokered through vetted native adapters with user permission and audit logging.
3. **Simulated environment:** In-app desktop canvas, window compositor, virtual workspaces, virtual filesystem (Nova Home), themes, widgets, and in-app software catalog.
4. **Unavailable or host-controlled:** Host kernel processes, global window manager hijacking, hardware drivers, BIOS/boot settings, and unrestricted raw host disk access.

---

## 2. Workspace & Environment Baseline

A rigorous inspection of the workspace root (`c:\Users\amiyo garai\OneDrive\Documents\AMIYO OS`) was conducted:

| Item | Observed State | Architectural Implication |
|---|---|---|
| **Repository Root** | `C:\Users\amiyo garai\OneDrive\Documents\AMIYO OS` | Dedicated project directory under user documents. |
| **Existing Codebase** | Greenfield / Empty directory | No legacy technical debt or broken legacy dependencies to preserve; clean architectural runway. |
| **Version Control** | Initialized Git repository (`git init`) | Commit history and traceable incremental milestones established. |
| **Node.js Runtime** | Node.js `v26.5.0` | Modern LTS/Current runtime with native `node:sqlite` built-in (eliminates ABI compilation issues). |
| **Package Manager** | `npm v11.17.0` | Standard, reliable package management with lockfile integrity (`package-lock.json`). |
| **Host Operating System** | Windows 11 (64-bit) | Target primary development platform with cross-platform abstractions for macOS and Linux. |
| **Sandbox Execution** | Standard Terminal Sandbox | Validated sandboxed tool execution for build and test workflows. |

---

## 3. Risk Register & Mitigations

| Risk ID | Category | Risk Description | Severity | Mitigation Strategy |
|---|---|---|---|---|
| **RSK-01** | Security | Remote Code Execution via Electron renderer compromise or malicious preload exposure. | High | Strict Electron isolation (`contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`), strict Content Security Policy, zero generic IPC bridges (`invoke(channel, any)`), and payload validation with Zod schemas. |
| **RSK-02** | Security | Unrestricted filesystem access or path traversal leaking sensitive host files. | High | Multi-tiered filesystem architecture. Nova Home is a safe virtual filesystem. Host folder access requires explicit user grants via native pickers, canonical path resolution, and path confinement enforcement. |
| **RSK-03** | Stability | Native module compilation failures across platforms (Windows vs Linux vs macOS). | Medium | Use Node.js built-in `node:sqlite` (introduced in Node 22.5+) with zero native C++ build requirements, preventing ABI mismatch and compilation errors. |
| **RSK-04** | Performance | Heavy DOM tree or unoptimized React re-renders in desktop window compositor. | Medium | Virtualized desktop canvas, lightweight window frames, CSS transform hardware acceleration, lazy-loaded built-in applications, and bounded event queues. |
| **RSK-05** | Legal & Brand | Imitating proprietary trademarks, logos, sounds, or exact layouts of Windows, macOS, or GNOME/KDE. | High | Original design system named **Nova Surface**, unique iconography, original layout geometry, distinct brand voice, and clear disclaimers. |
| **RSK-06** | Developer / Terminal | Command injection through shell execution. | High | Terminal broker uses structured process spawning (`child_process.spawn`) with explicit argument arrays, approved working directories, and user opt-in confirmation. |

---

## 4. Platform Capability Matrix Baseline

| Capability Area | Windows | macOS | Linux | Implementation Mechanism |
|---|---|---|---|---|
| **Application Window & Shell** | Full Support | Full Support | Full Support | Electron `BrowserWindow` with custom frameless window styling. |
| **In-App Window Manager** | Full Support | Full Support | Full Support | Custom TypeScript window compositor (snapping, stacking, workspaces). |
| **Virtual Filesystem (Nova Home)** | Full Support | Full Support | Full Support | SQLite-backed metadata + local app data storage. |
| **Permitted Host Folder Access** | Full Support | Full Support | Full Support | Native `dialog.showOpenDialog` with path confinement. |
| **Interactive Terminal** | PowerShell / cmd | zsh / bash | bash / zsh | Platform adapter with structured process spawning. |
| **System Notifications** | Native Notification API | Native Notification API | Freedesktop Notification | Electron `Notification` API with in-app notification center fallback. |
| **System Metrics** | Node `os` module | Node `os` module | Node `os` module | CPU, memory, uptime, platform stats via secure IPC. |
| **Theme & Accent Adaptation** | Native / Custom | Native / Custom | Native / Custom | Electron `nativeTheme` + Nova Surface theme engine. |

---

## 5. Implementation Backlog & Phased Roadmap

### Phase 0: Baseline & Architectural Foundation (Current)
- [x] Workspace inspection and environment verification.
- [x] Git initialization.
- [x] Architecture audit (`ARCHITECTURE_AUDIT.md`) and baseline documentation.
- [ ] Core package dependencies, build scripts, TypeScript configuration, and Vitest test runner.

### Phase 1: Secure Platform Foundation
- [ ] Secure Electron main process architecture (`main.ts`, lifecycle, crash handling).
- [ ] Hardened Electron configuration (`contextIsolation`, `nodeIntegration: false`, sandbox, CSP).
- [ ] Type-safe Preload bridge with zero raw Node access.
- [ ] IPC Router with Zod schema validation for all channels.
- [ ] Permission Broker (grants, persistence, audit logging).
- [ ] Data Layer with SQLite migrations (`node:sqlite`).
- [ ] Nova Surface Design System tokens and foundational UI components.

### Phase 2: Desktop Shell & Window Manager
- [ ] Desktop canvas with wallpaper, icon grid, selection box, and context menus.
- [ ] Configurable launcher (Dock / Taskbar / Panel modes) with pinned apps, active indicators.
- [ ] Window Manager core: Z-order stack, focus management, move, resize, minimize, maximize.
- [ ] Snap Engine: Half split, quarter quadrants, third split, keyboard shortcuts.
- [ ] Virtual Workspaces with independent window assignments and overview mode.
- [ ] Global Command Palette (Cmd/Ctrl + Space / K) for apps, commands, and files.
- [ ] System status bar, Notification Center, and Quick Settings drawer.

### Phase 3: Filesystem, Search & Settings
- [ ] Layered filesystem broker: Nova Home virtual hierarchy + Permitted host folders.
- [ ] Safe file operations (create, copy, move, rename, trash, restore) with path confinement.
- [ ] Files App: Dual-pane/tabbed navigation, breadcrumbs, search, previews, recycle bin.
- [ ] Settings App: Appearance, Workspaces, Permissions, Storage, Shortcuts, About.
- [ ] Full-text search indexer for virtual files, notes, and system settings.

### Phase 4: Built-in App Ecosystem & Developer Experience
- [ ] Notes App: Local rich/plain text notes with autosave, categories, search.
- [ ] Terminal App: Secure, host-aware shell with working directory confinement and output streaming.
- [ ] Calculator App: Keyboard-accessible standard and scientific modes.
- [ ] Clock App: World clocks, stopwatch, timer, alarms with notification dispatch.
- [ ] System Monitor: Real-time memory, CPU, process breakdown, and NovaOS resource stats.
- [ ] Media Viewer: Image and audio/video previewer with safe local-file access.
- [ ] Developer Console: Live IPC inspector, permission auditor, manifest viewer, system diagnostics.

### Phase 5: Software Center & App Manifest System
- [ ] Formal App Manifest schema (`AppManifest`) and capability declarations.
- [ ] Software Center UI: Catalog browsing, installed app management, permission reviews.
- [ ] App lifecycle manager: isolated sandboxed runtime, state persistence, rollback support.

### Phase 6: Platform Integration, Performance & Security Audit
- [ ] Platform adapters for Windows, macOS, and Linux conventions.
- [ ] Accessibility compliance (WCAG 2.2 AA keyboard navigation, focus trap, screen-reader labels).
- [ ] Performance optimizations (render budgets, virtualized lists, lazy loading).
- [ ] Automated security test suite (IPC fuzzing, path traversal rejection, permission elevation blocking).

### Phase 7: Release Engineering & Complete Documentation
- [ ] Packaging configuration (`electron-builder` configuration for Windows installer / portable, Linux AppImage/deb, macOS).
- [ ] Complete documentation suite (`README.md`, `ARCHITECTURE.md`, `SECURITY.md`, `CAPABILITY_MATRIX.md`, `APP_SDK.md`, `TEST_PLAN.md`, `CHANGELOG.md`, `KNOWN_LIMITATIONS.md`, `RELEASE_CHECKLIST.md`).
- [ ] Release verification and quality sign-off.

