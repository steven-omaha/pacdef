use std::process::exit;

use std::io::Write::std::io;

use std;

pub(crate) fn get_user_confirmation() {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().unwrap();
    let reply = std::io::stdin().lock().lines().next().unwrap().unwrap();
    if !(reply.is_empty() || reply.to_lowercase().contains('y')) {
        exit(0)
    }
}
