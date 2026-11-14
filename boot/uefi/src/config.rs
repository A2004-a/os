//! NovaOS Zero — Freestanding Boot Configuration Parser (novaos.cfg)

pub struct BootConfig {
    pub timeout_seconds: u32,
    pub kernel_path: &'static str,
    pub command_line: &'static str,
}

impl BootConfig {
    pub const fn default() -> Self {
        Self {
            timeout_seconds: 3,
            kernel_path: "novaos.elf",
            command_line: "console=serial,gop loglevel=debug",
        }
    }

    /// Parse a simple key=value ASCII configuration string
    pub fn parse(content: &str) -> Self {
        let mut cfg = Self::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');

                match key {
                    "timeout" => {
                        let mut num: u32 = 0;
                        for b in val.bytes() {
                            if b >= b'0' && b <= b'9' {
                                num = num * 10 + (b - b'0') as u32;
                            }
                        }
                        cfg.timeout_seconds = num;
                    }
                    "kernel" => {
                        // Static path buffer
                        static mut KERNEL_BUF: [u8; 64] = [0; 64];
                        unsafe {
                            let len = val.len().min(63);
                            KERNEL_BUF[..len].copy_from_slice(&val.as_bytes()[..len]);
                            KERNEL_BUF[len] = 0;
                            if let Ok(s) = core::str::from_utf8(&KERNEL_BUF[..len]) {
                                cfg.kernel_path = s;
                            }
                        }
                    }
                    "cmdline" => {
                        static mut CMDLINE_BUF: [u8; 256] = [0; 256];
                        unsafe {
                            let len = val.len().min(255);
                            CMDLINE_BUF[..len].copy_from_slice(&val.as_bytes()[..len]);
                            CMDLINE_BUF[len] = 0;
                            if let Ok(s) = core::str::from_utf8(&CMDLINE_BUF[..len]) {
                                cfg.command_line = s;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        cfg
    }
}
