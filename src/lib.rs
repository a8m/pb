//! # Terminal progress bar for Rust
//!
//! Console progress bar for Rust Inspired from [pb](http://github.com/cheggaaa/pb), support and
//! tested on MacOS, Linux and Windows
//!
//! ![Screenshot](https://raw.githubusercontent.com/a8m/pb/master/gif/rec_v3.gif)
//!
//! [Documentation](http://a8m.github.io/pb/doc/pbr/index.html)
//!
//! ### Examples
//! 1. simple example
//!
//! ```ignore
//! use pbr::ProgressBar;
//! use std::thread;
//!
//! fn main() {
//!     let count = 1000;
//!     let mut pb = ProgressBar::new(count);
//!     pb.format("╢▌▌░╟");
//!     for _ in 0..count {
//!         pb.inc();
//!         thread::sleep_ms(200);
//!     }
//!     pb.finish_print("done");
//! }
//! ```
//!
//! 2. MultiBar example. see full example [here](https://github.com/a8m/pb/blob/master/examples/multi.rs)
//!
//! ```ignore
//! use std::thread;
//! use pbr::MultiBar;
//! use std::time::Duration;
//!
//! fn main() {
//!     let mut mb = MultiBar::new();
//!     let count = 100;
//!     mb.println("Application header:");
//!
//!     let mut p1 = mb.create_bar(count);
//!     let _ = thread::spawn(move || {
//!         for _ in 0..count {
//!             p1.inc();
//!             thread::sleep(Duration::from_millis(100));
//!         }
//!         // notify the multibar that this bar finished.
//!         p1.finish();
//!     });
//!
//!     mb.println("add a separator between the two bars");
//!
//!     let mut p2 = mb.create_bar(count * 2);
//!     let _ = thread::spawn(move || {
//!         for _ in 0..count * 2 {
//!             p2.inc();
//!             thread::sleep(Duration::from_millis(100));
//!         }
//!         // notify the multibar that this bar finished.
//!         p2.finish();
//!     });
//!
//!     // start listen to all bars changes.
//!     // this is a blocking operation, until all bars will finish.
//!     // to ignore blocking, you can run it in a different thread.
//!     mb.listen();
//! }
//! ```
//!
//! 3. Broadcast writing(simple file copying)
//!
//! ```ignore
//! #![feature(io)]
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
//!     pb.finish_print("done");
//! }
//! ```

// Macro for writing to the giving writer.
// Used in both pb.rs and multi.rs modules.
//
// # Examples
//
// ```
// let w = io::stdout();
// printfl!(w, "");
// printfl!(w, "\r{}", out);
//
// ```
macro_rules! printfl {
   ($w:expr, $($tt:tt)*) => {{
        $w.write_all(&format!($($tt)*).as_bytes()).ok().expect("write() fail");
        $w.flush().ok().expect("flush() fail");
    }}
}

mod multi;
mod pb;
mod tty;
pub use multi::{MultiBar, Pipe};
pub use pb::{ProgressBar, Units};
use std::io::{stdout, Stdout, Write};

pub struct PbIter<T, I>
where
    I: Iterator,
    T: Write,
{
    iter: I,
    progress_bar: ProgressBar<T>,
}

impl<I> PbIter<Stdout, I>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        Self::on(stdout(), iter)
    }
}

impl<T, I> PbIter<T, I>
where
    I: Iterator,
    T: Write,
{
    pub fn on(handle: T, iter: I) -> Self {
        let size = iter.size_hint().0;
        PbIter {
            iter,
            progress_bar: ProgressBar::on(handle, size as u64),
        }
    }
}

impl<T, I> Iterator for PbIter<T, I>
where
    I: Iterator,
    T: Write,
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
