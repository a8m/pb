use std::io::{self, Write};
use std::iter::repeat;
use std::time::Duration;
use time::{self, SteadyTime};
use std::io::Stdout;
use tty::{Width, terminal_size};

macro_rules! printfl {
   ($w:expr, $($tt:tt)*) => {{
        $w.write(&format!($($tt)*).as_bytes()).ok().expect("write() fail");
        $w.flush().ok().expect("flush() fail");
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

const FORMAT: &'static str = "[=>-]";
const TICK_FORMAT: &'static str = "\\|/-";
const NANOS_PER_SEC: u32 = 1_000_000_000;

// Output type format, indicate which format wil be used in
// the speed box.
#[derive(Debug)]
pub enum Units {
    Default,
    Bytes,
}

pub struct ProgressBar<T: Write> {
    start_time: SteadyTime,
    units: Units,
    pub total: u64,
    current: u64,
    bar_start: String,
    bar_current: String,
    bar_current_n: String,
    bar_remain: String,
    bar_end: String,
    tick: Vec<String>,
    tick_state: usize,
    tick_len: usize,
    message: String,
    pub is_finish: bool,
    pub show_bar: bool,
    pub show_speed: bool,
    pub show_percent: bool,
    pub show_counter: bool,
    pub show_time_left: bool,
    pub show_tick: bool,
    pub show_message: bool,
    handle: T,
}

impl ProgressBar<Stdout> {
    /// Create a new ProgressBar with default configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread;
    /// use pbr::{ProgressBar, Units};
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
    pub fn new(total: u64) -> ProgressBar<Stdout> {
        let handle = ::std::io::stdout();
        ProgressBar::on(handle, total)
    }
}

impl<T: Write> ProgressBar<T> {
    /// Create a new ProgressBar with default configuration but
    /// pass an arbitrary writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread;
    /// use std::io::stderr;
    /// use pbr::{ProgressBar, Units};
    ///
    /// let count = 1000;
    /// let mut pb = ProgressBar::on(stderr(), count);
    /// pb.set_units(Units::Bytes);
    ///
    /// for _ in 0..count {
    ///    pb.inc();
    ///    thread::sleep_ms(100);
    /// }
    /// ```
    pub fn on(handle: T, total: u64) -> ProgressBar<T> {
        let mut pb = ProgressBar {
            total: total,
            current: 0,
            start_time: SteadyTime::now(),
            units: Units::Default,
            is_finish: false,
            show_bar: true,
            show_speed: true,
            show_percent: true,
            show_counter: true,
            show_time_left: true,
            show_tick: false,
            show_message: true,
            bar_start: String::new(),
            bar_current: String::new(),
            bar_current_n: String::new(),
            bar_remain: String::new(),
            bar_end: String::new(),
            tick: Vec::new(),
            tick_state: 0,
            tick_len: 4,
            message: String::new(),
            handle: handle,
        };
        pb.format(FORMAT);
        pb.tick_format(TICK_FORMAT);
        pb
    }

    /// Set units, default is simple numbers
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::{ProgressBar, Units};
    ///
    /// let n_bytes = 100;
    /// let mut pb = ProgressBar::new(n_bytes);
    /// pb.set_units(Units::Bytes);
    /// ```
    pub fn set_units(&mut self, u: Units) {
        self.units = u;
    }

    /// Set custom format to the drawing bar, default is `[=>-]`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut pb = ProgressBar::new(...);
    /// pb.format("[=>_]");
    /// ```
    pub fn format(&mut self, fmt: &str) {
        if fmt.len() >= 5 {
            let v: Vec<&str> = fmt.split("").collect();
            self.bar_start = v[1].to_owned();
            self.bar_current = v[2].to_owned();
            self.bar_current_n = v[3].to_owned();
            self.bar_remain = v[4].to_owned();
            self.bar_end = v[5].to_owned();
        }
    }

    /// Set message to display in the prefix, call with "" to stop printing a message.
    ///
    /// # Examples
    /// ```ignore
    /// let mut pb = ProgressBar::new(20);
    ///
    /// for x in 0..20 {
    ///    match x {
    ///       0 => pb.message("Doing 1st Quarter"),
    ///       5 => pb.message("Doing 2nd Quarter"),
    ///       10 => pb.message("Doing 3rd Quarter"),
    ///       15 => pb.message("Doing 4th Quarter"),
    ///    }
    ///    pb.inc().
    /// }
    ///
    /// ```
    pub fn message(&mut self, message: &str) {
        self.message = message.to_owned()
    }

    /// Set tick format for the progressBar, default is \\|/-
    ///
    /// Format is not limited to 4 characters, any string can
    /// be used as a tick format (the tick will successively
    /// take the value of each char but won't loop backwards).
    ///
    ///
    /// # Examples
    /// ```ignore
    /// let mut pb = ProgressBar::new(...);
    /// pb.tick_format("▀▐▄▌")
    /// ```
    pub fn tick_format(&mut self, tick_fmt: &str) {
        if tick_fmt != TICK_FORMAT {
            self.show_tick = true;
        };
        self.tick = tick_fmt.split("").map(|x| x.to_owned()).filter(|x| x != "").collect();
        self.tick_len = self.tick.len();
    }

    /// Update progress bar even though no progress are made
    /// Useful to see if a program is bricked or just
    /// not doing any progress.
    ///
    /// tick is not needed with add or inc
    /// as performed operation take place
    /// in draw function.
    ///
    /// # Examples
    /// ```ignore
    /// let mut pb = ProgressBar::new(...);
    /// pb.inc();
    /// for _ in ... {
    ///    ...do something
    ///    pb.tick();
    /// }
    /// pb.finish();
    /// ```
    pub fn tick(&mut self) {
        self.tick_state = (self.tick_state + 1) % self.tick_len;
        if self.current <= self.total {
            self.draw()
        };
    }

    /// Add to current value
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::ProgressBar;
    ///
    /// let mut pb = ProgressBar::new(10);
    /// pb.add(5);
    /// pb.finish();
    /// ```
    pub fn add(&mut self, i: u64) -> u64 {
        self.current += i;
        self.tick();
        self.current
    }

    /// Increment current value
    pub fn inc(&mut self) -> u64 {
        self.add(1)
    }

    fn draw(&mut self) {
        let time_elapsed = time_to_std(SteadyTime::now() - self.start_time);
        let speed = self.current as f64 / fract_dur(time_elapsed);

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
        if self.show_speed {
            suffix = match self.units {
                Units::Default => suffix + &format!("{:.*}/s ", 2, speed),
                Units::Bytes => suffix + &format!("{}/s ", kb_fmt!(speed)),
            };
        }
        // time left box
        if self.show_time_left && self.current > 0 {
            if self.total > self.current {
                let left = 1. / speed * (self.total - self.current) as f64;
                if left < 60. {
                    suffix = suffix + &format!("{:.0}s", left);
                } else {
                    suffix = suffix + &format!("{:.0}m", left / 60.);
                }
            }
        }
        // message box
        if self.show_message {
            prefix = prefix + &format!("{}", self.message)
        }
        // counter box
        if self.show_counter {
            let (c, t) = (self.current as f64, self.total as f64);
            prefix = prefix +
                     &match self.units {
                Units::Default => format!("{} / {} ", c, t),
                Units::Bytes => format!("{} / {} ", kb_fmt!(c), kb_fmt!(t)),
            };
        }
        // tick box
        if self.show_tick {
            prefix = prefix + &format!("{} ", self.tick[self.tick_state]);
        }
        // bar box
        if self.show_bar {
            let p = prefix.len() + suffix.len() + 3;
            if p <= width {
                let size = width - p;
                if size > 0 {
                    let curr_count = ((self.current as f64 / self.total as f64) * size as f64)
                        .ceil() as usize;
                    if size > curr_count {
                        let rema_count = size - curr_count;
                        base = self.bar_start.clone();
                        if rema_count > 0 && curr_count > 0 {
                            base = base + repeat!(self.bar_current.as_ref(), curr_count - 1) +
                                   &self.bar_current_n;
                        } else {
                            base = base + repeat!(self.bar_current.as_ref(), curr_count);
                        }
                        base = base + repeat!(self.bar_remain.as_ref(), rema_count) + &self.bar_end;
                    }
                }
            }
        }
        out = prefix + &base + &suffix;
        // pad
        if out.len() < width {
            let gap = width - out.len();
            out = out + repeat!(" ", gap);
        }
        // print
        printfl!(self.handle, "\r{}", out);
    }

    /// Calling finish manually will set current to total and draw
    /// the last time
    pub fn finish(&mut self) {
        if self.current < self.total {
            self.current = self.total;
            self.draw();
        }
        printfl!(self.handle, "");
        self.is_finish = true;
    }

    /// Call finish and write string 's'
    pub fn finish_print(&mut self, s: &str) {
        self.finish();
        printfl!(self.handle, "{}", s)
    }
}

// Implement io::Writer
impl<T: Write> Write for ProgressBar<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len();
        self.add(n as u64);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn time_to_std(d: time::Duration) -> Duration {
    assert!(d > time::Duration::zero());

    let secs = d.num_seconds();
    let nsecs = (d - time::Duration::seconds(secs)).num_nanoseconds().unwrap();
    Duration::new(secs as u64, nsecs as u32)
}

fn fract_dur(d: Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 / NANOS_PER_SEC as f64
}

#[cfg(test)]
mod test {
    use pb::ProgressBar;

    #[test]
    fn add() {
        let mut pb = ProgressBar::new(10);
        pb.add(2);
        assert!(pb.current == 2, "should add the given `n` to current");
        assert!(pb.add(2) == pb.current,
                "add should return the current value");
    }

    #[test]
    fn inc() {
        let mut pb = ProgressBar::new(10);
        pb.inc();
        assert!(pb.current == 1, "should increment current by 1");
    }

    #[test]
    fn format() {
        let fmt = "[~> ]";
        let mut pb = ProgressBar::new(1);
        pb.format(fmt);
        assert!(pb.bar_start + &pb.bar_current + &pb.bar_current_n + &pb.bar_remain +
                &pb.bar_end == fmt);
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
