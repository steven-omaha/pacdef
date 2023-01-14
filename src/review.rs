use std::collections::HashSet;
use std::io::{self, stdin, Read};
use std::rc::Rc;

use anyhow::{bail, Result};
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

impl Reviews {
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
            'inner: loop {
                match ask_user_action_for_package()? {
                    ReviewAction::AsDependency => todo!(),
                    ReviewAction::AssignGroupBackend => {
                        if let Some((group, section)) = ask_user_group_section(&package, &groups)? {
                            reviews
                                .assign
                                .push((backend.clone(), package, &group, &section));
                            break 'inner;
                        }
                    }
                    ReviewAction::Delete => {
                        reviews.delete.push((backend.clone(), package));
                        break 'inner;
                    }
                    ReviewAction::Info => backend.show_package_info(&package)?,
                    ReviewAction::Invalid => (),
                    ReviewAction::Skip => break 'inner,
                    ReviewAction::Quit => bail!("user wants to quit"),
                }
            }
        }
    }

    todo!()
}

fn ask_user_group_section<'a>(
    package: &'a Package,
    groups: &'a [Group],
) -> Result<Option<(&'a Group, &'a Section)>> {
    if let Some(group) = ask_group(groups)? {
        if let Some(section) = ask_section(section)? {
            return Ok(Some((group, section)));
        }
    }
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
