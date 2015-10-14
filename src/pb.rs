use std::iter::{repeat};
use std::io::{self, Write};
use time::{self, Timespec, Duration};
use tty::{Width, terminal_size};

macro_rules! printfl {
    ($($tt:tt)*) => {{
        print!($($tt)*);
        io::stdout().flush().ok().expect("flush() fail");
    }}
}

macro_rules! kb_fmt {
    ($n: ident) => {{
        let kb = 1024f64;
        match $n {
            $n if $n >= kb.powf(4_f64) => format!("{:.*} TB", 2, $n / kb.powf(4_f64)),
            $n if $n >= kb.powf(3_f64) => format!("{:.*} GB", 2, $n / kb.powf(3_f64)),
            $n if $n >= kb.powf(2_f64) => format!("{:.*} MB", 2, $n / kb.powf(2_f64)),
            $n if $n >= kb => format!("{:.*} KB", 2, $n / kb),
            _ => format!("{:.*} B", 0, $n)
        }
    }}
}

macro_rules! repeat {
    ($s: expr, $n: expr) => {{
        &repeat($s).take($n).collect::<String>()
    }}
}

static FORMAT: &'static str = "[=>-]";

// Output type format, indicate which format wil be used in
// the speed box.
#[derive(Debug)]
pub enum Units {
    Default,
    Bytes,
}

#[derive(Debug)]
pub struct ProgressBar {
    start_time: Timespec,
    units: Units,
    total: usize,
    current: usize,
    is_finish: bool,
    show_bar: bool,
    show_speed: bool,
    show_percent: bool,
    show_counter: bool,
    show_time_left: bool,
    bar_start: String,
    bar_current: String,
    bar_current_n: String,
    bar_remain: String,
    bar_end: String,
}

impl ProgressBar {
    /// Create a new ProgressBar with default configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread;
    /// use pb::{ProgressBar, Units};
    ///
    /// let count = 1000;
    /// let mut pb = ProgressBar::new(count);
    /// pb.set_units(Units::Bytes);
    ///
    /// for _ in 0..count {
    ///    pb.inc();
    ///    thread::sleep_ms(100);
    /// }
    /// ```
    pub fn new(total: usize) -> ProgressBar {
        let mut pb = ProgressBar {
            total: total,
            current: 0,
            start_time: time::get_time(),
            units: Units::Default,
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

    /// set units, default is simple numbers
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pb::{ProgressBar, Units};
    ///
    /// let mut pb = ProgressBar::new(n_bytes);
    /// pb.set_units(Units::Bytes);
    /// ```
    pub fn set_units(&mut self, u: Units) {
        self.units = u;
    }

    /// Set custom format to the drawing bar, default is "[=>-]"
    ///
    /// # Examples
    ///
    /// ```no_run
    /// pb.format("[=>_]")
    /// ```
    pub fn format(&mut self, fmt: &str) {
        if fmt.len() >= 5 {
            let v: Vec<&str> = fmt.split("").collect();
            self.bar_start = v[1].to_string();
            self.bar_current = v[2].to_string();
            self.bar_current_n = v[3].to_string();
            self.bar_remain = v[4].to_string();
            self.bar_end = v[5].to_string();
        }
    }

    /// Add to current value
    pub fn add(&mut self, i: usize) -> usize {
        self.current += i;
        if self.current <= self.total {
            self.draw()
        };
        self.current
    }

    /// Increment current value
    pub fn inc(&mut self) -> usize {
        return self.add(1);
    }

    fn draw(&self) {
        let tty_size = terminal_size();
        let width = if let Some((Width(w), _)) = tty_size {
            w as usize
        } else {
            80
        };
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
        if !self.show_speed {
            let from_start = (time::get_time() - self.start_time).num_nanoseconds().unwrap() as f64;
            let sec_nano = Duration::seconds(1).num_nanoseconds().unwrap() as f64;
            let speed = self.current as f64 / (from_start / sec_nano);
            suffix = match self.units {
                Units::Default => suffix + &format!("{}/s ", speed),
                Units::Bytes => suffix + &format!("{}/s ", kb_fmt!(speed)),
            };
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
            let (c, t) = (self.current as f64, self.total as f64);
            prefix = match self.units {
                Units::Default => format!("{} / {} ", c, t),
                Units::Bytes => format!("{} / {} ", kb_fmt!(c), kb_fmt!(t)),
            };
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

    /// Calling finish manually will set current to total and draw
    /// the last time
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

#[cfg(test)]
mod test {
    use pb::{ProgressBar};

    #[test]
    fn add() {
        let mut pb = ProgressBar::new(10);
        pb.add(2);
        assert!(pb.current == 2, "should add the given `n` to current");
        assert!(pb.add(2) == pb.current, "add should return the current value");
    }

    #[test]
    fn inc() {
        let mut pb = ProgressBar::new(10);
        pb.inc();
        assert!(pb.current == 1, "should increment current by 1");
    }

    #[test]
    fn format() {
        let FORMAT = "[~> ]";
        let mut pb = ProgressBar::new(1);
        pb.format(FORMAT);
        assert!(pb.bar_start + &pb.bar_current + &pb.bar_current_n + &pb.bar_remain + &pb.bar_end == FORMAT);
    }

    #[test]
    fn finish() {
        let mut pb = ProgressBar::new(10);
        pb.finish();
        assert!(pb.current == pb.total, "should set current to total");
        assert!(pb.is_finish, "should set is_finish to true");
    }

    #[test]
    fn kb_fmt() {
        let kb = 1024f64;
        let mb = kb.powf(2f64);
        let gb = kb.powf(3f64);
        let tb = kb.powf(4f64);
        assert_eq!(kb_fmt!(kb), "1.00 KB");
        assert_eq!(kb_fmt!(mb), "1.00 MB");
        assert_eq!(kb_fmt!(gb), "1.00 GB");
        assert_eq!(kb_fmt!(tb), "1.00 TB");
    }
}
