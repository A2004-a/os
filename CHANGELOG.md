# NovaOS Changelog

All notable changes to the NovaOS platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-11-14

### Added
- **Core Platform Architecture**: Initialized secure Electron + TypeScript platform with strict context isolation, Chromium sandbox, and defense-in-depth CSP.
- **Dual-Engine SQLite**: Resilient database layer utilizing Node 20/22/26 native `node:sqlite` where available and bundled `sql.js` WebAssembly fallback for zero-native-compilation compatibility.
- **Windows 11-Level Shell Polish**:
  - *Snap Layouts Flyout*: Hover-activated layout selector on window Maximize button (50/50 split, 67/33 split, 3-column, 4-quadrant).
  - *Snap Assist*: Contextual floating picker suggesting companion windows to fill complementary screen quadrants.
  - *Nova Launcher*: Centered/left hybrid Start Menu with pinned apps grid, quick actions (New Note, Terminal, Calculator), "Continue Working" recent documents, and universal search with capability badges.
  - *Interactive Calendar Popover*: Date/time taskbar flyout with interactive month grid, today jump, and agenda schedule items.
  - *Quick Settings Flyout*: System controls for volume slider, simulated display brightness, Wi-Fi toggle, DND focus mode, and transparency effects.
  - *In-App Clipboard History*: Persistent clipboard snippets accessible via `Win+V` or launcher.
  - *Interactive Desktop*: Draggable desktop icons with persistent coordinates, rubberband box multi-selection, clean desktop mode toggle, and right-click desktop context menu.
  - *Tabbed File Explorer*: Multi-tab browsing (`+` tab, `x` tab), clickable breadcrumbs, back/forward history navigation, Grid/Details view toggle, and storage breakdown bar.
  - *Focus Sessions*: Integrated Pomodoro timer in Clock app with session tracking and Do-Not-Disturb notification suppression.
- **Typed Preload Gateway**: Zero-Node-exposure preload bridge with Zod schema validation across all IPC channels.
- **Permission Broker**: Fine-grained capability checks for host folder access, terminal execution, and system metrics.
- **Desktop Shell & Compositor**: Responsive desktop canvas, 5 abstract wallpapers, drag-selection, and icon grid.
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
  - *Clock*: World clock, timer, stopwatch, alarms, and focus sessions.
  - *Software Center*: In-app manifest-based application catalog.
  - *Developer Console*: Diagnostics, IPC inspector, and permission audit viewer.
- **Comprehensive Documentation**: Complete architecture, security, capability, SDK, test plan, and release documentation suite.

