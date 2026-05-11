/// Trim whitespace from both ends of a string.
pub fn trim(s: &str) -> &str {
    s.trim()
}

/// Convert ASCII characters to uppercase.
pub fn to_upper_ascii(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// Parse a boolean value from a string.
pub fn parse_bool(s: &str, default: bool) -> bool {
    match s.trim().to_ascii_uppercase().as_str() {
        "1" | "TRUE" | "YES" | "ON" => true,
        "0" | "FALSE" | "NO" | "OFF" => false,
        _ => default,
    }
}

/// Split a string by a delimiter.
pub fn split(s: &str, delimiter: char) -> Vec<String> {
    s.split(delimiter).map(|part| part.to_string()).collect()
}

/// Remove surrounding quotes from a string.
pub fn unquote(s: &str) -> &str {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

/// Extract the basename from a file path.
pub fn base_name(path: &str) -> &str {
    match path.rfind(&['/', '\\'][..]) {
        Some(pos) => &path[pos + 1..],
        None => path,
    }
}

/// Convert a UTF-8 string to a wide (UTF-16) null-terminated vector for Win32 APIs.
pub fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert a UTF-16 slice (without null terminator) to a Rust String.
pub fn from_wide(s: &[u16]) -> String {
    let len = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf16_lossy(&s[..len])
}
