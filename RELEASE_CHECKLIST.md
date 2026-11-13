# NovaOS Production Release Checklist

**Document ID:** NOVA-REL-001  
**Version:** 0.1.0  
**Status:** Approved  

---

## 1. Pre-Flight Verification Gate

- [ ] **Clean Git Workspace**: Working tree is clean; no untracked experimental files or uncommitted modifications.
- [ ] **Typecheck Verification**: `npm run typecheck` exits with code 0 (zero TypeScript errors).
- [ ] **Automated Test Suite**: `npm test` runs all unit, component, accessibility, and security tests with 100% pass rate.
- [ ] **Production Build**: `npm run build` succeeds without warnings or bundle errors.
- [ ] **Security Audit**:
  - `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true` verified in Electron window creation.
  - No wildcard IPC listeners or generic invoke bridges.
  - All file path inputs canonicalized and checked for root containment.
  - Sensitive operations guarded by the `PermissionBroker`.
- [ ] **Accessibility (WCAG 2.2 AA)**:
  - Tab navigation functions smoothly through all shell surfaces.
  - High-contrast theme renders with accessible color ratios.
  - Reduced-motion mode disables non-essential animations.

---

## 2. Packaging & Distribution Verification

- [ ] **Target Artifacts Generated**:
  - Windows: Portable `.exe` or NSIS installer package.
  - Linux: `.AppImage` or `.deb`.
  - macOS: `.dmg` or `.zip`.
- [ ] **Executable Launch Test**: Verify binary launches cleanly from target distribution directory.
- [ ] **Smoke Test Built-in Apps**: Verify Files, Notes, Terminal, Settings, Calculator, Clock, System Monitor, Media Viewer, and Software Center.
- [ ] **Session Persistence**: Verify window layout and user preferences restore on application restart.

---

## 3. Post-Release & Sign-Off

- [ ] Tag release in Git (`git tag -a v0.1.0 -m "Release v0.1.0"`).
- [ ] Archive audit logs and test reports.
- [ ] Publish release notes and updated capability matrix.
