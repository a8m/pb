use pbr::ProgressBar;
use std::thread;
use std::time::Duration;

fn main() {
    let count = 30;
    let mut pb = ProgressBar::new(count * 10);
    pb.tick_format("\\|/-");
    pb.format("|#--|");
    pb.show_tick = true;
    pb.show_speed = false;
    pb.show_percent = false;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.inc();
    for _ in 0..count {
        for _ in 0..10 {
            pb.message("normalize -> thing ");
            thread::sleep(Duration::from_millis(80));
            pb.tick();
        }
        for _ in 0..10 {
            pb.message("fuzz -> tree       ");
            thread::sleep(Duration::from_millis(80));
            pb.inc();
        }
    }
    pb.finish_println("done!");
}
