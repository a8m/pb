use pb::ProgressBar;
use std::str::from_utf8;
use tty::{Width, terminal_size, move_cursor_up};
use std::io::{Stdout, Result, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::iter::repeat;

use ::PBR_LOG_BOUNDARY;

pub struct MultiBar<T: Write> {
    nlines: usize,

    lines: Vec<String>,

    nbars: usize,

    chan: (Sender<WriteMsg>, Receiver<WriteMsg>),

    handle: T,

    width: Option<usize>,
}

impl MultiBar<Stdout> {
    /// Create a new MultiBar with stdout as a writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread;
    /// use pbr::MultiBar;
    ///
    /// let mut mb = MultiBar::new();
    /// mb.println("Application header:");
    ///
    /// let mut p1 = mb.create_bar(count);
    /// let _ = thread::spawn(move || {
    ///     for _ in 0..count {
    ///         p1.inc();
    ///         thread::sleep(Duration::from_millis(100));
    ///     }
    ///     // notify the multibar that this bar finished.
    ///     p1.finish();
    /// });
    ///
    /// mb.println("add a separator between the two bars");
    ///
    /// let mut p2 = mb.create_bar(count * 2);
    /// let _ = thread::spawn(move || {
    ///     for _ in 0..count * 2 {
    ///         p2.inc();
    ///         thread::sleep(Duration::from_millis(100));
    ///     }
    ///     // notify the multibar that this bar finished.
    ///     p2.finish();
    /// });
    ///
    /// // start listen to all bars changes.
    /// // this is a blocking operation, until all bars will finish.
    /// // to ignore blocking, you can run it in a different thread.
    /// mb.listen();
    /// ```
    pub fn new() -> MultiBar<Stdout> {
        MultiBar::on(::std::io::stdout())
    }
}

impl<T: Write> MultiBar<T> {
    /// Create a new MultiBar with an arbitrary writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::MultiBar;
    /// use std::io::stderr;
    ///
    /// let mut mb = MultiBar::on(stderr());
    /// // ...
    /// // see full example in `MultiBar::new`
    /// // ...
    /// ```
    pub fn on(handle: T) -> MultiBar<T> {
        MultiBar {
            nlines: 0,
            nbars: 0,
            lines: Vec::new(),
            chan: mpsc::channel(),
            handle: handle,
            width: None,
        }
    }

    /// println used to add text lines between the bars.
    /// for example: you could add a header to your application,
    /// or text separators between bars.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::MultiBar;
    ///
    /// let mut mb = MultiBar::new();
    /// mb.println("Application header:");
    ///
    /// let mut p1 = MultiBar::create_bar(count);
    /// // ...
    ///
    /// mb.println("Text line between bar1 and bar2");
    ///
    /// let mut p2 = MultiBar::create_bar(count);
    /// // ...
    ///
    /// mb.println("Text line between bar2 and bar3");
    ///
    /// // ...
    /// // ...
    /// mb.listen();
    /// ```
    pub fn println(&mut self, s: &str) {
        let mut out = format!("{}", s);

        let width = self.width();

        if out.len() < width {
            let gap = width - out.len();
            out = out + repeat!(" ", gap);
        }

        self.lines.push(out);
        self.nlines += 1;
    }

    /// create_bar creates new `ProgressBar` with `Pipe` as the writer.
    ///
    /// The ordering of the method calls is important. it means that in
    /// the first call, you get a progress bar in level 1, in the 2nd call,
    /// you get a progress bar in level 2, and so on.
    ///
    /// ProgressBar that finish its work, must call `finish()` (or `finish_print`)
    /// to notify the `MultiBar` about it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::MultiBar;
    ///
    /// let mut mb = MultiBar::new();
    ///
    /// // progress bar in level 1
    /// let mut p1 = MultiBar::create_bar(count1);
    /// // ...
    ///
    /// // progress bar in level 2
    /// let mut p2 = MultiBar::create_bar(count2);
    /// // ...
    ///
    /// // progress bar in level 3
    /// let mut p3 = MultiBar::create_bar(count3);
    ///
    /// // ...
    /// mb.listen();
    /// ```
    pub fn create_bar(&mut self, total: u64) -> ProgressBar<Pipe> {
        self.println("");
        self.nbars += 1;
        let mut p = ProgressBar::on(Pipe {
                                        level: self.nlines - 1,
                                        chan: self.chan.0.clone(),
                                    },
                                    total);
        p.set_width(self.width);
        p.is_multibar = true;
        p.add(0);
        p
    }


    /// listen start listen to all bars changes.
    ///
    /// `ProgressBar` that finish its work, must call `finish()` (or `finish_print`)
    /// to notify the `MultiBar` about it.
    ///
    /// This is a blocking operation and blocks until all bars will
    /// finish.
    /// To ignore blocking, you can run it in a different thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::MultiBar;
    ///
    /// let mut mb = MultiBar::new();
    ///
    /// // ...
    /// // create some bars here
    /// // ...
    ///
    /// thread::spawn(move || {
    ///     mb.listen();
    ///     println!("all bars done!");
    /// });
    ///
    /// // ...
    /// ```
    pub fn listen(&mut self) {
        let mut first = true;
        let mut nbars = self.nbars;
        while nbars > 0 {

            // receive message
            let msg = self.chan.1.recv().unwrap();
            if msg.done {
                nbars -= 1;
                continue;
            }
            self.lines[msg.level] = msg.string;

            // and draw
            let mut out = String::new();
            if !first {
                out += &move_cursor_up(self.nlines);
            } else {
                first = false;
            }

            // draw the log line if we have one & scroll the log message upward to prevent it from
            // being overwritten by the progress bar(s) and message strings
            if let Some(log_line) = msg.log_line {
                out.push_str(&format!("\r{}\n", log_line));
            }

            for l in self.lines.iter() {
                out.push_str(&format!("\r{}\n", l));
            }
            printfl!(self.handle, "{}", out);
        }
    }

    /// Set width, or `None` for default.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut mb = MultiBar::new(...);
    /// mb.set_width(Some(80));
    /// ```
    pub fn set_width(&mut self, w: Option<usize>) {
        self.width = w;
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

pub struct Pipe {
    level: usize,
    chan: Sender<WriteMsg>,
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let s = from_utf8(buf).unwrap().to_owned();

        // check to see if ProgressBar set a logging boundary & split out the log if we find it
        let (log_line, bar) = match s.contains(PBR_LOG_BOUNDARY) {
            true => {
                let v: Vec<&str> = s.split(PBR_LOG_BOUNDARY).collect();
                let log_line = Some(v[0].to_owned());

                let bar = v[1].to_owned();

                (log_line, bar)
            },
            false => {
                (None, s)
            }

        };

        self.chan
            .send(WriteMsg {
                // finish method emit empty string
                done: bar == "",
                level: self.level,
                string: bar,
                log_line: log_line,
            })
            .unwrap();
        Ok(1)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// WriteMsg is the message format used to communicate
// between MultiBar and its bars
struct WriteMsg {
    done: bool,
    level: usize,
    string: String,
    log_line: Option<String>,
}
