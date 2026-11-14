//! NovaOS Apex — VFS Path Normalization and Utilities
//! Canonicalizes absolute and relative filesystem paths, resolves `.` and `..`, and splits segments.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Normalizes a path string, resolving `.` and `..` components.
/// If `path` is relative, it is resolved against `cwd`.
pub fn normalize_path(cwd: &str, path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();

    let combined = if path.starts_with('/') {
        path
    } else {
        // We will push cwd segments first
        for part in cwd.split('/') {
            if !part.is_empty() && part != "." {
                if part == ".." {
                    segments.pop();
                } else {
                    segments.push(part);
                }
            }
        }
        path
    };

    for part in combined.split('/') {
        if part.is_empty() || part == "." {
            continue;
        } else if part == ".." {
            segments.pop();
        } else {
            segments.push(part);
        }
    }

    if segments.is_empty() {
        String::from("/")
    } else {
        let mut result = String::new();
        for seg in segments {
            result.push('/');
            result.push_str(seg);
        }
        result
    }
}

/// Splits a normalized absolute path into its directory name and base name.
/// Example: "/etc/os-release" -> ("/etc", "os-release")
/// Example: "/etc" -> ("/", "etc")
/// Example: "/" -> ("/", "")
pub fn split_path(path: &str) -> (&str, &str) {
    if path == "/" {
        return ("/", "");
    }

    let trimmed = path.trim_end_matches('/');
    if let Some(idx) = trimmed.rfind('/') {
        if idx == 0 {
            ("/", &trimmed[1..])
        } else {
            (&trimmed[..idx], &trimmed[idx + 1..])
        }
    } else {
        ("/", trimmed)
    }
}
