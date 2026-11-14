//! NovaOS Apex — ELF64 Binary Loader
//! Parses 64-bit ELF binaries, validates headers, allocates user pages, maps PT_LOAD segments, and sets up user stack.

extern crate alloc;
use alloc::vec::Vec;
use crate::memory::address::{PhysAddr, VirtAddr, PAGE_SIZE};
use crate::memory::frame::alloc_frame;
use crate::memory::paging::{OffsetPageTable, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::serial;

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS_64: u8 = 2;
pub const ELF_DATA_2LSB: u8 = 1; // Little-endian
pub const EM_X86_64: u16 = 0x3E;
pub const PT_LOAD: u32 = 1;
pub const PF_X: u32 = 1 << 0;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;

pub const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_0000;
pub const USER_STACK_PAGES: usize = 4; // 16 KiB

#[derive(Debug, PartialEq, Eq)]
pub enum ElfError {
    InvalidMagic,
    Not64Bit,
    NotLittleEndian,
    UnsupportedMachine,
    InvalidHeaderSize,
    BufferTooShort,
    AllocationFailed,
    PagingFailed,
    NoLoadableSegments,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub struct LoadedElf {
    pub entry_point: u64,
    pub user_stack_top: u64,
    pub allocated_frames: Vec<PhysAddr>,
}

pub fn load_elf64(
    elf_data: &[u8],
    pml4_phys: PhysAddr,
) -> Result<LoadedElf, ElfError> {
    if elf_data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(ElfError::BufferTooShort);
    }

    let header = unsafe { &*(elf_data.as_ptr() as *const Elf64Header) };

    // 1. Validate ELF magic & architecture
    if header.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::InvalidMagic);
    }
    if header.e_ident[4] != ELF_CLASS_64 {
        return Err(ElfError::Not64Bit);
    }
    if header.e_ident[5] != ELF_DATA_2LSB {
        return Err(ElfError::NotLittleEndian);
    }
    if header.e_machine != EM_X86_64 {
        return Err(ElfError::UnsupportedMachine);
    }

    let phoff = header.e_phoff as usize;
    let phentsize = header.e_phentsize as usize;
    let phnum = header.e_phnum as usize;

    if elf_data.len() < phoff + (phentsize * phnum) {
        return Err(ElfError::BufferTooShort);
    }

    let mut mapper = OffsetPageTable::new(pml4_phys);
    let mut allocated_frames = Vec::new();
    let mut loadable_count = 0;

    // 2. Iterate Program Headers and map PT_LOAD segments
    for i in 0..phnum {
        let phdr_offset = phoff + (i * phentsize);
        let phdr = unsafe { &*(elf_data[phdr_offset..].as_ptr() as *const Elf64Phdr) };

        if phdr.p_type == PT_LOAD {
            loadable_count += 1;
            let vaddr = phdr.p_vaddr;
            let memsz = phdr.p_memsz;
            let filesz = phdr.p_filesz;
            let offset = phdr.p_offset as usize;

            let start_page = vaddr & !0xFFF;
            let end_page = (vaddr + memsz + 0xFFF) & !0xFFF;
            let page_count = ((end_page - start_page) / PAGE_SIZE as u64) as usize;

            let mut flags = PAGE_PRESENT | PAGE_USER;
            if (phdr.p_flags & PF_W) != 0 {
                flags |= PAGE_WRITABLE;
            }

            for p in 0..page_count {
                let page_vaddr = start_page + (p as u64 * PAGE_SIZE as u64);
                let frame = alloc_frame().ok_or(ElfError::AllocationFailed)?;
                allocated_frames.push(frame);

                mapper.map_page(VirtAddr::new(page_vaddr), frame, flags)
                    .map_err(|_| ElfError::PagingFailed)?;

                // Zero the physical frame
                let frame_ptr = frame.as_u64() as *mut u8;
                unsafe {
                    core::ptr::write_bytes(frame_ptr, 0, PAGE_SIZE);
                }

                // Copy file contents if this page overlaps with the file segment
                let _page_start_offset = page_vaddr.saturating_sub(vaddr) as usize;
                let page_in_segment = page_vaddr + (PAGE_SIZE as u64) > vaddr && page_vaddr < vaddr + filesz;

                if page_in_segment {
                    let seg_offset = if page_vaddr >= vaddr {
                        (page_vaddr - vaddr) as usize
                    } else {
                        0
                    };

                    let dest_offset = if page_vaddr < vaddr {
                        (vaddr - page_vaddr) as usize
                    } else {
                        0
                    };

                    let copy_len = if seg_offset < filesz as usize {
                        (filesz as usize - seg_offset).min(PAGE_SIZE - dest_offset)
                    } else {
                        0
                    };

                    if copy_len > 0 && offset + seg_offset + copy_len <= elf_data.len() {
                        unsafe {
                            let src = elf_data.as_ptr().add(offset + seg_offset);
                            let dst = frame_ptr.add(dest_offset);
                            core::ptr::copy_nonoverlapping(src, dst, copy_len);
                        }
                    }
                }
            }
        }
    }

    if loadable_count == 0 {
        return Err(ElfError::NoLoadableSegments);
    }

    // 3. Allocate and map user stack (16 KiB)
    let stack_base = USER_STACK_TOP - (USER_STACK_PAGES as u64 * PAGE_SIZE as u64);
    for p in 0..USER_STACK_PAGES {
        let page_vaddr = stack_base + (p as u64 * PAGE_SIZE as u64);
        let frame = alloc_frame().ok_or(ElfError::AllocationFailed)?;
        allocated_frames.push(frame);

        mapper.map_page(
            VirtAddr::new(page_vaddr),
            frame,
            PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER,
        ).map_err(|_| ElfError::PagingFailed)?;

        // Zero the stack frame
        let frame_ptr = frame.as_u64() as *mut u8;
        unsafe {
            core::ptr::write_bytes(frame_ptr, 0, PAGE_SIZE);
        }
    }

    serial::write_str("[ELF64] Successfully loaded binary. Entry point: 0x");
    print_hex_u64(header.e_entry);
    serial::write_str(", User stack top: 0x");
    print_hex_u64(USER_STACK_TOP);
    serial::write_str("\n");

    Ok(LoadedElf {
        entry_point: header.e_entry,
        user_stack_top: USER_STACK_TOP,
        allocated_frames,
    })
}

fn print_hex_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        buf[i] = hex_chars[(n & 0x0F) as usize];
        n >>= 4;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
