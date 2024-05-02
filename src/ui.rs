use std::io::Write;

use anyhow::{Context, Result};

pub fn get_user_confirmation() -> Result<bool> {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().context("flushing stdout")?;

    let mut reply = String::new();
    std::io::stdin()
        .read_line(&mut reply)
        .context("reading stdin")?;

    Ok(reply.trim().is_empty() || reply.to_lowercase().contains('y'))
}
