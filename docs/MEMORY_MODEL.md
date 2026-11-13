# NovaOS Memory Model Specification

## 1. Virtual Address Space Layout

```text
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : Userland Space (Lower Half, 128 TiB)
0x0000_8000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF : Non-canonical Guard Gap
0xFFFF_8000_0000_0000 - 0xFFFF_BFFF_FFFF_FFFF : Physical Memory Mapping Window
0xFFFF_C000_0000_0000 - 0xFFFF_DFFF_FFFF_FFFF : Kernel Heap (0x4444_4444_0000 default)
0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel Image Code & Static Data
```

## 2. 4-Level Paging Translation

Virtual addresses are decomposed into four 9-bit table indices:
* `PML4` (Bits 47..39): Level 4 Page Map Table
* `PDPT` (Bits 38..30): Page Directory Pointer Table
* `PD` (Bits 29..21): Page Directory
* `PT` (Bits 20..12): Page Table
* Offset (Bits 11..0): 4096-byte page offset

## 3. Kernel Dynamic Heap
* Base Address: `0x4444_4444_0000`
* Initial Size: `256 KiB`
* Allocator: `Linked-List LockedHeap` providing global allocation for `Box`, `Vec`, `String`.
