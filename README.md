# Console progress bar for Rust

Console progress bar for Rust Inspired from [pb](http://github.com/cheggaaa/pb), support and 
tested on MacOS and Linux(should work on Windows too, but not tested yet).

![Screenshot](https://github.com/a8m/pb/blob/master/gif/rec.gif)

[Documentation](http://a8m.github.io/pb/doc/pbr/index.html)

### Examples
1. simple example

```rust
extern crate pbr;

use pbr::ProgressBar;
use std::thread;

fn main() {
    let count = 1000;
    let mut pb = ProgressBar::new(count);
    for _ in 0..count {
        pb.inc();
        thread::sleep_ms(200);
    }
    println!("done!");
}
```

2. Broadcast writing(simple file copying)

```rust
#![feature(io)]
extern crate pbr;

use std::io::copy;
use std::io::prelude::*;
use std::fs::File;
use pbr::{ProgressBar, Units};

fn main() {
    let mut file = File::open("/usr/share/dict/words").unwrap();
    let n_bytes = file.metadata().unwrap().len() as usize;
    let mut pb = ProgressBar::new(n_bytes);
    pb.set_units(Units::Bytes);
    let mut handle = File::create("copy-words").unwrap().broadcast(&mut pb);
    copy(&mut file, &mut handle).unwrap();
    println!("done!");
}
```

### License
MIT

