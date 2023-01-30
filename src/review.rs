use std::collections::HashSet;
use std::io::{self, stdin, Read};
use std::rc::Rc;

use anyhow::{anyhow, bail, Context, Result};
use termios::*;

use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::grouping::{Group, Package, Section};

#[derive(Debug)]
enum ReviewAction {
    AsDependency,
    AssignGroupBackend,
    Delete,
    Info,
    Invalid,
    Skip,
    Quit,
}

struct Reviews<'a> {
    pub delete: Vec<(Rc<Box<dyn Backend>>, Package)>,
    pub assign: Vec<(Rc<Box<dyn Backend>>, Package, &'a Group, &'a Section)>,
}

impl<'a> Reviews<'a> {
    fn new() -> Self {
        Self {
            delete: vec![],
            assign: vec![],
        }
    }
}

pub(crate) fn review(todo_per_backend: ToDoPerBackend, groups: HashSet<Group>) -> Result<()> {
    let mut reviews = Reviews::new();
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_unstable();

    if todo_per_backend.nothing_to_do_for_all_backends() {
        println!("nothing to do");
        return Ok(());
    }

    for (backend, packages) in todo_per_backend.into_iter() {
        let backend = Rc::new(backend);
        for package in packages {
            println!("{}: {package}", backend.get_section());
            get_action_for_package(package, &mut groups, &mut reviews, &backend)?;
        }
    }

    todo!()
}

fn get_action_for_package(
    package: Package,
    groups: &mut [Group],
    reviews: &mut Reviews,
    backend: &Rc<Box<dyn Backend>>,
) -> Result<()> {
    loop {
        match ask_user_action_for_package()? {
            ReviewAction::AsDependency => todo!(),
            ReviewAction::AssignGroupBackend => {
                if let Ok(()) = assign_package_to_group(&package, groups) {
                    break;
                };
            }
            ReviewAction::Delete => {
                reviews.delete.push((backend.clone(), package));
                break;
            }
            ReviewAction::Info => backend.show_package_info(&package)?,
            ReviewAction::Invalid => (),
            ReviewAction::Skip => break,
            ReviewAction::Quit => bail!("user wants to quit"),
        }
    }
    Ok(())
}

fn ask_user_group(groups: &[Group]) -> Result<Option<GroupReply>> {
    let group = match ask_group(groups)? {
        Some(group) => group,
        None => return Ok(None),
    };

    Ok(Some(GroupReply::Existing(group)))
}

enum GroupReply<'a> {
    Existing(&'a Group),
    New,
}

fn ask_new_section_name() -> Result<String> {
    print!("new section name: ");
    let reply = stdin().lines().next().context("reading line from stdin")?;
    reply.map_err(|e| anyhow!(e))
}

fn print_enumerated_sections(sections: &[Section]) {
    for (i, section) in sections.iter().enumerate() {
        println!("{i}: {}", section.name);
    }
    println!("{}: [new]", sections.len());
}

fn ask_user_action_for_package() -> Result<ReviewAction> {
    print!("assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? ");
    match read_single_char_from_terminal()? {
        'a' => Ok(ReviewAction::AsDependency),
        'd' => Ok(ReviewAction::Delete),
        'g' => Ok(ReviewAction::AssignGroupBackend),
        'i' => Ok(ReviewAction::Info),
        'q' => Ok(ReviewAction::Quit),
        's' => Ok(ReviewAction::Skip),
        _ => Ok(ReviewAction::Invalid),
    }
}

fn read_single_char_from_terminal() -> Result<char> {
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

fn print_enumerated_groups(groups: &[Group]) {
    for (i, group) in groups.iter().enumerate() {
        println!("{i}: {}", group.name);
    }
}

fn ask_group(groups: &[Group]) -> Result<Option<&Group>> {
    print_enumerated_groups(groups);
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    let reply = buf.trim();

    let idx: usize = if let Ok(idx) = reply.parse() {
        idx
    } else {
        return Ok(None);
    };

    if idx < groups.len() {
        Ok(Some(&groups[idx]))
    } else {
        Ok(None)
    }
}

fn assign_package_to_group(package: &Package, groups: &mut [Group]) -> Result<()> {
    let reply = ask_user_group(groups)?;
    match reply {
        Some(GroupReply::Existing(group)) => todo!(),
        Some(GroupReply::New) => todo!(),
        None => todo!(),
    }

    todo!()
}
