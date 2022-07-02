use pbr::MultiBar;
use rand::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    let mb = MultiBar::new();
    mb.println("Your Application Header:");
    mb.println("");

    for i in 1..6 {
        let count = 100 * i;
        let mut pb = mb.create_bar(count);
        pb.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
        pb.show_message = true;
        thread::spawn(move || {
            for _ in 0..count / 20 {
                for _ in 0..20 {
                    pb.message("Waiting  : ");
                    thread::sleep(Duration::from_millis(50));
                    pb.tick();
                }
                for _ in 0..20 {
                    let n = thread_rng().gen_range(0..100);
                    pb.message("Connected: ");
                    thread::sleep(Duration::from_millis(n));
                    pb.inc();
                }
            }
            for _ in 0..20 {
                pb.message("Cleaning :");
                thread::sleep(Duration::from_millis(100));
                pb.tick();
            }
            pb.finish_print(&format!("{}: Pull complete", rand_string()));
        });
    }

    mb.println("");
    mb.println("Text lines separate between two sections: ");
    mb.println("");

    for i in 1..4 {
        let count = 100 * i;
        let mut pb = mb.create_bar(count);
        thread::spawn(move || {
            for _ in 0..count {
                pb.inc();
                let n = thread_rng().gen_range(0..100);
                thread::sleep(Duration::from_millis(n));
            }
            pb.finish();
        });
    }

    mb.listen();

    println!("\nall bars done!\n");
}

fn rand_string() -> String {
    let mut v = Vec::new();
    while v.len() < 12 {
        let b = random::<u8>();
        // [0-9a-f]
        if b > 47 && b < 58 || b > 96 && b < 103 {
            v.push(b);
        }
    }
    std::str::from_utf8(&v).unwrap().to_string()
}
