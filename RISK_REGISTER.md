# NovaOS — Risk Register

**Document ID:** NOVA-RISK-001  
**Project:** NovaOS — One Desktop. Three Worlds. One Experience.  
**Date:** November 14, 2026  
**Status:** Active Tracking

---

## 1. Risk Matrix Overview

Risks are categorized across Security, Architecture, Performance, Usability, and Platform Compatibility.

| Risk ID | Category | Description | Severity | Probability | Impact | Mitigation Strategy | Status |
|:---|:---|:---|:---:|:---:|:---:|:---|:---:|
| **RSK-SEC-01** | Security | Malicious renderer escape or RCE via electron preload API | Critical | Low | Severe | `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`, CSP, typed `window.nova` API without generic `ipcRenderer.send/invoke`. | Mitigated |
| **RSK-SEC-02** | Security | Arbitrary host filesystem read/write or path traversal via `..` | High | Medium | High | Multi-tier filesystem: virtual `nova://` database storage + strict path canonicalization (`realpath`) and folder boundary checks for permitted host paths. | Mitigated |
| **RSK-SEC-03** | Security | Untrusted app elevating permissions without user consent | High | Medium | High | Capability-based permission broker with explicit user confirmation modals and immutable audit logs. | Mitigated |
| **RSK-SEC-04** | Security | Command injection via terminal emulator process spawning | High | Low | High | Structured process spawning with fixed argument arrays (`child_process.spawn`), no raw shell evaluation string, sandboxed working directories. | Mitigated |
| **RSK-PERF-01** | Performance | Heavy DOM tree or animation lag when running 10+ windows on desktop | Medium | Medium | Medium | CSS `will-change: transform`, virtualized offscreen windows, debounced resize/drag handlers, low-overhead SVG icons. | In Progress |
| **RSK-PERF-02** | Performance | Large file lists causing UI freeze in Nova Files explorer | Medium | Medium | Medium | Virtualized list rendering, paginated directory querying, cached metadata. | In Progress |
| **RSK-UX-01** | UX / Legal | Imitating proprietary Windows 11 / macOS icons, branding, or assets | High | Low | Severe | Original **Nova Surface** design language, custom geometric vector icons, distinct terminology, and clear disclaimer of application nature. | Mitigated |
| **RSK-UX-02** | UX / A11y | Keyboard trapping or inaccessibility in custom window frames and modals | Medium | Medium | High | Tab navigation traps for dialogs, global `Esc` key handling, `aria-label` attributes on all interactive controls. | In Progress |
| **RSK-COMPAT-01**| Platform | Cross-platform SQLite engine divergence (Node 20 vs Node 26) | High | High | High | Dual-engine SQLite adapter: native `node:sqlite` on Node >= 22 + WASM `sql.js` inside Electron (Node 20), prebundled wasm. | Mitigated |
| **RSK-COMPAT-02**| Platform | Shell selection differences across Windows (pwsh/cmd) and Unix (bash/zsh) | Medium | Medium | Medium | Platform-aware shell resolver in `TerminalBroker` defaulting to appropriate system shell per OS. | Mitigated |
