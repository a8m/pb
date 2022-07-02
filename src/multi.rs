use crate::tty::move_cursor_up;
use crate::ProgressBar;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::{Result, Stdout, Write};
use std::str::from_utf8;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct MultiBar<T: Write> {
    state: Mutex<State<T>>,
    chan: (Sender<WriteMsg>, Receiver<WriteMsg>),
    nbars: AtomicUsize,
}

struct State<T: Write> {
    lines: Vec<String>,
    nlines: usize,
    handle: T,
}

impl MultiBar<Stdout> {
    /// Create a new MultiBar with stdout as a writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread;
    /// use pbr::MultiBar;
    /// use std::time::Duration;
    ///
    /// let mut mb = MultiBar::new();
    /// mb.println("Application header:");
    ///
    /// # let count = 250;
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
            state: Mutex::new(State {
                lines: Vec::new(),
                handle,
                nlines: 0,
            }),
            chan: unbounded(),
            nbars: AtomicUsize::new(0),
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
    /// # let count = 250;
    /// let mut p1 = mb.create_bar(count);
    /// // ...
    ///
    /// mb.println("Text line between bar1 and bar2");
    ///
    /// let mut p2 = mb.create_bar(count);
    /// // ...
    ///
    /// mb.println("Text line between bar2 and bar3");
    ///
    /// // ...
    /// // ...
    /// mb.listen();
    /// ```
    pub fn println(&self, s: &str) {
        let mut state = self.state.lock().unwrap();
        state.lines.push(s.to_owned());
        state.nlines += 1;
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
    /// # let (count1, count2, count3) = (250, 62500, 15625000);
    ///
    /// // progress bar in level 1
    /// let mut p1 = mb.create_bar(count1);
    /// // ...
    ///
    /// // progress bar in level 2
    /// let mut p2 = mb.create_bar(count2);
    /// // ...
    ///
    /// // progress bar in level 3
    /// let mut p3 = mb.create_bar(count3);
    ///
    /// // ...
    /// mb.listen();
    /// ```
    pub fn create_bar(&self, total: u64) -> ProgressBar<Pipe> {
        let mut state = self.state.lock().unwrap();

        state.lines.push(String::new());
        state.nlines += 1;

        self.nbars.fetch_add(1, Ordering::SeqCst);

        let mut p = ProgressBar::on(
            Pipe {
                level: state.nlines - 1,
                chan: self.chan.0.clone(),
            },
            total,
        );

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
    /// use std::thread;
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
    pub fn listen(&self) {
        let mut first = true;
        let mut out = String::new();

        while self.nbars.load(Ordering::SeqCst) > 0 {
            // receive message
            let msg = self.chan.1.recv().unwrap();
            if msg.done {
                self.nbars.fetch_sub(1, Ordering::SeqCst);
                continue;
            }

            out.clear();
            let mut state = self.state.lock().unwrap();
            state.lines[msg.level] = msg.string;

            // and draw
            if !first {
                out += &move_cursor_up(state.nlines);
            } else {
                first = false;
            }

            for l in state.lines.iter() {
                out.push_str(&format!("\r{}\n", l));
            }

            printfl!(state.handle, "{}", out);
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
        self.chan
            .send(WriteMsg {
                // finish method emit empty string
                done: s.is_empty(),
                level: self.level,
                string: s,
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
}
