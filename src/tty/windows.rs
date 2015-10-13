extern crate winapi;
extern crate kernel32;

use super::{Width, Height};

/// Returns the size of the terminal, if available.
///
/// Note that this returns the size of the actual command window, and
/// not the overall size of the command window buffer
pub fn terminal_size() -> Option<(Width, Height)> {
    use self::winapi::{DWORD, HANDLE};
    use self::kernel32::{GetStdHandle, GetConsoleScreenBufferInfo};
    use self::winapi::{STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
    use self::winapi::{CONSOLE_SCREEN_BUFFER_INFO, COORD, SMALL_RECT};

    let hand: HANDLE = unsafe {
        GetStdHandle(STD_OUTPUT_HANDLE)
    };

    let zc = COORD{X: 0, Y: 0};
    let mut csbi = CONSOLE_SCREEN_BUFFER_INFO{
        dwSize: zc.clone(),
        dwCursorPosition: zc.clone(),
        wAttributes: 0,
        srWindow: SMALL_RECT{Left:0, Top: 0, Right: 0, Bottom: 0},
        dwMaximumWindowSize: zc

    };
    let success:bool = unsafe {
        GetConsoleScreenBufferInfo(hand, &mut csbi) != 0
    };
    if success {
        let w: Width = Width((csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16);
        let h: Height = Height((csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16);
        Some((w, h))
    } else {
        None
    }
}
