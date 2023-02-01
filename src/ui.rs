use std::io::{self, BufRead, Read, Write};

use anyhow::Result;
use termios::*;

#[must_use]
pub(crate) fn get_user_confirmation() -> bool {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().unwrap();
    let reply = std::io::stdin().lock().lines().next().unwrap().unwrap();
    reply.trim().is_empty() || reply.to_lowercase().contains('y')
}

pub(crate) fn read_single_char_from_terminal() -> Result<char> {
    let fd = 0; // 0 is the file descriptor for stdin
    let termios = Termios::from_fd(fd)?;
    let mut new_termios = termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[VMIN] = 1;
    new_termios.c_cc[VTIME] = 0;
    tcsetattr(fd, TCSANOW, &new_termios).unwrap();

    let mut input = [0u8; 1];
    io::stdin().read_exact(&mut input[..]).unwrap();
    let result = input[0] as char;
    println!("{result}");

    tcsetattr(fd, TCSANOW, &termios).unwrap(); // restore previous settings
    Ok(result)
}
