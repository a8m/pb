use crate::tty::{terminal_size, Width};
use std::io::Stdout;
use std::io::{self, Write};
use std::time::{Duration, Instant};

macro_rules! kb_fmt {
    ($n: ident) => {{
        let kb = 1024f64;
        match $n {
            $n if $n >= kb.powf(4_f64) => format!("{:.*} TB", 2, $n / kb.powf(4_f64)),
            $n if $n >= kb.powf(3_f64) => format!("{:.*} GB", 2, $n / kb.powf(3_f64)),
            $n if $n >= kb.powf(2_f64) => format!("{:.*} MB", 2, $n / kb.powf(2_f64)),
            $n if $n >= kb => format!("{:.*} KB", 2, $n / kb),
            _ => format!("{:.*} B", 0, $n),
        }
    }};
}

const FORMAT: &str = "[=>-]";
const TICK_FORMAT: &str = "\\|/-";

// Output type format, indicate which format wil be used in
// the speed box.
#[derive(Debug)]
pub enum Units {
    Default,
    Bytes,
}

pub struct ProgressBar<T: Write> {
    start_time: Instant,
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
    width: Option<usize>,
    message: String,
    last_refresh_time: Instant,
    max_refresh_rate: Option<Duration>,
    pub is_finish: bool,
    pub is_multibar: bool,
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
            total,
            current: 0,
            start_time: Instant::now(),
            units: Units::Default,
            is_finish: false,
            is_multibar: false,
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
            width: None,
            message: String::new(),
            last_refresh_time: Instant::now(),
            max_refresh_rate: None,
            handle,
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
    /// All newlines are replaced with spaces.
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
        self.message = message.replace(['\n', '\r'], " ")
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
        }
        self.tick = tick_fmt
            .split("")
            .map(|x| x.to_owned())
            .filter(|x| !x.is_empty())
            .collect();
    }

    /// Set width, or `None` for default.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut pb = ProgressBar::new(...);
    /// pb.set_width(Some(80));
    /// ```
    pub fn set_width(&mut self, w: Option<usize>) {
        self.width = w;
    }

    /// Set max refresh rate, above which the progress bar will not redraw, or `None` for none.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut pb = ProgressBar::new(...);
    /// pb.set_max_refresh_rate(Some(Duration::from_millis(100)));
    /// ```
    pub fn set_max_refresh_rate(&mut self, w: Option<Duration>) {
        self.max_refresh_rate = w;
        if let Some(dur) = self.max_refresh_rate {
            self.last_refresh_time = self.last_refresh_time - dur;
        }
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
        self.tick_state = (self.tick_state + 1) % self.tick.len();
        if self.current <= self.total {
            self.draw()
        }
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

    /// Manually set the current value of the bar
    ///
    /// # Examples
    /// ```no_run
    /// use pbr::ProgressBar;
    ///
    /// let mut pb = ProgressBar::new(10);
    /// pb.set(8);
    /// pb.finish();
    pub fn set(&mut self, i: u64) -> u64 {
        self.current = i;
        self.tick();
        self.current
    }

    /// Increment current value
    pub fn inc(&mut self) -> u64 {
        self.add(1)
    }

    /// Resets the start time to now
    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
    }

    fn draw(&mut self) {
        let now = Instant::now();
        if let Some(mrr) = self.max_refresh_rate {
            if now - self.last_refresh_time < mrr && self.current < self.total {
                return;
            }
        }

        let mut time_elapsed = now - self.start_time;
        if time_elapsed.is_zero() {
            time_elapsed = Duration::from_nanos(1);
        }
        let speed = self.current as f64 / time_elapsed.as_secs_f64();
        let width = self.width();

        let mut out;
        let mut parts = Vec::new();
        let mut base = String::new();
        let mut prefix = String::new();
        let mut suffix = String::from(" ");

        // precent box
        if self.show_percent {
            let percent = self.current as f64 / (self.total as f64 / 100f64);
            parts.push(format!(
                "{:.*} %",
                2,
                if percent.is_nan() { 0.0 } else { percent }
            ));
        }
        // speed box
        if self.show_speed {
            match self.units {
                Units::Default => parts.push(format!("{:.*}/s", 2, speed)),
                Units::Bytes => parts.push(format!("{}/s", kb_fmt!(speed))),
            };
        }
        // time left box
        if self.show_time_left && self.current > 0 && self.total > self.current {
            let left = 1. / speed * (self.total - self.current) as f64;
            if left < 60. {
                parts.push(format!("{:.0}s", left));
            } else {
                parts.push(format!("{:.0}m", left / 60.));
            };
        }
        suffix += &parts.join(" ");
        // message box
        if self.show_message {
            prefix = prefix + &self.message;
        }
        // counter box
        if self.show_counter {
            let (c, t) = (self.current as f64, self.total as f64);
            prefix = prefix
                + &match self.units {
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
            let p = prefix.chars().count() + suffix.chars().count() + 3;
            if p < width {
                let size = width - p;
                let curr_count =
                    ((self.current as f64 / self.total as f64) * size as f64).ceil() as usize;
                if size >= curr_count {
                    let rema_count = size - curr_count;
                    base = self.bar_start.clone();
                    if rema_count > 0 && curr_count > 0 {
                        base =
                            base + &self.bar_current.repeat(curr_count - 1) + &self.bar_current_n;
                    } else {
                        base = base + &self.bar_current.repeat(curr_count);
                    }
                    base = base + &self.bar_remain.repeat(rema_count) + &self.bar_end;
                }
            }
        }
        out = prefix + &base + &suffix;
        // pad
        if out.len() < width {
            let gap = width - out.len();
            out = out + &" ".repeat(gap);
        }
        // print
        printfl!(self.handle, "\r{}", out);

        self.last_refresh_time = Instant::now();
    }

    // finish_draw ensure that the progress bar is reached to its end, and do the
    // last drawing if needed.
    fn finish_draw(&mut self) {
        let mut redraw = false;

        if let Some(mrr) = self.max_refresh_rate {
            if Instant::now() - self.last_refresh_time < mrr {
                self.max_refresh_rate = None;
                redraw = true;
            }
        }

        if self.current < self.total {
            self.current = self.total;
            redraw = true;
        }

        if redraw {
            self.draw();
        }
        self.is_finish = true;
    }

    /// Calling finish manually will set current to total and draw
    /// the last time
    pub fn finish(&mut self) {
        self.finish_draw();
        printfl!(self.handle, "");
    }

    /// Call finish and write string `s` that will replace the progress bar.
    pub fn finish_print(&mut self, s: &str) {
        self.finish_draw();
        let width = self.width();
        let mut out = s.to_owned();
        if s.len() < width {
            out += &" ".repeat(width - s.len());
        };
        printfl!(self.handle, "\r{}", out);
        self.finish();
    }

    /// Call finish and write string `s` below the progress bar.
    ///
    /// If the ProgressBar is part of MultiBar instance, you should use
    /// `finish_print` to print message.
    pub fn finish_println(&mut self, s: &str) {
        // `finish_println` does not allow in MultiBar mode, because printing
        // new line will break the multiBar output.
        if self.is_multibar {
            return self.finish_print(s);
        }
        self.finish_draw();
        printfl!(self.handle, "\n{}", s);
    }

    /// Get terminal width, from configuration, terminal size, or default(80)
    fn width(&mut self) -> usize {
        if let Some(w) = self.width {
            w
        } else if let Some((Width(w), _)) = terminal_size() {
            w as usize
        } else {
            80
        }
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

#[cfg(test)]
mod test {
    use crate::{ProgressBar, Units};
    use std::time::Duration;

    #[test]
    fn add() {
        let mut pb = ProgressBar::new(10);
        pb.add(2);
        assert!(pb.current == 2, "should add the given `n` to current");
        assert!(
            pb.add(2) == pb.current,
            "add should return the current value"
        );
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
        assert!(
            pb.bar_start + &pb.bar_current + &pb.bar_current_n + &pb.bar_remain + &pb.bar_end
                == fmt
        );
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

    #[test]
    fn disable_speed_percent() {
        let mut out = Vec::new();
        let mut pb = ProgressBar::on(&mut out, 10);
        pb.show_speed = false;
        pb.show_percent = false;
        pb.set_width(Some(80));
        pb.add(2);
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "\r2 / 10 [=============>-----------------------------------------------------] 0s ",
        );
    }

    #[test]
    fn disable_speed_time_left() {
        let mut out = Vec::new();
        let mut pb = ProgressBar::on(&mut out, 10);
        pb.show_speed = false;
        pb.show_time_left = false;
        pb.set_width(Some(65));
        pb.add(1);
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "\r1 / 10 [====>------------------------------------------] 10.00 % ",
        );
    }

    #[test]
    fn disable_percent_time_left() {
        let mut out = Vec::new();
        let mut pb = ProgressBar::on(&mut out, 10);
        pb.show_percent = false;
        pb.show_time_left = false;
        pb.set_units(Units::Bytes);
        pb.set_width(Some(65));
        pb.draw();
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "\r0 B / 10 B [---------------------------------------------] 0 B/s ",
        );
    }

    #[test]
    fn disable_suffix() {
        let mut out = Vec::new();
        let mut pb = ProgressBar::on(&mut out, 10);
        pb.show_speed = false;
        pb.show_percent = false;
        pb.show_time_left = false;
        pb.set_units(Units::Bytes);
        pb.set_width(Some(65));
        pb.draw();
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "\r0 B / 10 B [--------------------------------------------------]  ",
        );
    }

    #[test]
    fn max_refresh_rate_finish() {
        let count = 500;
        let mut out = Vec::new();
        let mut pb = ProgressBar::on(&mut out, count);
        pb.format("╢▌▌░╟");
        pb.set_width(Some(80));
        pb.set_max_refresh_rate(Some(Duration::from_millis(100)));
        pb.show_speed = false;
        pb.show_time_left = false;
        pb.add(count / 2);
        pb.add(count / 2);
        let mut split = std::str::from_utf8(&out)
            .unwrap()
            .trim_start_matches('\r')
            .split('\r');
        assert_eq!(
            split.next(),
            Some("250 / 500 ╢▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░░╟ 50.00 %")
        );
        assert_eq!(
            split.next(),
            Some("500 / 500 ╢▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌╟ 100.00 %")
        );
    }
}
