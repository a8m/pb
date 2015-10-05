extern crate time;
use time::{Timespec, Duration};
use std::thread;
use std::ops::Add;
use std::io::{self, Write, Read};

macro_rules! printfl {
    ($($tt:tt)*) => {{
        use std::io::{self, Write};
        print!($($tt)*);
        io::stdout().flush().ok().expect("flush() fail");
    }}
}

pub struct ProgressBar {
    start_time: Timespec,
    total: usize,
    current: usize,
    is_finish: bool,
}

impl ProgressBar {
    pub fn new(total: usize) -> ProgressBar {
            ProgressBar {
                total: total,
                current: 0,
                start_time: time::get_time(),
                is_finish: false,
            }
    }    

    fn add(&mut self, i: usize) -> usize {
        self.current += i;
        if self.current <= self.total {
            self.draw()
        };
        self.current
    }

    fn draw(&self) {
        let width = 143;    // replace to -> get_tty_size()
        let percent_box;
        let counter_box;
        let mut time_left_box = format!("");
        let speed_box;
        let mut bar_box = "[".to_string();
        let mut out;
        // precent box
        let percent = self.current as f64 / (self.total as f64 / 100f64);
        percent_box = format!(" {:.*} % ", 2, percent);
        // counter box
        counter_box = format!("{} / {} ", self.current, self.total);
        // time left box
        let from_start = time::get_time() - self.start_time;
        let per_entry = from_start / self.current as i32; // Why the hack
        let mut left = per_entry * (self.total - self.current) as i32;
        let sec_nano = Duration::seconds(1).num_nanoseconds().unwrap() as i32;
        left = (left / sec_nano) * sec_nano;
        if left.num_seconds() > 0 {
            time_left_box = format!("{}s", left.num_seconds());
        }
        // NOT WORKING: speed box
        let speed = (from_start / sec_nano) / self.current as i32;
        speed_box = format!("{}/s", speed.num_nanoseconds().unwrap() as f64);
        // bar_box - Add prefix & suffix(2)
        let size = width -
                   (percent_box.to_string() + &counter_box + &time_left_box + &speed_box).len();
        // Test if size > 0
        let curr_count = ((self.current as f64 / self.total as f64) * size as f64).ceil();
        let err_count = size as f64 - curr_count;
        bar_box = bar_box + &std::iter::repeat("=").take(curr_count as usize).collect::<String>();
        if self.current < self.total {
            bar_box = bar_box + ">";
        } else {
            bar_box = bar_box + "=";
        }
        bar_box = bar_box + &std::iter::repeat("-").take(err_count as usize).collect::<String>() +
                  "]";
        out = counter_box.to_string() + &bar_box + &percent_box + &time_left_box;
        // Print
        if out.len() < width {
            let gap = width - out.len();
            out = out + &std::iter::repeat(" ").take(gap as usize).collect::<String>();
        }
        printfl!("\r{}", out);
    }
    
    fn finish(&mut self) {
        if self.current < self.total {
            self.current = self.total;
            self.draw();
        }
        println!("");
        self.is_finish = true;
    }
}

// Implement io::Writer
impl Write for ProgressBar {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { 
        let n = buf.len();
        self.add(n);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { 
        Ok(()) 
    }
}

// TODO: Implement io::Reader

fn main() {
    let mut pb = ProgressBar::new(1000);
    for _ in 0..1000 {
        pb.add(1);
        thread::sleep_ms(2);
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
