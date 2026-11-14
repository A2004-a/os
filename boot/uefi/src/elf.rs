//! NovaOS Zero — Freestanding ELF64 Executable Validator & Segment Loader

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELF_CLASS_64: u8 = 2;
pub const ELF_DATA_2LSB: u8 = 1; // Little-endian
pub const ET_EXEC: u16 = 2;      // Executable file
pub const EM_X86_64: u16 = 0x3E; // AMD x86-64 architecture
pub const PT_LOAD: u32 = 1;      // Loadable program segment

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    Not64Bit,
    NotLittleEndian,
    NotExecutable,
    NotX86_64,
    BufferTooSmall,
}

impl Elf64Header {
    /// Validates that raw bytes represent a valid AMD64 ELF executable
    pub fn parse(buffer: &[u8]) -> Result<&Self, ElfError> {
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

        Ok(header)
    }

    /// Load all PT_LOAD segments into their specified memory addresses
    pub unsafe fn load_segments(&self, raw_elf: &[u8]) -> Result<(), ElfError> {
        let ph_offset = self.phoff as usize;
        let ph_size = self.phentsize as usize;
        let ph_count = self.phnum as usize;

        for i in 0..ph_count {
            let offset = ph_offset + i * ph_size;
            if offset + ph_size > raw_elf.len() {
                return Err(ElfError::BufferTooSmall);
            }

            let ph = &*(raw_elf.as_ptr().add(offset) as *const Elf64ProgramHeader);
            if ph.p_type == PT_LOAD {
                let dest = ph.p_paddr as *mut u8;
                let src_offset = ph.p_offset as usize;
                let filesz = ph.p_filesz as usize;
                let memsz = ph.p_memsz as usize;

                if src_offset + filesz <= raw_elf.len() {
                    // Copy segment data from ELF image
                    core::ptr::copy_nonoverlapping(raw_elf.as_ptr().add(src_offset), dest, filesz);
                    // Zero remaining BSS memory if memsz > filesz
                    if memsz > filesz {
                        core::ptr::write_bytes(dest.add(filesz), 0, memsz - filesz);
                    }
                }
            }
        }

        Ok(())
    }
}
