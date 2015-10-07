extern crate time;
mod pb;
use std::thread;
use pb::ProgressBar;

fn main() {
    let count = 1000;
    let mut pb = ProgressBar::new(count);
    for _ in 0..count {
        pb.add(1);
        thread::sleep_ms(3);
    }
    pb.finish();
    print!("The end!");


    /*let name = "/usr/share/dict/words";
    let mut file = std::fs::File::open(name).unwrap();
    let bytes = std::fs::metadata(name).unwrap().len() as i64;
    let mut pb = ProgressBar::new(bytes);
    std::io::copy(&mut file, &mut pb).unwrap();
    println!("Done");
    // Create example that use multiWriter and decorateWriter example too
    */
}
