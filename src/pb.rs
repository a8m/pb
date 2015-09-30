extern crate time;
use time::*;

pub struct ProgressBar {
    start_time:  time.Timespec,
    total:      i64,
    current:    i64,
}
