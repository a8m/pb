use pbr::ProgressBar;
use std::thread;
use std::time::Duration;

fn main() {
    let mut pb = ProgressBar::new(10000);
    pb.set_max_refresh_rate(Some(Duration::from_secs(1)));

    for _ in 0..10000 {
        pb.inc();
        thread::sleep(Duration::from_millis(1));
    }

    pb.finish_print("");
}
