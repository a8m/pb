extern crate libc;
use super::{Width, Height};
use std::io::Write;

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

/// Sometimes save cursor position for restore;
///
/// Magic for use with `restore_cursor_pos_or_move_cursor_n_up()`.
///
/// Do **not** rely on this to return the actual cursor position.
pub fn save_cursor_pos(_: bool) -> (usize, usize) {
    (0, 0)
}

/// Either restore cursor position saved with `save_cursor_pos()` or move the cursor `n` lines up.
///
/// 300% magic.
pub fn restore_cursor_pos_or_move_cursor_n_up<W: Write>(out: &mut W, _: (usize, usize), n: usize, _: bool) {
    if n != 0 {
        write!(out, "\x1B[{}A", n).ok().expect("write() fail");
        out.flush().ok().expect("flush() fail");
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
