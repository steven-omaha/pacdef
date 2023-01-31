use std::collections::HashSet;
use std::io::{self, stdin, stdout, Read, Write};
use std::rc::Rc;

use anyhow::{anyhow, bail, Context, Result};
use termios::*;

use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::grouping::{Group, Package, Section};
use crate::ui::get_user_confirmation;

#[derive(Debug)]
enum ReviewAction {
    AsDependency,
    AssignGroup,
    Delete,
    Info,
    Invalid,
    Skip,
    Quit,
}

#[derive(Debug)]
struct Reviews<'a> {
    pub as_dependency: Vec<(Rc<Box<dyn Backend>>, Package)>,
    pub assign: Vec<(Rc<Box<dyn Backend>>, Package, &'a Group)>,
    pub delete: Vec<(Rc<Box<dyn Backend>>, Package)>,
}

impl<'a> Reviews<'a> {
    fn new() -> Self {
        Self {
            as_dependency: vec![],
            assign: vec![],
            delete: vec![],
        }
    }

    fn run_strategy(self) -> Result<()> {
        todo!()
    }

    fn print_strategy(&self) {
        println!("delete:");
        for (backend, package) in &self.delete {
            println!("{} {}", backend.get_section(), package.name);
        }

        println!("assign:");
        for (backend, package, group) in &self.assign {
            println!("{} {} {}", backend.get_section(), package.name, group.name);
        }

        println!("as dependency:");
        for (backend, package) in &self.as_dependency {
            println!("{} {}", backend.get_section(), package.name);
        }
    }
}

pub(crate) fn review(todo_per_backend: ToDoPerBackend, groups: HashSet<Group>) -> Result<()> {
    dbg!(&todo_per_backend);

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
            get_action_for_package(package, &groups, &mut reviews, &backend)?;
        }
    }

    reviews.print_strategy();

    if !get_user_confirmation() {
        return Ok(());
    }

    reviews.run_strategy()
}

fn get_action_for_package<'a>(
    package: Package,
    groups: &'a [Group],
    reviews: &mut Reviews<'a>,
    backend: &Rc<Box<dyn Backend>>,
) -> Result<()> {
    loop {
        match ask_user_action_for_package()? {
            ReviewAction::AsDependency => {
                reviews.as_dependency.push((backend.clone(), package));
                break;
            }
            ReviewAction::AssignGroup => {
                if let Ok(Some(group)) = ask_group(groups) {
                    reviews.assign.push((backend.clone(), package, group));
                    break;
                };
            }
            ReviewAction::Delete => {
                reviews.delete.push((backend.clone(), package));
                break;
            }
            ReviewAction::Info => {
                backend.show_package_info(&package)?;
            }
            ReviewAction::Invalid => (),
            ReviewAction::Skip => break,
            // TODO custom return type
            ReviewAction::Quit => bail!("user wants to quit"),
        }
    }
    Ok(())
}

fn ask_user_action_for_package() -> Result<ReviewAction> {
    print!("assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? ");
    stdout().lock().flush()?;
    match read_single_char_from_terminal()? {
        'a' => Ok(ReviewAction::AsDependency),
        'd' => Ok(ReviewAction::Delete),
        'g' => Ok(ReviewAction::AssignGroup),
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
