use std;
use std::ops::Add;
use std::io::{self, Write, Read};
use time::{self, Timespec, Duration};

macro_rules! printfl {
    ($($tt:tt)*) => {{
        use std::io::{self, Write};
        print!($($tt)*);
        io::stdout().flush().ok().expect("flush() fail");
    }}
}

macro_rules! kb_fmt {
    ($n: ident) => {{
        let kb = 1024f64;
        match $n {
            $n if $n > kb.powf(4_f64) => {format!("{:.*} TG", 2, $n / kb.powf(4_f64))},
            $n if $n > kb.powf(3_f64) => format!("{:.*} GB", 2, $n / kb.powf(3_f64)),
            $n if $n > kb => format!("{:.*} KB", 2, $n / kb),
            _ => format!("{:.*} B", 2, $n)
        }
    }}
}

macro_rules! repeat {
    ($s: expr, $n: expr) => {{
        &std::iter::repeat($s).take($n).collect::<String>()
    }}
}

static FORMAT: &'static str = "[=>-]";

pub struct ProgressBar {
    start_time: Timespec,
    total: usize,
    current: usize,
    is_finish: bool,
    show_bar: bool,
    show_speed: bool,
    show_percent: bool,
    show_counter: bool,
    show_time_left: bool,
    // Format
    bar_start: String,
    bar_current: String,
    bar_current_n: String,
    bar_remain: String,
    bar_end: String,
}

impl ProgressBar {
    pub fn new(total: usize) -> ProgressBar {
        let mut pb = ProgressBar {
            total: total,
            current: 0,
            start_time: time::get_time(),
            is_finish: false,
            show_bar: true,
            show_speed: false,
            show_percent: true,
            show_counter: true,
            show_time_left: true,
            bar_start: String::new(),
            bar_current: String::new(),
            bar_current_n: String::new(),
            bar_remain: String::new(),
            bar_end: String::new(),
        };
        pb.format(FORMAT);
        pb
    }

    fn format(&mut self, fmt: &str) {
        if fmt.len() >= 5 {
            let v: Vec<&str> = fmt.split("").collect();
            self.bar_start = v[1].to_string();
            self.bar_current = v[2].to_string();
            self.bar_current_n = v[3].to_string();
            self.bar_remain = v[4].to_string();
            self.bar_end = v[5].to_string();
        }
    }

    pub fn add(&mut self, i: usize) -> usize {
        self.current += i;
        if self.current <= self.total {
            self.draw()
        };
        self.current
    }

    fn draw(&self) {
        let width = 149;    // replace to -> get_tty_size()
        let mut base = String::new();
        let mut suffix = String::new();
        let mut prefix = String::new();
        let mut out;
        // precent box
        if self.show_percent {
            let percent = self.current as f64 / (self.total as f64 / 100f64);
            suffix = suffix + &format!(" {:.*} % ", 2, percent);
        }
        // speed box
        // TODO: ADD KB FORMAT
        if !self.show_speed {
            let from_start = (time::get_time() - self.start_time).num_nanoseconds().unwrap() as f64;
            let sec_nano = Duration::seconds(1).num_nanoseconds().unwrap() as f64;
            let speed = self.current as f64 / (from_start / sec_nano);
            suffix = suffix + &format!("{}/s ", speed as usize);
        }
        // time left box
        if self.show_time_left {
            let from_start = time::get_time() - self.start_time;
            let sec_nano = Duration::seconds(1).num_nanoseconds().unwrap() as i32;
            let per_entry = from_start / self.current as i32;
            let mut left = per_entry * (self.total - self.current) as i32;
            left = (left / sec_nano) * sec_nano;
            if left.num_seconds() > 0 {
                if left.num_seconds() < Duration::minutes(1).num_seconds() {
                    suffix = suffix + &format!("{}s", left.num_seconds());
                } else {
                    suffix = suffix + &format!("{}m", left.num_minutes());
                }
            }
        }
        // counter box
        if self.show_counter {
            prefix = format!("{} / {} ", self.current, self.total);
        }
        // bar box
        if self.show_bar {
            let size = width - (prefix.len() + suffix.len() + 3);
            if size > 0 {
                let curr_count =
                    ((self.current as f64 / self.total as f64) * size as f64).ceil() as usize;
                let rema_count = size - curr_count;
                base = self.bar_start.to_string();
                if rema_count > 0 {
                    base = base + repeat!(self.bar_current.as_ref(), curr_count - 1) +
                           &self.bar_current_n;
                } else {
                    base = base + repeat!(self.bar_current.as_ref(), curr_count);
                }
                base = base + repeat!(self.bar_remain.as_ref(), rema_count) + &self.bar_end;
            }
        }
        out = prefix + &base + &suffix;
        // pad
        if out.len() < width {
            let gap = width - out.len();
            out = out + repeat!(" ", gap);
        }
        // print
        printfl!("\r{}", out);
    }

    pub fn finish(&mut self) {
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
