//! NovaOS Apex — WebAssembly (WASM) Sandboxed Execution Engine (M14)
//! Provides a secure, fuel-metered WebAssembly MVP stack virtual machine with
//! 64 KiB linear memory bounds isolation, LEB128 decoding, and opcode dispatch.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D]; // \0asm
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00]; // version 1
pub const WASM_PAGE_SIZE: usize = 65536; // 64 KiB

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmTrap {
    GasExhausted,
    OutOfBoundsMemory,
    StackUnderflow,
    DivisionByZero,
    InvalidBytecode,
    InvalidMagic,
    InvalidVersion,
    UnresolvedExport,
}

pub struct WasmMemory {
    pub pages: usize,
    pub data: Vec<u8>,
}

impl WasmMemory {
    pub fn new(pages: usize) -> Self {
        let size = pages * WASM_PAGE_SIZE;
        Self {
            pages,
            data: vec![0u8; size],
        }
    }

    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    pub fn read_i32(&self, offset: u32) -> Result<i32, WasmTrap> {
        let off = offset as usize;
        if off + 4 > self.data.len() {
            return Err(WasmTrap::OutOfBoundsMemory);
        }
        let b = [
            self.data[off],
            self.data[off + 1],
            self.data[off + 2],
            self.data[off + 3],
        ];
        Ok(i32::from_le_bytes(b))
    }

