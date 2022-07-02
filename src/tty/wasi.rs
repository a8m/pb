use super::{Height, Width};

/// For WASI so far it will return none
///
/// For background https://github.com/WebAssembly/WASI/issues/42
pub fn terminal_size() -> Option<(Width, Height)> {
    return None;
}

/// This is inherited from unix and will work only when wasi executed on unix.
///
/// For background https://github.com/WebAssembly/WASI/issues/42
pub fn move_cursor_up(n: usize) -> String {
    format!("\x1B[{}A", n)
}
