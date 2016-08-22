extern crate pbr;
use pbr::ProgressBar;
use std::thread;
use std::time::Duration;

fn main() {
    let mut pb_l0 = ProgressBar::new_level(4, 0);
    pb_l0.message("level0 ");

    for _ in 0..4 {
        pb_l0.inc();

        let mut pb_l1 = ProgressBar::new_level(4, 1);
        pb_l1.message("level1 ");

        for _ in 0..4 {
            pb_l1.inc();

            let mut pb_l2 = ProgressBar::new_level(4, 2);
            pb_l2.message("level2 ");

            for _ in 0..4 {
                pb_l2.inc();

                let mut pb_l3 = ProgressBar::new_level(4, 3);
                pb_l3.message("level3 ");

                for _ in 0..4 {
                    pb_l3.inc();

                    let mut pb_l4 = ProgressBar::new_level(4, 4);
                    pb_l4.message("level4 ");

                    for _ in 0..4 {
                        pb_l4.inc();
                        thread::sleep(Duration::from_millis(50));
                    }

                    pb_l4.finish();
                }

                pb_l3.finish();
            }

            pb_l2.finish();
        }

        pb_l1.finish();
    }

    pb_l0.finish();

    // `finish()` ends on the same line, so space out leftovers
    println!("");
    println!("");
    println!("");
    println!("");
    println!("");
}