    pub fn write_i32(&mut self, offset: u32, val: i32) -> Result<(), WasmTrap> {
        let off = offset as usize;
        if off + 4 > self.data.len() {
            return Err(WasmTrap::OutOfBoundsMemory);
        }
        let b = val.to_le_bytes();
        self.data[off..off + 4].copy_from_slice(&b);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WasmExport {
    pub name: String,
    pub kind: u8, // 0 = function, 2 = memory
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WasmFunction {
    pub locals_count: u32,
    pub code: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WasmModule {
    pub exports: Vec<WasmExport>,
    pub functions: Vec<WasmFunction>,
    pub memory_pages: usize,
}

impl WasmModule {
    pub fn parse(bytes: &[u8]) -> Result<Self, WasmTrap> {
        if bytes.len() < 8 {
            return Err(WasmTrap::InvalidBytecode);
        }
        if bytes[..4] != WASM_MAGIC {
            return Err(WasmTrap::InvalidMagic);
        }
        if bytes[4..8] != WASM_VERSION {
            return Err(WasmTrap::InvalidVersion);
        }

        let mut offset = 8;
        let mut exports = Vec::new();
        let mut functions = Vec::new();
        let mut memory_pages = 1;

        while offset < bytes.len() {
            let section_id = bytes[offset];
            offset += 1;
            let (section_len, len_bytes) = decode_u32_leb128(&bytes[offset..])?;
            offset += len_bytes;
            let section_end = offset + section_len as usize;

            if section_end > bytes.len() {
                return Err(WasmTrap::InvalidBytecode);
            }

            let sec_data = &bytes[offset..section_end];
            offset = section_end;

            match section_id {
                5 => {
                    // Memory section
                    if !sec_data.is_empty() {
                        let (_, n_bytes) = decode_u32_leb128(sec_data)?;
                        if n_bytes < sec_data.len() {
                            let (pages, _) = decode_u32_leb128(&sec_data[n_bytes + 1..])?;
                            memory_pages = (pages as usize).max(1);
                        }
                    }
                }
                7 => {
                    // Export section
                    let mut s_off = 0;
                    if let Ok((count, c_bytes)) = decode_u32_leb128(sec_data) {
                        s_off += c_bytes;
                        for _ in 0..count {
                            if s_off >= sec_data.len() {
                                break;
                            }
                            if let Ok((name_len, nl_bytes)) = decode_u32_leb128(&sec_data[s_off..]) {
                                s_off += nl_bytes;
                                let nl = name_len as usize;
                                if s_off + nl <= sec_data.len() {
                                    let name_str = core::str::from_utf8(&sec_data[s_off..s_off + nl])
                                        .unwrap_or("exported");
                                    s_off += nl;
                                    if s_off < sec_data.len() {
                                        let kind = sec_data[s_off];
                                        s_off += 1;
                                        if let Ok((index, idx_bytes)) = decode_u32_leb128(&sec_data[s_off..]) {
                                            s_off += idx_bytes;
                                            exports.push(WasmExport {
                                                name: String::from(name_str),
                                                kind,
                                                index,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                10 => {
                    // Code section
                    let mut s_off = 0;
                    if let Ok((func_count, c_bytes)) = decode_u32_leb128(sec_data) {
                        s_off += c_bytes;
                        for _ in 0..func_count {
                            if s_off >= sec_data.len() {
                                break;
                            }
                            if let Ok((body_size, b_bytes)) = decode_u32_leb128(&sec_data[s_off..]) {
                                s_off += b_bytes;
                                let sz = body_size as usize;
                                if s_off + sz <= sec_data.len() {
                                    let body = sec_data[s_off..s_off + sz].to_vec();
                                    s_off += sz;
                                    functions.push(WasmFunction {
                                        locals_count: 0,
                                        code: body,
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {} // Other sections skipped safely in MVP
            }
        }

        Ok(Self {
            exports,
            functions,
            memory_pages,
        })
    }
}

pub struct WasmVm {
    pub stack: Vec<i64>,
    pub memory: WasmMemory,
    pub gas_remaining: usize,
}

impl WasmVm {
    pub fn new(memory_pages: usize, gas_limit: usize) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            memory: WasmMemory::new(memory_pages),
            gas_remaining: gas_limit,
        }
    }

    pub fn execute(&mut self, bytecode: &[u8], initial_locals: &[i64]) -> Result<i64, WasmTrap> {
        let mut pc = 0;
        let mut locals = initial_locals.to_vec();
        // Ensure at least 8 local slots
        while locals.len() < 8 {
            locals.push(0);
        }

        while pc < bytecode.len() {
            if self.gas_remaining == 0 {
                return Err(WasmTrap::GasExhausted);
            }
            self.gas_remaining -= 1;

            let op = bytecode[pc];
            pc += 1;

            match op {
                0x01 => {
                    // nop
                }
                0x0B => {
                    // end
                    break;
                }
                0x0F => {
                    // return
                    break;
                }
                0x20 => {
                    // local.get
                    let (idx, n) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n;
                    let val = locals.get(idx as usize).copied().unwrap_or(0);
                    self.stack.push(val);
                }
                0x21 => {
                    // local.set
                    let (idx, n) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n;
                    let val = self.stack.pop().ok_or(WasmTrap::StackUnderflow)?;
                    let i = idx as usize;
                    if i < locals.len() {
                        locals[i] = val;
                    }
                }
                0x41 => {
                    // i32.const
                    let (val, n) = decode_i32_leb128(&bytecode[pc..])?;
                    pc += n;
                    self.stack.push(val as i64);
                }
                0x42 => {
                    // i64.const
                    let (val, n) = decode_i32_leb128(&bytecode[pc..])?;
                    pc += n;
                    self.stack.push(val as i64);
                }
                0x45 => {
                    // i32.eqz
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    self.stack.push(if a == 0 { 1 } else { 0 });
                }
                0x46 => {
                    // i32.eq
                    let b = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    self.stack.push(if a == b { 1 } else { 0 });
                }
                0x6A => {
                    // i32.add
                    let b = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    self.stack.push((a.wrapping_add(b)) as i64);
                }
                0x6B => {
                    // i32.sub
                    let b = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    self.stack.push((a.wrapping_sub(b)) as i64);
                }
                0x6C => {
                    // i32.mul
                    let b = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    self.stack.push((a.wrapping_mul(b)) as i64);
                }
                0x6D => {
                    // i32.div_s
                    let b = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let a = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    if b == 0 {
                        return Err(WasmTrap::DivisionByZero);
                    }
                    self.stack.push((a / b) as i64);
                }
                0x28 => {
                    // i32.load (align, offset)
                    let (_align, n1) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n1;
                    let (offset, n2) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n2;
                    let base = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as u32;
                    let effective_addr = base.wrapping_add(offset);
                    let val = self.memory.read_i32(effective_addr)?;
                    self.stack.push(val as i64);
                }
                0x36 => {
                    // i32.store (align, offset)
                    let (_align, n1) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n1;
                    let (offset, n2) = decode_u32_leb128(&bytecode[pc..])?;
                    pc += n2;
                    let val = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as i32;
                    let base = self.stack.pop().ok_or(WasmTrap::StackUnderflow)? as u32;
                    let effective_addr = base.wrapping_add(offset);
                    self.memory.write_i32(effective_addr, val)?;
                }
                _ => return Err(WasmTrap::InvalidBytecode),
            }
        }

        Ok(self.stack.pop().unwrap_or(0))
    }
}

pub fn decode_u32_leb128(bytes: &[u8]) -> Result<(u32, usize), WasmTrap> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut count = 0;

    for &b in bytes {
        count += 1;
        result |= ((b & 0x7F) as u32) << shift;
        if (b & 0x80) == 0 {
            return Ok((result, count));
        }
        shift += 7;
        if shift >= 35 {
            return Err(WasmTrap::InvalidBytecode);
        }
    }
    Err(WasmTrap::InvalidBytecode)
}

pub fn decode_i32_leb128(bytes: &[u8]) -> Result<(i32, usize), WasmTrap> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut count = 0;

    for &b in bytes {
        count += 1;
        result |= ((b & 0x7F) as i32) << shift;
        shift += 7;
        if (b & 0x80) == 0 {
            if shift < 32 && (b & 0x40) != 0 {
                result |= !0 << shift;
            }
            return Ok((result, count));
        }
        if shift >= 35 {
            return Err(WasmTrap::InvalidBytecode);
        }
    }
    Err(WasmTrap::InvalidBytecode)
}
