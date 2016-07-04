//! # Console progress bar for Rust
//!
//! Console progress bar for Rust Inspired from [pb](http://github.com/cheggaaa/pb).
//! support and tested on MacOS and Linux(should work on Windows too, but not tested yet).
//!
//! ![Screenshot](https://raw.githubusercontent.com/a8m/pb/master/gif/rec_v2.gif)
//!
//! ### Examples
//! 1. simple example
//!
//! ```no_run
//! extern crate pbr;
//!
//! use pbr::ProgressBar;
//! use std::thread;
//!
//! fn main() {
//!     let count = 1000;
//!     let mut pb = ProgressBar::new(count);
//!     for _ in 0..count {
//!         pb.inc();
//!         thread::sleep_ms(200);
//!     }
//!     println!("done!");
//! }
//! ```
//!
//! 2. Broadcast writing(simple file copying)
//!
//! ```ignore
//! #![feature(io)]
//! extern crate pbr;
//!
//! use std::io::copy;
//! use std::io::prelude::*;
//! use std::fs::File;
//! use pbr::{ProgressBar, Units};
//!
//! fn main() {
//!     let mut file = File::open("/usr/share/dict/words").unwrap();
//!     let n_bytes = file.metadata().unwrap().len() as usize;
//!     let mut pb = ProgressBar::new(n_bytes);
//!     pb.set_units(Units::Bytes);
//!     let mut handle = File::create("copy-words").unwrap().broadcast(&mut pb);
//!     copy(&mut file, &mut handle).unwrap();
//!     println!("done!");
//! }
//! ```
extern crate time;
mod tty;
mod pb;
pub use pb::{ProgressBar, Units};
use std::io::Write;

pub struct PbIter<T, I>
    where I: Iterator,
          T: Write
{
    iter: I,
    progress_bar: ProgressBar<T>,
}

impl<T, I> PbIter<T, I>
    where I: Iterator,
          T: Write
{
    pub fn new(handle: T, iter: I) -> Self {
        let size = iter.size_hint().0;
        let pb: PbIter<T, I> = PbIter {iter: iter, progress_bar: ProgressBar::new(handle, size as u64)};
        pb
    }
}

impl<T, I> Iterator for PbIter<T, I>
    where I: Iterator,
    T: Write
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        match self.iter.next() {
            Some(i) => {
                self.progress_bar.inc();
                Some(i)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
