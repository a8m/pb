extern crate libc;
use super::{Width, Height};

/// Dummy struct for `move_cursor_up_method()`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoveUpDummy;

impl MoveUpDummy {
    /// It's a dummy, it doesn't do anything.
    pub fn move_up(self) {}
}

/// Returns the size of the terminal, if available.
///
/// If STDOUT is not a tty, returns `None`
pub fn terminal_size() -> Option<(Width, Height)> {
    use self::libc::{ioctl, isatty, STDOUT_FILENO, TIOCGWINSZ, winsize};
    let is_tty: bool = unsafe { isatty(STDOUT_FILENO) == 1 };

    if !is_tty {
        return None;
    }

    let (rows, cols) = unsafe {
        let mut winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut winsize);
        let rows = if winsize.ws_row > 0 {
            winsize.ws_row
        } else {
            0
        };
        let cols = if winsize.ws_col > 0 {
            winsize.ws_col
        } else {
            0
        };
        (rows as u16, cols as u16)
    };

    if rows > 0 && cols > 0 {
        Some((Width(cols), Height(rows)))
    } else {
        None
    }
}

/// How to move the cursor up after printing the draw string.
///
/// If this returns `Ok(str)` append `str` to the draw string so it's internally synchronised (non-Windows).
///
/// If this returns `Err(str)` call `str.move_up()` after printing the draw string
/// to restore the cursor to the position before it got moved by printing (Windows).
///
/// Note: this creates desync issues mentioned in https://github.com/a8m/pb/pull/27#issuecomment-244564706 on platforms returning `Err()`
pub fn move_cursor_up_method(n: usize, _: bool) -> Result<String, MoveUpDummy> {
    if n != 0 {
        Ok(format!("\x1B[{}A", n))
    } else {
        Ok(MoveUpDummy)
    }
}

#[test]
/// Compare with the output of `stty size`
fn compare_with_stty() {
    use std::process::Command;
    use std::process::Stdio;
    let mut args = vec!["-F", "/dev/stderr", "size"];
    if cfg!(target_os = "macos") {
        args[0] = "-f"
    }
    let output = Command::new("stty")
                     .args(&args)
                     .stderr(Stdio::inherit())
                     .output()
                     .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());

    // stdout is "rows cols"
    let mut data = stdout.split_whitespace();
    let rows = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
    let cols = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
    println!("{}", stdout);
    println!("{} {}", rows, cols);

    if let Some((Width(w), Height(h))) = terminal_size() {
        assert_eq!(rows, h);
        assert_eq!(cols, w);
    }
}
