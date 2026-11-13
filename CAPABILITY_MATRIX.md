# NovaOS Platform Capability Matrix

**Document ID:** NOVA-CAP-001  
**Version:** 0.1.0  
**Status:** Approved  

This document provides a transparent, granular classification of capabilities supported by NovaOS across host operating systems.

---

## 1. Capability Classification Tiers

- 🟢 **Native Integration:** A real host-supported action, such as opening a native file picker, triggering desktop notifications, reading authorized host system metrics, or spawning an interactive host shell process.
- 🟡 **Managed Integration:** A controlled operation governed through a vetted native bridge with user consent dialogs, path canonicalization, and SQLite audit logging.
- 🔵 **Simulated Environment:** In-app desktop canvas, window manager, virtual workspaces, virtual filesystem (*Nova Home*), themes, widgets, settings, and built-in apps created inside NovaOS.
- 🔴 **Unavailable or Host-Controlled:** Kernel processes, arbitrary global window interception, direct hardware driver control, bootloader settings, or unrestricted raw disk access.

---

## 2. Granular Cross-Platform Matrix

| Capability | Windows 10/11 | macOS (12+) | Modern Linux | Classification | Architectural Implementation |
|---|---|---|---|---|---|
| **In-App Desktop Compositor** | Full Support | Full Support | Full Support | 🔵 Simulated | React + TypeScript canvas with draggable icons & snap grids. |
| **Window Snapping & Grid** | Full Support | Full Support | Full Support | 🔵 Simulated | Custom math layout engine (halves, quarters, thirds). |
| **Virtual Workspaces** | Full Support | Full Support | Full Support | 🔵 Simulated | Multi-desktop virtualizer with isolated window groups. |
| **Global Command Palette** | Full Support | Full Support | Full Support | 🔵 Simulated | In-memory index of apps, actions, settings, and files. |
| **Nova Home Virtual FS** | Full Support | Full Support | Full Support | 🔵 Simulated | SQLite-indexed virtual directory hierarchy. |
| **Host Folder Browsing** | Full Support | Full Support | Full Support | 🟢 Native / 🟡 Managed | Electron `dialog.showOpenDialog` + Path Confinement Broker. |
| **Interactive Terminal** | PowerShell / CMD | zsh / bash | bash / zsh | 🟢 Native / 🟡 Managed | `child_process.spawn` with structured args & streaming stdout. |
| **System Notifications** | Windows Toast | macOS Notification | Freedesktop DBus | 🟢 Native | Electron `Notification` API with in-app center fallback. |
| **System Hardware Metrics** | Full Support | Full Support | Full Support | 🟢 Native | Node `os` module (CPU load, free memory, architecture). |
| **Host Clipboard Sync** | Explicit Opt-In | Explicit Opt-In | Explicit Opt-In | 🟢 Native / 🟡 Managed | Electron `clipboard` API gated by explicit user gesture. |
| **System Theme Adaptation** | Dark/Light sync | Dark/Light sync | Dark/Light sync | 🟢 Native | Electron `nativeTheme` events synced to Nova Surface CSS. |
| **Custom App Installation** | Nova Packages | Nova Packages | Nova Packages | 🔵 Simulated | Manifest-based local package installation & sandboxing. |
| **Global OS Window Hooking**| ❌ Not Supported | ❌ Not Supported | ❌ Not Supported | 🔴 Unavailable | NovaOS manages its own windows, not host OS windows. |
| **Kernel / Driver Control** | ❌ Not Supported | ❌ Not Supported | ❌ Not Supported | 🔴 Unavailable | Strictly out of scope; user-space application only. |
| **Raw Disk / BIOS Access**  | ❌ Not Supported | ❌ Not Supported | ❌ Not Supported | 🔴 Unavailable | Blocked by platform boundaries and security model. |

