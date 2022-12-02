use anyhow::{Context, Result};

use pacdef::{args, Group, Pacdef};

fn main() -> Result<()> {
    let args = args::get_args();
    let groups = Group::load_from_dir();
    let pacdef = Pacdef::new(args, groups);
    pacdef.run_action_from_arg().context("running action")
}
