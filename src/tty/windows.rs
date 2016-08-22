extern crate winapi;
extern crate kernel32;

use super::{Width, Height};
use std::io::Write;

/// Returns the size of the terminal, if available.
///
/// Note that this returns the size of the actual command window, and
/// not the overall size of the command window buffer
pub fn terminal_size() -> Option<(Width, Height)> {
    if let Some((_, csbi)) = get_csbi() {
        let w: Width = Width((csbi.srWindow.Right - csbi.srWindow.Left) as u16);
        let h: Height = Height((csbi.srWindow.Bottom - csbi.srWindow.Top) as u16);
        Some((w, h))
    } else {
        None
    }
}

/// Sometimes save cursor position for restore;
///
/// Magic for use with `restore_cursor_pos_or_move_cursor_n_up()`.
///
/// Do **not** rely on this to return the actual cursor position.
pub fn save_cursor_pos(console_out: bool) -> (usize, usize) {
    if console_out {
        if let Some((_, csbi)) = get_csbi() {
            (csbi.dwCursorPosition.X as usize, csbi.dwCursorPosition.Y as usize)
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    }
}

/// Either restore cursor position saved with `save_cursor_pos()` or move the cursor `n` lines up.
///
/// 300% magic.
pub fn restore_cursor_pos_or_move_cursor_n_up<W: Write>(_: &mut W, pos: (usize, usize), _: usize, console_out: bool) {
    use self::kernel32::SetConsoleCursorPosition;
    use self::winapi::COORD;

    if console_out {
        if let Some((hand, _)) = get_csbi() {
            unsafe {
                SetConsoleCursorPosition(hand, COORD {
                    X: pos.0 as i16,
                    Y: pos.1 as i16,
                });
            }
        }
    }
}

fn get_csbi() -> Option<(self::winapi::HANDLE, self::winapi::CONSOLE_SCREEN_BUFFER_INFO)> {
    use self::winapi::HANDLE;
    use self::kernel32::{GetStdHandle, GetConsoleScreenBufferInfo};
    use self::winapi::STD_OUTPUT_HANDLE;
    use self::winapi::{CONSOLE_SCREEN_BUFFER_INFO, COORD, SMALL_RECT};

    let hand: HANDLE = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };

    let zc = COORD { X: 0, Y: 0 };
    let mut csbi = CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: zc.clone(),
        dwCursorPosition: zc.clone(),
        wAttributes: 0,
        srWindow: SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: 0,
            Bottom: 0,
        },
        dwMaximumWindowSize: zc,
    };
    match unsafe { GetConsoleScreenBufferInfo(hand, &mut csbi) } {
        0 => None,
        _ => Some((hand, csbi)),
    }
}
