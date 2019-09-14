extern crate termion;
use super::{Height, Width};

pub fn terminal_size() -> Option<(Width, Height)> {
    match termion::terminal_size() {
        Ok((cols, rows)) => Some((Width(cols), Height(rows))),
        Err(..) => None,
    }
}

pub fn move_cursor_up(n: usize) -> String {
    format!("{}", termion::cursor::Up(n as u16))
}
