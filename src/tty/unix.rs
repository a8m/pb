use super::{Height, Width};

// We need to convert from c_int to c_ulong at least on DragonFly and FreeBSD.
#[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
fn ioctl_conv<T: Into<libc::c_ulong>>(v: T) -> libc::c_ulong {
    v.into()
}

// No-op on any other operating system.
#[cfg(not(any(target_os = "dragonfly", target_os = "freebsd")))]
fn ioctl_conv<T: Copy>(v: T) -> T {
    v
}

/// Returns the size of the terminal, if available.
///
/// If STDOUT is not a tty, returns `None`
pub fn terminal_size() -> Option<(Width, Height)> {
    use libc::{ioctl, isatty, winsize, STDOUT_FILENO, TIOCGWINSZ};
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
        ioctl(STDOUT_FILENO, ioctl_conv(TIOCGWINSZ), &mut winsize);
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

/// Return string that move the cursor `n` lines up.
pub fn move_cursor_up(n: usize) -> String {
    format!("\x1B[{}A", n)
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
