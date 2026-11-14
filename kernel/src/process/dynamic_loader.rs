//! NovaOS Apex — Dynamic Linker & ELF Shared Object Loader (M12)
//! Provides ELF64 dynamic section parsing, symbol table resolution, PLT/GOT relocations
//! (R_X86_64_RELATIVE, R_X86_64_GLOB_DAT, R_X86_64_JUMP_SLOT), and runtime shared object management.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::scheduler::sync::SpinLock;

// Dynamic Array Tags (d_tag)
pub const DT_NULL: i64 = 0;
pub const DT_NEEDED: i64 = 1;
pub const DT_PLTRELSZ: i64 = 2;
pub const DT_PLTGOT: i64 = 3;
pub const DT_HASH: i64 = 4;
pub const DT_STRTAB: i64 = 5;
pub const DT_SYMTAB: i64 = 6;
pub const DT_RELA: i64 = 7;
pub const DT_RELASZ: i64 = 8;
pub const DT_RELAENT: i64 = 9;
pub const DT_STRSZ: i64 = 10;
pub const DT_SYMENT: i64 = 11;
pub const DT_INIT: i64 = 12;
pub const DT_FINI: i64 = 13;
pub const DT_SONAME: i64 = 14;
pub const DT_RPATH: i64 = 15;
pub const DT_SYMBOLIC: i64 = 16;
pub const DT_REL: i64 = 17;
pub const DT_RELSZ: i64 = 18;
pub const DT_RELENT: i64 = 19;
pub const DT_PLTREL: i64 = 20;
pub const DT_DEBUG: i64 = 21;
pub const DT_TEXTREL: i64 = 22;
pub const DT_JMPREL: i64 = 23;

// x86_64 Relocation Types
pub const R_X86_64_NONE: u32 = 0;
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_PC32: u32 = 2;
pub const R_X86_64_GOT32: u32 = 3;
pub const R_X86_64_PLT32: u32 = 4;
pub const R_X86_64_COPY: u32 = 5;
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JUMP_SLOT: u32 = 7;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_X86_64_GOTPCREL: u32 = 9;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Dyn {
    pub d_tag: i64,
    pub d_val: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Elf64Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64,
}

impl Elf64Rela {
    pub fn r_sym(&self) -> u32 {
        (self.r_info >> 32) as u32
    }

    pub fn r_type(&self) -> u32 {
        (self.r_info & 0xFFFF_FFFF) as u32
    }

