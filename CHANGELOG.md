# NovaOS Changelog

All notable changes to the NovaOS platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-11-14

### Added
- **Core Platform Architecture**: Initialized secure Electron + TypeScript platform with strict context isolation, Chromium sandbox, and defense-in-depth CSP.
- **Typed Preload Gateway**: Zero-Node-exposure preload bridge with Zod schema validation across all IPC channels.
- **Data Layer**: Embedded `node:sqlite` database for state persistence, audit logging, and virtual filesystem metadata.
- **Permission Broker**: Fine-grained capability checks for host folder access, terminal execution, and system metrics.
- **Desktop Shell & Compositor**: Responsive desktop canvas, wallpaper system, drag-selection, and icon grid.
- **Window Manager**: Full window management supporting move, resize, minimize, maximize, snap zones (half/quarter/third), virtual workspaces, and Z-index stacking.
- **Universal Launcher**: Multi-mode launcher supporting Floating Dock (macOS-style), Classic Taskbar (Windows-style), and Compact Panel (Linux-style).
- **Global Command Palette**: Instant keyboard-driven command search (`Ctrl+Space` / `Cmd+Space`) for apps, files, and settings.
- **Built-in App Ecosystem**:
  - *Files*: Dual virtual Nova Home and permitted host folder browser with recycle bin.
  - *Notes*: Markdown-capable note editor with autosave.
  - *Terminal*: Host-aware shell with working directory confinement.
  - *Settings*: Comprehensive theme, workspace, accessibility, and permission manager.
  - *System Monitor*: Real-time CPU, memory, and NovaOS resource charts.
  - *Media Viewer*: Safe image and media inspector.
  - *Calculator*: Standard and scientific calculation modes.
  - *Clock*: World clock, timer, stopwatch, and alarms.
  - *Software Center*: In-app manifest-based application catalog.
  - *Developer Console*: Diagnostics, IPC inspector, and permission audit viewer.
- **Comprehensive Documentation**: Complete architecture, security, capability, SDK, test plan, and release documentation suite.
