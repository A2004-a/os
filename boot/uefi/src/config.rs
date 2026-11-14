//! NovaOS Apex — Boot Configuration Parser (/novaos.cfg)

#[derive(Clone, Copy)]
pub struct BootConfig {
    pub kernel_path: [u8; 64],
    pub kernel_path_len: usize,
    pub cmdline: [u8; 128],
    pub cmdline_len: usize,
    pub timeout: u32,
}

impl BootConfig {
    pub const fn default() -> Self {
        let mut cfg = Self {
            kernel_path: [0u8; 64],
            kernel_path_len: 0,
            cmdline: [0u8; 128],
            cmdline_len: 0,
            timeout: 3,
        };

        let default_kernel = b"\\novaos-apex.elf";
        let mut i = 0;
        while i < default_kernel.len() {
            cfg.kernel_path[i] = default_kernel[i];
            i += 1;
        }
        cfg.kernel_path_len = default_kernel.len();

        let default_cmdline = b"console=com1 init=/init loglevel=debug";
        let mut j = 0;
        while j < default_cmdline.len() {
            cfg.cmdline[j] = default_cmdline[j];
            j += 1;
        }
        cfg.cmdline_len = default_cmdline.len();

        cfg
    }

    pub fn parse(bytes: &[u8]) -> Self {
        let mut cfg = Self::default();
        let mut pos = 0;

        while pos < bytes.len() {
            // Find end of line
            let start = pos;
            while pos < bytes.len() && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                pos += 1;
            }
            let line = &bytes[start..pos];

            // Advance over newline characters
            while pos < bytes.len() && (bytes[pos] == b'\n' || bytes[pos] == b'\r') {
                pos += 1;
            }

            // Skip empty lines or comment lines
            let trimmed = trim(line);
            if trimmed.is_empty() || trimmed[0] == b'#' {
                continue;
            }

            // Split on '='
            if let Some(eq_pos) = find_byte(trimmed, b'=') {
                let key = trim(&trimmed[..eq_pos]);
                let val = trim(&trimmed[eq_pos + 1..]);

                if eq_ignore_ascii_case(key, b"kernel") {
                    let len = val.len().min(63);
                    cfg.kernel_path[..len].copy_from_slice(&val[..len]);
                    cfg.kernel_path_len = len;
                } else if eq_ignore_ascii_case(key, b"cmdline") {
                    let len = val.len().min(127);
                    cfg.cmdline[..len].copy_from_slice(&val[..len]);
                    cfg.cmdline_len = len;
                } else if eq_ignore_ascii_case(key, b"timeout") {
                    cfg.timeout = parse_u32(val).unwrap_or(3);
                }
            }
        }

        cfg
    }
}

fn trim(mut s: &[u8]) -> &[u8] {
    while let Some((&first, rest)) = s.split_first() {
        if first == b' ' || first == b'\t' {
            s = rest;
        } else {
            break;
        }
    }
    while let Some((&last, rest)) = s.split_last() {
        if last == b' ' || last == b'\t' {
            s = rest;
        } else {
            break;
        }
    }
    s
}

fn find_byte(s: &[u8], target: u8) -> Option<usize> {
    for (i, &b) in s.iter().enumerate() {
        if b == target {
            return Some(i);
        }
    }
    None
}

fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i].to_ascii_lowercase() != b[i].to_ascii_lowercase() {
            return false;
        }
    }
    true
}

fn parse_u32(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut val: u32 = 0;
    for &b in s {
        if b < b'0' || b > b'9' {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}
