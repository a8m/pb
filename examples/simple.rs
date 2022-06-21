use pbr::ProgressBar;
use rand::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    let count = 500;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");
    for _ in 0..count {
        pb.inc();
        let n = thread_rng().gen_range(0..100);
        thread::sleep(Duration::from_millis(n));
    }
    pb.finish_println("done!");
}
