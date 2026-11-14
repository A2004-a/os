# NovaOS Apex — Memory Model & Virtual Address Layout

## 64-bit Canonical Virtual Address Space

NovaOS Apex implements canonical 48-bit virtual addressing across standard x86_64 paging:

```text
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : Userland Canonical Space (128 TiB)
  - 0x0000_0000_0000 - 0x0000_0000_0FFF : Guard Page (Null pointer trap)
  - 0x0000_0040_0000 - 0x0000_0080_0000 : Application Code & Read-Only Data (ELF)
  - 0x0000_0080_0000 - 0x0000_1000_0000 : User Heap
  - 0x0000_7FFF_F000_0000 - 0x0000_7FFF_FFFF_0000 : User Thread Stacks

---------------- Non-Canonical Void Gap -----------------

0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel Higher-Half Space (128 TiB)
  - 0xFFFF_8000_0000_0000 : Physical Memory Direct Map (Phys-to-Virt offset)
  - 0xFFFF_8000_0020_0000 : Kernel Code & Data Sections
  - 0xFFFF_9000_0000_0000 : Kernel Dynamic Heap
  - 0xFFFF_A000_0000_0000 : Device MMIO Mappings (GOP Framebuffer, PCI BARs)
  - 0xFFFF_F000_0000_0000 : Emergency Interrupt Stack Tables (IST 1..7)
```

## Physical Frame Allocator
- 4 KiB page frame bitmap managing all physical memory discovered in the UEFI memory map.
- Fast bitwise scanning with allocation locks and bounds enforcement.

## Paging & Protection
- 4-level page table hierarchy (PML4 -> PDPT -> Page Directory -> Page Table).
- Kernel pages marked `User/Supervisor = 0` (inaccessible from Ring 3).
- Data pages marked `No-Execute (NX = 1)`.
- Write-Protection active in CR0 (`WP = 1`), preventing Ring 0 writes to read-only pages.