    pub fn new(offset: u64, sym: u32, r_type: u32, addend: i64) -> Self {
        Self {
            r_offset: offset,
            r_info: ((sym as u64) << 32) | (r_type as u64),
            r_addend: addend,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Elf64Sym {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

#[derive(Clone, Debug)]
pub struct ParsedDynamicSection {
    pub strtab_offset: u64,
    pub strtab_size: u64,
    pub symtab_offset: u64,
    pub rela_offset: u64,
    pub rela_size: u64,
    pub rela_ent: u64,
    pub needed_libraries: Vec<u64>,
}

impl ParsedDynamicSection {
    pub fn parse(dyn_entries: &[Elf64Dyn]) -> Self {
        let mut parsed = Self {
            strtab_offset: 0,
            strtab_size: 0,
            symtab_offset: 0,
            rela_offset: 0,
            rela_size: 0,
            rela_ent: 24, // sizeof(Elf64Rela)
            needed_libraries: Vec::new(),
        };

        for entry in dyn_entries {
            match entry.d_tag {
                DT_NULL => break,
                DT_STRTAB => parsed.strtab_offset = entry.d_val,
                DT_STRSZ => parsed.strtab_size = entry.d_val,
                DT_SYMTAB => parsed.symtab_offset = entry.d_val,
                DT_RELA => parsed.rela_offset = entry.d_val,
                DT_RELASZ => parsed.rela_size = entry.d_val,
                DT_RELAENT => parsed.rela_ent = entry.d_val,
                DT_NEEDED => parsed.needed_libraries.push(entry.d_val),
                _ => {}
            }
        }

        parsed
    }
}

pub struct SharedLibrary {
    pub id: u32,
    pub name: String,
    pub base_vaddr: u64,
    pub size: usize,
    pub symbols: Vec<(String, u64)>,
    pub dependencies: Vec<String>,
}

impl SharedLibrary {
    pub fn new(id: u32, name: &str, base_vaddr: u64, size: usize) -> Self {
        Self {
            id,
            name: String::from(name),
            base_vaddr,
            size,
            symbols: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, name: &str, offset: u64) {
        self.symbols.push((String::from(name), self.base_vaddr + offset));
    }

    pub fn lookup(&self, name: &str) -> Option<u64> {
        for (sym_name, addr) in &self.symbols {
            if sym_name == name {
                return Some(*addr);
            }
        }
        None
    }
}

pub struct DynamicLinker {
    libraries: Vec<SharedLibrary>,
    next_lib_id: u32,
}

impl DynamicLinker {
    pub const fn new() -> Self {
        Self {
            libraries: Vec::new(),
            next_lib_id: 1,
        }
    }

    pub fn register_library(&mut self, name: &str, base: u64, size: usize) -> u32 {
        let id = self.next_lib_id;
        self.next_lib_id += 1;
        let lib = SharedLibrary::new(id, name, base, size);
        self.libraries.push(lib);
        id
    }

    pub fn add_symbol_to_library(&mut self, lib_id: u32, sym: &str, offset: u64) {
        if let Some(lib) = self.libraries.iter_mut().find(|l| l.id == lib_id) {
            lib.add_symbol(sym, offset);
        }
    }

    pub fn add_dependency(&mut self, lib_id: u32, dep_name: &str) {
        if let Some(lib) = self.libraries.iter_mut().find(|l| l.id == lib_id) {
            lib.dependencies.push(String::from(dep_name));
        }
    }

    pub fn resolve_symbol(&self, name: &str) -> Option<u64> {
        for lib in &self.libraries {
            if let Some(addr) = lib.lookup(name) {
                return Some(addr);
            }
        }
        None
    }

    pub fn apply_relocation(
        &self,
        base_vaddr: u64,
        rela: &Elf64Rela,
        sym_name: Option<&str>,
        memory: &mut [u8],
    ) -> Result<u64, &'static str> {
        let rel_type = rela.r_type();
        let target_offset = rela.r_offset as usize;

        if target_offset + 8 > memory.len() {
            return Err("Relocation target exceeds memory slice boundary");
        }

        match rel_type {
            R_X86_64_NONE => Ok(0),

            // Base address + addend
            R_X86_64_RELATIVE => {
                let val = base_vaddr.wrapping_add(rela.r_addend as u64);
                let bytes = val.to_le_bytes();
                memory[target_offset..target_offset + 8].copy_from_slice(&bytes);
                Ok(val)
            }

            // Symbol value + addend (GOT entry)
            R_X86_64_GLOB_DAT => {
                let sym = sym_name.ok_or("Missing symbol for GLOB_DAT")?;
                let sym_addr = self.resolve_symbol(sym).ok_or("Unresolved symbol")?;
                let val = sym_addr.wrapping_add(rela.r_addend as u64);
                let bytes = val.to_le_bytes();
                memory[target_offset..target_offset + 8].copy_from_slice(&bytes);
                Ok(val)
            }

            // PLT jump slot: binds function entry point
            R_X86_64_JUMP_SLOT => {
                let sym = sym_name.ok_or("Missing symbol for JUMP_SLOT")?;
                let sym_addr = self.resolve_symbol(sym).ok_or("Unresolved symbol")?;
                let val = sym_addr.wrapping_add(rela.r_addend as u64);
                let bytes = val.to_le_bytes();
                memory[target_offset..target_offset + 8].copy_from_slice(&bytes);
                Ok(val)
            }

            _ => Err("Unsupported relocation type"),
        }
    }

    pub fn dlopen(&mut self, path: &str) -> Result<u64, i32> {
        for lib in &self.libraries {
            if lib.name == path {
                return Ok(lib.id as u64);
            }
        }
        // If not already loaded, register mock/virtual library for system runtime
        let id = self.register_library(path, 0x0000_7F00_0000_0000, 0x100000);
        Ok(id as u64)
    }

    pub fn dlsym(&self, handle: u64, name: &str) -> Result<u64, i32> {
        if handle == 0 {
            // Global search
            self.resolve_symbol(name).ok_or(-2) // ENOENT
        } else {
            let lib = self.libraries.iter().find(|l| l.id as u64 == handle).ok_or(-9)?; // EBADF
            lib.lookup(name).ok_or(-2)
        }
    }
}

pub static DYNAMIC_LINKER: SpinLock<DynamicLinker> = SpinLock::new("dyn_linker", DynamicLinker::new());

pub fn sys_dlopen(path_str: &str) -> Result<u64, i32> {
    let mut linker = DYNAMIC_LINKER.lock();
    linker.dlopen(path_str)
}

pub fn sys_dlsym(handle: u64, sym_name: &str) -> Result<u64, i32> {
    let linker = DYNAMIC_LINKER.lock();
    linker.dlsym(handle, sym_name)
}
