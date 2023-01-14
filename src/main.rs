use anyhow::{Context, Result};

use pacdef::{args, Config, Group, Pacdef};

fn main() -> Result<()> {
    let args = args::get();
    let config = Config::load().context("loading config file")?;
    let groups = Group::load(&config).context("loading groups")?;
    let pacdef = Pacdef::new(args, config, groups);
    pacdef.run_action_from_arg().context("running action")
}
