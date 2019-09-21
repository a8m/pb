extern crate pbr;

use pbr::{PbIter, ProgressBar};
use std::thread;
use std::time::Duration;

#[test]
fn simple_example() {
    let count = 5000;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_println("done!");
}

#[test]
fn custom_width_example() {
    let count = 500;
    let mut pb = ProgressBar::new(count);
    pb.set_width(Some(80));
    pb.format("╢▌▌░╟");
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_println("done!");
}

#[test]
fn simple_iter_example() {
    for _ in PbIter::new(0..2000) {
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn timeout_example() {
    let count = 10;
    let mut pb = ProgressBar::new(count * 20);
    pb.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
    pb.show_message = true;
    pb.inc();
    for _ in 0..count {
        for _ in 0..20 {
            pb.message("Waiting  : ");
            thread::sleep(Duration::from_millis(50));
            pb.tick();
        }
        for _ in 0..20 {
            pb.message("Connected: ");
            thread::sleep(Duration::from_millis(50));
            pb.inc();
        }
    }
    for _ in 0..10 {
        pb.message("Cleaning :");
        thread::sleep(Duration::from_millis(50));
        pb.tick();
    }
    pb.finish_println("done!");
}

#[test]
// see: issue #11
fn tick_before_start() {
    let count = 100;
    let mut pb = ProgressBar::new(count);
    pb.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
    pb.tick();
    for _ in 0..count {
        pb.tick();
        thread::sleep(Duration::from_millis(50));
    }
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn npm_bar() {
    let count = 30;
    let mut pb = ProgressBar::new(count * 5);
    pb.tick_format("\\|/-");
    pb.format("|#--|");
    pb.show_tick = true;
    pb.show_speed = false;
    pb.show_percent = false;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.inc();
    for _ in 0..count {
        for _ in 0..5 {
            pb.message("normalize -> thing ");
            thread::sleep(Duration::from_millis(80));
            pb.tick();
        }
        for _ in 0..5 {
            pb.message("fuzz -> tree       ");
            thread::sleep(Duration::from_millis(80));
            pb.inc();
        }
    }
    pb.finish_println("done!");
}

#[test]
// see: issue 45#
fn final_redraw_max_refresh_rate() {
    let count = 500;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");
    pb.set_max_refresh_rate(Some(Duration::from_millis(100)));
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_println("done!");
}
