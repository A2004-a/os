# NovaOS Apex — Recovery & System Diagnostics Manual

## Recovery Environment Overview

NovaOS Apex incorporates a dedicated recovery partition and emergency fallback boot mode.

## Recovery Capabilities

1. **Superblock & Inode Verification:** Validates `NovaFS` superblock magic (`0x4E4F5641`), bitmap allocations, and directory tree consistency.
2. **Snapshot Rollback:** Automatically restores the root volume from the secondary recovery snapshot partition if a boot failure or unrecoverable filesystem corruption is detected.
3. **Emergency Shell:** Provides a minimal standalone recovery shell if userspace daemons fail to initialize.
4. **Hardware Diagnostics:** Displays memory map anomalies, PCI controller enumeration, and CPU exception dump logs.
