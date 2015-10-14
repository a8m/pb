//! Console progress bar for Rust Inspired from github.com/cheggaaa/pb
//!
//! Support and tested on MacOS, Linux, Windows will be some day.
//!
extern crate time;
mod tty;
mod pb;
pub use pb::{ProgressBar, Units};
