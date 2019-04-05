use pb::ProgressBar;
use std::str::from_utf8;
use tty::move_cursor_up;
use std::io::{Stdout, Result, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;

pub enum FinishMethod {
    Graduate,
    Remove,
    Keep,
}

pub struct MultiBar<T: Write> {
    nlines: usize,

    lines: Vec<String>,

    bar_id: u64,

    bars: HashMap<u64, usize>,

    chan: (Sender<WriteMsg>, Receiver<WriteMsg>),

    handle: T,

    finish: FinishMethod,
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
            nlines: 0,
            bar_id: 0,
            lines: Vec::new(),
            bars: HashMap::new(),
            chan: mpsc::channel(),
            handle: handle,
            finish: FinishMethod::Keep,
        }
    }

    /// Set finish method
    /// FinishMethod::Keep => keep the bar when finishing
    /// FinishMethod::Remove => remove the bar when finishing
    /// FinishMethod::Graduate => graduate the bar when finishing
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pbr::{MultiBar, FinishMethod};
    /// use std::io::stderr;
    ///
    /// let mut mb = MultiBar::on(stderr());
    /// mb.set_finish(FinishMethod::Remove);
    /// // ...
    /// // see full example in `MultiBar::new`
    /// // ...
    /// ```
    pub fn set_finish(&mut self, finish: FinishMethod) {
        self.finish = finish
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
    pub fn println(&mut self, s: &str) {
        self.lines.push(s.to_owned());
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
    pub fn create_bar(&mut self, total: u64) -> ProgressBar<Pipe> {
        let bar_id = self.bar_id;
        self.bars.insert(bar_id, self.nlines);
        let mut p = ProgressBar::on(Pipe {
                                        bar_id: bar_id,
                                        chan: self.chan.0.clone(),
                                    },
                                    total);
        self.println(&format!("bar_{}", bar_id));
        self.bar_id += 1;
        p.is_multibar = true;
        p.add(0);
        p
    }

    fn bar_finish(&mut self, bar_id: u64) -> String {
        let result = match self.finish {
            FinishMethod::Keep => String::new(),
            FinishMethod::Remove | FinishMethod::Graduate => {
                for (&i, v) in self.bars.iter_mut() {
                    if i > bar_id {
                        *v -= 1;
                    }
                }
                self.nlines -= 1;
                self.lines.remove(self.bars[&bar_id])
            }
        };
        self.bars.remove(&bar_id);
        if let FinishMethod::Graduate = self.finish {
            format!("\r{}\n", result)
        } else {
            String::new()
        }
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
    pub fn listen(&mut self) {
        let mut first = true;
        let mut nlines = self.nlines;
        let mut graduate = String::new();
        while self.bars.len() > 0 {

            // receive message
            let msg = self.chan.1.recv().unwrap();
            if msg.done {
                graduate.push_str(&self.bar_finish(msg.bar_id));
                continue;
            }
            self.lines[self.bars[&msg.bar_id]] = msg.string;

            // and draw
            let mut out = String::new();
            if !first {
                out += &move_cursor_up(nlines);
            } else {
                first = false;
            }
            out.push_str(&graduate);
            graduate = String::new();
            for l in self.lines.iter() {
                out.push_str(&format!("\r{}\n", l));
            }
            printfl!(self.handle, "{}", out);
            nlines = self.nlines;
        }
    }

    /// flush all bars.
    ///
    /// see also `listen()`
    ///
    /// This is a non-blocking operation.
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
    /// let nlines = mb.flush(None);
    ///
    /// // ...
    /// // change bars here
    /// // ...
    ///
    /// mb.flush(Some(nlines));
    ///
    /// // ...
    /// ```
    pub fn flush(&mut self, nlines: Option<usize>) -> usize {
        if self.bars.len() > 0 {
            let mut graduate = String::new();

            // receive message
            loop {
                let msg = if let Ok(msg) = self.chan.1.try_recv() {
                    msg
                } else  {
                    break;
                };
                if msg.done {
                    graduate.push_str(&self.bar_finish(msg.bar_id));
                    continue;
                }
                self.lines[self.bars[&msg.bar_id]] = msg.string;
            }

            // and draw
            let mut out = String::new();
            if nlines.is_some() {
                out += &move_cursor_up(nlines.unwrap());
            }
            out.push_str(&graduate);
            for l in self.lines.iter() {
                out.push_str(&format!("\r{}\n", l));
            }
            printfl!(self.handle, "{}", out);
        }
        self.nlines
    }
}

pub struct Pipe {
    bar_id: u64,
    chan: Sender<WriteMsg>,
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let s = from_utf8(buf).unwrap().to_owned();
        self.chan
            .send(WriteMsg {
                // finish method emit empty string
                done: s == "",
                bar_id: self.bar_id,
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
    bar_id: u64,
    string: String,
}
