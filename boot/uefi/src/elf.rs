//! NovaOS Apex — Freestanding ELF64 Executable Loader and Validator

use crate::uefi::BootServices;

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS_64: u8 = 2;
pub const ELF_DATA_2LSB: u8 = 1;
pub const EM_X86_64: u16 = 0x3E;
pub const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

#[repr(C)]
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

#[derive(Clone, Copy, Debug)]
pub struct LoadedKernel {
    pub entry_point: u64,
    pub phys_base: u64,
    pub virt_base: u64,
    pub total_size: u64,
}

#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    Not64Bit,
    NotLittleEndian,
    UnsupportedArchitecture,
    NoLoadableSegments,
    AllocationFailed,
    OutOfBounds,
}

pub fn validate_and_load(elf_data: &[u8], bs: &BootServices) -> Result<LoadedKernel, ElfError> {
    if elf_data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(ElfError::OutOfBounds);
    }

    let header = unsafe { &*(elf_data.as_ptr() as *const Elf64Header) };

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
        return Err(ElfError::UnsupportedArchitecture);
    }

    let ph_offset = header.e_phoff as usize;
    let ph_entsize = header.e_phentsize as usize;
    let ph_num = header.e_phnum as usize;

    if ph_offset + (ph_num * ph_entsize) > elf_data.len() {
        return Err(ElfError::OutOfBounds);
    }

    let mut min_phys: u64 = u64::MAX;
    let mut max_phys: u64 = 0;
    let mut min_virt: u64 = u64::MAX;
    let mut load_count = 0;

    for i in 0..ph_num {
        let offset = ph_offset + i * ph_entsize;
        let ph = unsafe { &*(elf_data.as_ptr().add(offset) as *const Elf64ProgramHeader) };

        if ph.p_type == PT_LOAD {
            load_count += 1;
            let target_phys = if ph.p_paddr != 0 { ph.p_paddr } else { ph.p_vaddr & 0x0000_0000_FFFF_FFFF };
            if target_phys < min_phys {
                min_phys = target_phys;
            }
            if ph.p_vaddr < min_virt {
                min_virt = ph.p_vaddr;
            }
            let end = target_phys + ph.p_memsz;
            if end > max_phys {
                max_phys = end;
            }
        }
    }

    if load_count == 0 || min_phys == u64::MAX {
        return Err(ElfError::NoLoadableSegments);
    }

    // Allocate the entire physical span for the kernel contiguously
    // In UEFI: 0 = AllocateAnyPages, 1 = AllocateMaxAddress, 2 = AllocateAddress
    let start_page = min_phys & !0xFFF;
    let end_page = (max_phys + 0xFFF) & !0xFFF;
    let total_pages = ((end_page - start_page) / 4096) as usize;

    let mut allocated_addr = start_page;
    let status = unsafe {
        (bs.allocate_pages)(2, 2, total_pages, &mut allocated_addr)
    };

    crate::serial::write_str("  [ELF] Kernel span: 0x");
    crate::serial::write_hex(start_page);
    crate::serial::write_str(" - 0x");
    crate::serial::write_hex(end_page);
    crate::serial::write_str(" (pages=0x");
    crate::serial::write_hex(total_pages as u64);
    crate::serial::write_str(", status=0x");
    crate::serial::write_hex(status as u64);
    crate::serial::write_str(")\n");

    if status != 0 {
        return Err(ElfError::AllocationFailed);
    }

    // Zero out the entire allocated kernel space
    unsafe {
        core::ptr::write_bytes(start_page as *mut u8, 0, total_pages * 4096);
    }

    // Copy file data to destination for each PT_LOAD segment
    for i in 0..ph_num {
        let offset = ph_offset + i * ph_entsize;
        let ph = unsafe { &*(elf_data.as_ptr().add(offset) as *const Elf64ProgramHeader) };

        if ph.p_type == PT_LOAD {
            let target_phys = if ph.p_paddr != 0 { ph.p_paddr } else { ph.p_vaddr & 0x0000_0000_FFFF_FFFF };
            let file_offset = ph.p_offset as usize;
            let file_size = ph.p_filesz as usize;

            if file_offset + file_size > elf_data.len() {
                return Err(ElfError::OutOfBounds);
            }

            unsafe {
                if file_size > 0 {
                    core::ptr::copy_nonoverlapping(
                        elf_data.as_ptr().add(file_offset),
                        target_phys as *mut u8,
                        file_size,
                    );
                }
            }
        }
    }

    Ok(LoadedKernel {
        entry_point: header.e_entry,
        phys_base: min_phys,
        virt_base: min_virt,
        total_size: max_phys.saturating_sub(min_phys),
    })
}
