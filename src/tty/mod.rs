//! Most of the code in for the `terminal_size()` function taken from:
//! https://github.com/eminence/terminal-size
//!
//! A simple utility for getting the size of a terminal, and moving `n` lines up.
//!
//! Supports both Linux and Windows, but help is needed to test other platforms
//!
//!

#[derive(Debug)]
pub struct Width(pub u16);
#[derive(Debug)]
pub struct Height(pub u16);

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;
