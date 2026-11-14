# NovaOS Apex — Installer, Storage Provisioning & System Updates

## Installation Media Layout

NovaOS Apex installer media boots directly from a FAT32 USB drive or ISO:

```text
/
├── EFI/
│   └── BOOT/
│       └── BOOTX64.EFI        # 64-bit UEFI Boot Manager
├── novaos-apex.elf            # Higher-half monolithic kernel
└── novaos.cfg                 # Boot configuration
```

## Automated Partitioning Strategy

Target storage drives are partitioned using standard GUID Partition Table (GPT):

1. **Partition 1 (ESP):** LBA 34..64, FAT32 format, Type GUID `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`. Holds UEFI bootloader.
2. **Partition 2 (Root):** LBA 65..(N - 50), NovaFS format, Type GUID `D4C3B2A1-F6E5-5B4A-8C9D-0E1F2A3B4C5D`. Holds root filesystem.
3. **Partition 3 (Recovery):** LBA (N - 49)..(N - 34), Type GUID `A1B2C3D4-E5F6-4A5B-9D8C-5D4C3B2A1F0E`. Holds fallback kernel snapshot.

## Atomic System Updates

- Updates are delivered as signed package bundles.
- Staged into an alternate partition or verified before activating.
- If an update fails health checks on the next boot, the system automatically rolls back to the recovery snapshot.
