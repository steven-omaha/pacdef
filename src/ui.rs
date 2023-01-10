use std::io::{BufRead, Write};

#[must_use]
pub(crate) fn get_user_confirmation() -> bool {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().unwrap();
    let reply = std::io::stdin().lock().lines().next().unwrap().unwrap();
    reply.trim().is_empty() || reply.to_lowercase().contains('y')
}
