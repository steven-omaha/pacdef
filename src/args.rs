use clap::Command;
pub(crate) fn get_arg_parser() -> Command<'static> {
    let result = Command::new("pacdef")
        .about("declarative package manager for Arch Linux")
        .version("1.0.0-alpha")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("sync").about("install packages from all imported groups"));
    result
}
