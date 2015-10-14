//! Most of the code in this module(tty) taken from:
//! https://github.com/eminence/terminal-size
//!
//! A simple utility for getting the size of a terminal.
//!
//! Supports both Linux and Windows, but help is needed to test other platforms
//!
//! # Example
//!
//! ```
//! use terminal_size::{Width, Height, terminal_size};
//!
//! let size = terminal_size();
//! if let Some((Width(w), Height(h))) = size {
//!     println!("Your terminal is {} cols wide and {} lines tall", w, h);
//! } else {
//!     println!("Unable to get terminal size");
//! }
//! ```
//!

#[derive(Debug)]
pub struct Width(pub u16);
#[derive(Debug)]
pub struct Height(pub u16);

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::terminal_size;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::terminal_size;
