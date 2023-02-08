use super::{Height, Width};

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

/// move the cursor `n` lines up; return an empty string, just to
/// be aligned with the unix version.
pub fn move_cursor_up(n: usize) -> String {
    use winapi::um::wincon::{SetConsoleCursorPosition, COORD};
    if let Some((hand, csbi)) = get_csbi() {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: 0,
                    Y: csbi.dwCursorPosition.Y - n as i16,
                },
            );
        }
    }
    "".to_string()
}

fn get_csbi() -> Option<(
    winapi::shared::ntdef::HANDLE,
    winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO,
)> {
    use winapi::shared::ntdef::HANDLE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use winapi::um::wincon::{
        GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO, COORD, SMALL_RECT,
    };

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
