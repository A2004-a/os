//! NovaOS Zero — Freestanding ELF64 Userland Program Loader
//! Maps ELF PT_LOAD segments and user stacks into canonical user virtual address space (< 0x0000_7FFF_FFFF_0000).

use crate::memory::frame::BootInfoFrameAllocator;
use crate::memory::paging::{PageMapper, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS_64: u8 = 2;
pub const ELF_DATA_2LSB: u8 = 1; // Little-endian
pub const ET_EXEC: u16 = 2;      // Executable
pub const EM_X86_64: u16 = 0x3E; // AMD x86-64
pub const PT_LOAD: u32 = 1;      // Loadable segment

pub const PF_X: u32 = 1 << 0;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;

pub const USER_SPACE_TOP: u64 = 0x0000_7FFF_FFFF_0000;
pub const USER_STACK_BASE: u64 = 0x0000_7FFF_0000_0000;
pub const USER_STACK_PAGES: u64 = 4; // 16 KiB stack
pub const USER_STACK_SIZE: u64 = USER_STACK_PAGES * 4096;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Elf64Header {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub pad: [u8; 7],
    pub elf_type: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ElfError {
    BufferTooSmall,
    InvalidMagic,
    Not64Bit,
    NotLittleEndian,
    NotExecutable,
    NotX86_64,
    KernelAddressViolation,
    SegmentOutOfBounds,
    OutOfMemory,
}

#[derive(Debug, Clone, Copy)]
pub struct LoadedProgram {
    pub entry_point: u64,
    pub user_rsp: u64,
    pub brk_start: u64,
    pub segment_count: usize,
}

pub struct ElfLoader;

impl ElfLoader {
    /// Validates ELF64 header fields
    pub fn parse_header(buffer: &[u8]) -> Result<&Elf64Header, ElfError> {
        if buffer.len() < core::mem::size_of::<Elf64Header>() {
            return Err(ElfError::BufferTooSmall);
        }

        let header = unsafe { &*(buffer.as_ptr() as *const Elf64Header) };

        if header.magic != ELF_MAGIC {
            return Err(ElfError::InvalidMagic);
        }
        if header.class != ELF_CLASS_64 {
            return Err(ElfError::Not64Bit);
        }
        if header.endian != ELF_DATA_2LSB {
            return Err(ElfError::NotLittleEndian);
        }
        if header.elf_type != ET_EXEC {
            return Err(ElfError::NotExecutable);
        }
        if header.machine != EM_X86_64 {
            return Err(ElfError::NotX86_64);
        }
        if header.entry >= USER_SPACE_TOP {
            return Err(ElfError::KernelAddressViolation);
        }

        Ok(header)
    }

    /// Load an ELF binary into the active user address space
    pub unsafe fn load(
        buffer: &[u8],
        mapper: &mut PageMapper,
        allocator: &mut BootInfoFrameAllocator,
    ) -> Result<LoadedProgram, ElfError> {
        let header = Self::parse_header(buffer)?;

        let ph_offset = header.phoff as usize;
        let ph_size = header.phentsize as usize;
        let ph_count = header.phnum as usize;

        let mut max_vaddr = 0u64;
        let mut loaded_segments = 0;

        for i in 0..ph_count {
            let offset = ph_offset + i * ph_size;
            if offset + ph_size > buffer.len() {
                return Err(ElfError::BufferTooSmall);
            }

            let ph = &*(buffer.as_ptr().add(offset) as *const Elf64ProgramHeader);

            if ph.p_type == PT_LOAD {
                // Security check: ensure segment resides strictly in lower-half user space
                let end_vaddr = match ph.p_vaddr.checked_add(ph.p_memsz) {
                    Some(end) => end,
                    None => return Err(ElfError::KernelAddressViolation),
                };

                if ph.p_vaddr >= USER_SPACE_TOP || end_vaddr > USER_SPACE_TOP {
                    return Err(ElfError::KernelAddressViolation);
                }

                if ph.p_offset + ph.p_filesz > buffer.len() as u64 {
                    return Err(ElfError::SegmentOutOfBounds);
                }

                if end_vaddr > max_vaddr {
                    max_vaddr = end_vaddr;
                }

                let page_start = ph.p_vaddr & !0xFFF;
                let page_end = (end_vaddr + 4095) & !0xFFF;

                let mut page = page_start;
                while page < page_end {
                    // Check if page is already mapped
                    if mapper.translate(page).is_none() {
                        let frame = allocator.allocate_frame().ok_or(ElfError::OutOfMemory)?;
                        let flags = PAGE_USER | PAGE_WRITABLE | PAGE_PRESENT;
                        mapper.map_page(page, frame, flags, allocator)
                            .map_err(|_| ElfError::OutOfMemory)?;
                        
                        // Zero the freshly allocated page
                        core::ptr::write_bytes(page as *mut u8, 0, 4096);
                    }
                    page += 4096;
                }

                // Copy segment file contents to user virtual address
                let dest = ph.p_vaddr as *mut u8;
                let src = buffer.as_ptr().add(ph.p_offset as usize);
                core::ptr::copy_nonoverlapping(src, dest, ph.p_filesz as usize);

                // Zero out any BSS section
                if ph.p_memsz > ph.p_filesz {
                    let bss_start = dest.add(ph.p_filesz as usize);
                    let bss_len = (ph.p_memsz - ph.p_filesz) as usize;
                    core::ptr::write_bytes(bss_start, 0, bss_len);
                }

                loaded_segments += 1;
            }
        }

        // Allocate and map user stack
        let stack_start = USER_STACK_BASE;
        for i in 0..USER_STACK_PAGES {
            let page_addr = stack_start + i * 4096;
            if mapper.translate(page_addr).is_none() {
                let frame = allocator.allocate_frame().ok_or(ElfError::OutOfMemory)?;
                let flags = PAGE_USER | PAGE_WRITABLE | PAGE_PRESENT;
                mapper.map_page(page_addr, frame, flags, allocator)
                    .map_err(|_| ElfError::OutOfMemory)?;
                core::ptr::write_bytes(page_addr as *mut u8, 0, 4096);
            }
        }

        let user_rsp = USER_STACK_BASE + USER_STACK_SIZE;
        let brk_start = (max_vaddr + 4095) & !0xFFF;

        Ok(LoadedProgram {
            entry_point: header.entry,
            user_rsp,
            brk_start,
            segment_count: loaded_segments,
        })
    }
}
