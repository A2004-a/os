# NovaOS

> **One Desktop. Three Worlds. One Experience.**

NovaOS is an original, production-quality hybrid desktop environment engineered to blend the productivity and multitasking of **Windows**, the fluid ergonomics, search, and visual polish of **macOS**, and the deep configurability, transparency, and terminal agency of **Linux**—without copying proprietary assets, branding, or pretending to be a low-level host operating system.

---

## 🧭 Core Boundary & Product Identity

NovaOS is a desktop environment **application** running inside Electron on Windows, macOS, and Linux. It provides a cohesive, simulated desktop experience within its own window frame while offering controlled, permissioned bridges to real host features.

Capabilities are explicitly labeled throughout documentation and in the Settings application:
- 🟢 **Native Integration:** Real host-supported actions (system notifications, native file dialogs, host OS metrics, external links in default browser, sandboxed shell terminal).
- 🟡 **Managed Integration:** Operations brokered through audited internal native helpers with user permission prompts and path confinement.
- 🔵 **Simulated Environment:** In-app window compositor, virtual workspaces, virtual filesystem (*Nova Home*), widgets, dock/panel layouts, notes, and local app store.
- 🔴 **Unavailable / Host-Controlled:** Kernel operations, boot settings, global display servers, unrestricted raw filesystem access, or hardware driver installations.

---

## 🚀 Getting Started

### Prerequisites
- **Node.js**: v20.0+ (Tested on Node v26.5.0)
- **Package Manager**: npm v10+ or compatible
- **Operating System**: Windows 10/11, macOS 12+, or modern Linux (Ubuntu 22.04+, Fedora 38+)

### Installation
```bash
# Clone the repository
git clone https://github.com/amiyogarai/novaos.git
cd "AMIYO OS"

# Install dependencies
npm install
```

### Development
```bash
# Run the Vite renderer in development mode
npm run dev:renderer

# Build main and renderer
npm run build

# Start the Electron application
npm start
```

### Running Tests
```bash
# Run unit and integration tests with Vitest
npm test

# Run TypeScript typechecks
npm run typecheck
```

### Packaging & Distribution
```bash
# Package directory build
npm run package:dir

# Package Windows installer
npm run package:win
```

---

## 🏛 Architecture Overview

NovaOS follows strict Electron security principles:
1. **Hardened Electron Main Process**: Context isolation enabled, Node integration disabled, Chromium sandbox enabled, restrictive CSP applied, and navigation restricted.
2. **Narrow Typed Preload**: No arbitrary `ipcRenderer.invoke` or raw Node primitives exposed. All APIs are strictly typed and bounded.
3. **Validated IPC Router**: Every request is validated against strict Zod runtime schemas before triggering native actions.
4. **Capability Permission Broker**: Critical actions (terminal execution, host folder access, clipboard reads) require explicit user consent with audit logs.
5. **SQLite Persistence**: Embedded transactional state management using Node's zero-dependency `node:sqlite`.
6. **Nova Surface Design System**: Original typography, accessible contrast, fluid physics-based motion with reduced-motion support.

---

## 📂 Documentation

- [Architecture Guide](ARCHITECTURE.md)
- [Architecture Audit & Baseline](ARCHITECTURE_AUDIT.md)
- [Security Model & Hardening](SECURITY.md)
- [Platform Capability Matrix](CAPABILITY_MATRIX.md)
- [App SDK & Manifest Specification](APP_SDK.md)
- [Test Plan & Quality Gates](TEST_PLAN.md)
- [Known Limitations](KNOWN_LIMITATIONS.md)
- [Changelog](CHANGELOG.md)
- [Release Checklist](RELEASE_CHECKLIST.md)

---

## 📄 License
Apache-2.0 License. Developed by NovaOS Platform Engineering.
