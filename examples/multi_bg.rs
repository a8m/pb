use pbr::MultiBar;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

fn main() {
    let complete = Arc::new(AtomicBool::new(false));
    let progress = Arc::new(MultiBar::new());

    thread::spawn({
        let complete = Arc::clone(&complete);
        let progress = Arc::clone(&progress);
        move || {
            for task in 1..=10 {
                thread::spawn({
                    let progress = Arc::clone(&progress);
                    move || {
                        let mut bar = progress.create_bar(100);
                        bar.message(&format!("Task {}: ", task));

                        for _ in 0..100 {
                            thread::sleep(Duration::from_millis(50));
                            bar.inc();
                        }

                        bar.finish_print(&format!("Task {} Complete", task));
                    }
                });

                thread::sleep(Duration::from_millis(1000));
            }

            complete.store(true, Ordering::SeqCst);
        }
    });

    while !complete.load(Ordering::SeqCst) {
        let _ = progress.listen();
        thread::sleep(Duration::from_millis(1000));
    }

    let _ = progress.listen();
}
