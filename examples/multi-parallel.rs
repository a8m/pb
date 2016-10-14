extern crate pbr;
use pbr::ProgressBar;
use std::thread;
use std::time::Duration;

/// Test for https://github.com/a8m/pb/pull/27#issuecomment-244564706
///
/// Because on non-Windows all cursor ops are handled by ANSI escapes
/// and `stdout()` is internally synchronised everything works perfectly there.
///
/// On Windows, however, where you use WinAPI (and, therefore, can *not* synchronise with `stdout()`)
/// we simply cannot make multiple progress bars on multiple threads work and the end-user needs to manually synchronise
/// `draw()` calls itself (be it via `Mutex` or `RwLock`, for example).
fn main() {
    let mut pb0 = ProgressBar::new_level(1000, 0);
    let mut pb1 = ProgressBar::new_level(2000, 1);

    let child0 = thread::spawn(move || {
        for _ in 0..1000 {
            pb0.inc();
            thread::sleep(Duration::from_millis(11));
        }
    });

    let child1 = thread::spawn(move || {
        for _ in 0..2000 {
            pb1.inc();
            thread::sleep(Duration::from_millis(13));
        }
    });

    let _ = child0.join();
    let _ = child1.join();

    println!("");
    println!("");
}
