//! In-Memory RAM Filesystem for Early Boot & Testing

pub struct RamFile {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub struct RamFs {
    files: [Option<RamFile>; 16],
    count: usize,
}

impl RamFs {
    pub const fn new() -> Self {
        const INIT_FILE: Option<RamFile> = None;
        RamFs {
            files: [INIT_FILE; 16],
            count: 0,
        }
    }

    pub fn insert(&mut self, file: RamFile) -> Result<(), &'static str> {
        if self.count >= 16 {
            return Err("RamFS full");
        }
        self.files[self.count] = Some(file);
        self.count += 1;
        Ok(())
    }

    pub fn read(&self, name: &str) -> Option<&'static [u8]> {
        for slot in self.files.iter().flatten() {
            if slot.name == name {
                return Some(slot.data);
            }
        }
        None
    }
}
