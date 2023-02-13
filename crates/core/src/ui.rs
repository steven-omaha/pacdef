use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use termios::*;

pub fn get_user_confirmation() -> Result<bool> {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().context("flushing stdout")?;

    let mut reply = String::new();
    std::io::stdin()
        .read_line(&mut reply)
        .context("reading stdin")?;

    Ok(reply.trim().is_empty() || reply.to_lowercase().contains('y'))
}

pub fn read_single_char_from_terminal() -> Result<char> {
    // 0 is the file descriptor for stdin
    let fd = 0;
    let termios = Termios::from_fd(fd).context("getting stdin fd")?;
    let mut new_termios = termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[VMIN] = 1;
    new_termios.c_cc[VTIME] = 0;
    tcsetattr(fd, TCSANOW, &new_termios).context("setting terminal mode")?;

    let mut input = [0u8; 1];
    io::stdin()
        .read_exact(&mut input[..])
        .context("reading one byte from stdin")?;
    let result: char = input[0]
        .try_into()
        .context("reading a single byte from stdin")?;
    // stdin is not echoed automatically in this terminal mode
    println!("{result}");

    // restore previous settings
    tcsetattr(fd, TCSANOW, &termios).context("restoring terminal mode")?;

    Ok(result)
}
