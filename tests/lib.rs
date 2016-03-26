extern crate pbr;

use pbr::{ProgressBar, PbIter};
use std::time::Duration;
use std::thread;

#[test]
fn simple_example() {
    let count = 1000;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(1));
    }
    println!("done!");
}

#[test]
fn simple_iter_example(){
    for _ in PbIter::new(0..1000) {
        thread::sleep(Duration::from_millis(1));
    }
    println!("done!");
}
