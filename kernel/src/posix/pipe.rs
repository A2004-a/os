//! NovaOS Apex — POSIX Anonymous Pipe Subsystem (M13)
//! Provides unidirectional byte streams with atomic circular buffering,
//! reader/writer reference counting, EOF semantics, and EPIPE signal generation.

extern crate alloc;
use crate::scheduler::sync::SpinLock;

pub const PIPE_BUFFER_SIZE: usize = 4096;
pub const MAX_PIPES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeError {
    NoFreePipes,
    NotFound,
    BrokenPipe, // EPIPE (-32)
    BufferFull,
    InvalidHandle,
}

pub struct Pipe {
    id: u32,
    buffer: [u8; PIPE_BUFFER_SIZE],
    head: usize, // write index
    tail: usize, // read index
    len: usize,
    readers: usize,
    writers: usize,
    active: bool,
}

impl Pipe {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            buffer: [0u8; PIPE_BUFFER_SIZE],
            head: 0,
            tail: 0,
            len: 0,
            readers: 0,
            writers: 0,
            active: false,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, i32> {
        if self.readers == 0 {
            return Err(-32); // EPIPE: Broken pipe
        }
        if data.is_empty() {
            return Ok(0);
        }

        let available = PIPE_BUFFER_SIZE - self.len;
        if available == 0 {
            return Ok(0); // Non-blocking pipe full
        }

        let write_len = data.len().min(available);
        for i in 0..write_len {
            self.buffer[self.head] = data[i];
            self.head = (self.head + 1) % PIPE_BUFFER_SIZE;
        }
        self.len += write_len;
        Ok(write_len)
    }

    pub fn read(&mut self, out_buf: &mut [u8]) -> Result<usize, i32> {
        if out_buf.is_empty() {
            return Ok(0);
        }

        if self.len == 0 {
            if self.writers == 0 {
                return Ok(0); // EOF: all writers closed
            }
            return Ok(0); // Non-blocking empty
        }

        let read_len = out_buf.len().min(self.len);
        for i in 0..read_len {
            out_buf[i] = self.buffer[self.tail];
            self.tail = (self.tail + 1) % PIPE_BUFFER_SIZE;
        }
        self.len -= read_len;
        Ok(read_len)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub struct PipeManager {
    pipes: [Pipe; MAX_PIPES],
    next_id: u32,
}

impl PipeManager {
    pub const fn new() -> Self {
        Self {
            pipes: [const { Pipe::empty() }; MAX_PIPES],
            next_id: 1,
        }
    }

    pub fn create_pipe(&mut self) -> Result<(u32, u32), i32> {
        let free_idx = self.pipes.iter().position(|p| !p.active).ok_or(-24)?; // EMFILE

        let id = self.next_id;
        self.next_id += 1;

        let pipe = &mut self.pipes[free_idx];
        pipe.id = id;
        pipe.head = 0;
        pipe.tail = 0;
        pipe.len = 0;
        pipe.readers = 1;
        pipe.writers = 1;
        pipe.active = true;

        // Returns (read_end_handle, write_end_handle)
        // High bit 0x8000_0000 denotes write end in composite handle
        let read_handle = id;
        let write_handle = id | 0x8000_0000;
        Ok((read_handle, write_handle))
    }

    pub fn write_pipe(&mut self, handle: u32, data: &[u8]) -> Result<usize, i32> {
        let id = handle & 0x7FFF_FFFF;
        let is_write_end = (handle & 0x8000_0000) != 0;
        if !is_write_end {
            return Err(-9); // EBADF: cannot write to read end
        }

        let pipe = self.pipes.iter_mut().find(|p| p.active && p.id == id).ok_or(-9)?;
        pipe.write(data)
    }

    pub fn read_pipe(&mut self, handle: u32, out: &mut [u8]) -> Result<usize, i32> {
        let id = handle & 0x7FFF_FFFF;
        let is_write_end = (handle & 0x8000_0000) != 0;
        if is_write_end {
            return Err(-9); // EBADF: cannot read from write end
        }

        let pipe = self.pipes.iter_mut().find(|p| p.active && p.id == id).ok_or(-9)?;
        pipe.read(out)
    }

    pub fn close_end(&mut self, handle: u32) -> Result<(), i32> {
        let id = handle & 0x7FFF_FFFF;
        let is_write_end = (handle & 0x8000_0000) != 0;

        let pipe = self.pipes.iter_mut().find(|p| p.active && p.id == id).ok_or(-9)?;
        if is_write_end {
            if pipe.writers > 0 {
                pipe.writers -= 1;
            }
        } else {
            if pipe.readers > 0 {
                pipe.readers -= 1;
            }
        }

        if pipe.readers == 0 && pipe.writers == 0 {
            pipe.active = false;
        }
        Ok(())
    }
}

pub static PIPE_MANAGER: SpinLock<PipeManager> = SpinLock::new("pipe_manager", PipeManager::new());

pub fn sys_pipe() -> Result<(u32, u32), i32> {
    PIPE_MANAGER.lock().create_pipe()
}

pub fn sys_pipe_write(handle: u32, data: &[u8]) -> Result<usize, i32> {
    PIPE_MANAGER.lock().write_pipe(handle, data)
}

pub fn sys_pipe_read(handle: u32, out: &mut [u8]) -> Result<usize, i32> {
    PIPE_MANAGER.lock().read_pipe(handle, out)
}

pub fn sys_pipe_close(handle: u32) -> Result<(), i32> {
    PIPE_MANAGER.lock().close_end(handle)
}

