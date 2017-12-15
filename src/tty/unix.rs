extern crate termion;
extern crate libc;
use super::{Width, Height};

/// Returns the size of the terminal, if available.
///
/// If STDOUT is not a tty, returns `None`
#[cfg(target_os = "redox")]
pub fn terminal_size() -> Option<(Width, Height)> {
    match termion::terminal_size() {
        Ok((cols, rows)) => Some((Width(cols), Height(rows))),
        Err(..) => None
    }
}

#[cfg(not(target_os = "redox"))]
fn terminal_size_fd(fd: libc::c_int) -> Option<(Width, Height)> {
    use std::mem;

    unsafe {
        let mut size: libc::winsize = mem::zeroed();
        if libc::ioctl(fd, libc::TIOCGWINSZ, &mut size as *mut _) == 0 {
            Some((Width(size.ws_col), Height(size.ws_row)))
        } else {
            None
        }
    }
}

/// Returns the size of the terminal, if available.
///
/// If neither STDOUT nor STDERR is a tty, returns `None`
#[cfg(not(target_os = "redox"))]
pub fn terminal_size() -> Option<(Width, Height)> {
    if unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 } {
        terminal_size_fd(libc::STDOUT_FILENO)
    } else if unsafe { libc::isatty(libc::STDERR_FILENO) == 1 }{
        terminal_size_fd(libc::STDERR_FILENO)
    } else {
        None
    }
}

/// Return string that move the cursor `n` lines up.
pub fn move_cursor_up(n: usize) -> String {
    assert!(n < 0x10000);
    format!("{}", termion::cursor::Up(n as u16))
}

#[cfg(not(target_os = "redox"))]
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
