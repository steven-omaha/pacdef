use anyhow::Result;

use pacdef::{args, Group, Pacdef};

fn main() -> Result<()> {
    let args = args::get_matched_arguments();
    let groups = Group::load_from_dir();
    let pacdef = Pacdef::new(args, groups);
    pacdef.run_action_from_arg();
    Ok(())
}
