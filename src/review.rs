use std::collections::HashSet;
use std::io::{self, stdin, Read};
use std::rc::Rc;

use anyhow::{anyhow, bail, Context, Result};
use termios::*;

use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::grouping::{Group, Package, Section};
use crate::ui::get_user_confirmation;

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

struct Reviews {
    pub delete: Vec<Delete>,
    pub assign: Vec<Assign>,
    pub as_dependency: Vec<AsDependency>,
}

struct AsDependency {
    backend: Rc<Box<dyn Backend>>,
    package: Package,
}

impl AsDependency {
    fn new(backend: Rc<Box<dyn Backend>>, package: Package) -> Self {
        Self { backend, package }
    }
}

struct Assign {
    backend: Rc<Box<dyn Backend>>,
    package: Package,
    group: Rc<Group>,
}

impl Assign {
    fn new(backend: Rc<Box<dyn Backend>>, package: Package, group: Rc<Group>) -> Self {
        Self {
            backend,
            package,
            group,
        }
    }
}

struct Delete {
    items: Vec<>
    backend: Rc<Box<dyn Backend>>,
    package: Package,
}

impl Delete {
    fn new(backend: Rc<Box<dyn Backend>>, package: Package) -> Self {
        Self { backend, package }
    }
}

impl Reviews {
    fn new() -> Self {
        Self {
            delete: vec![],
            assign: vec![],
            as_dependency: vec![],
        }
    }

    fn show_strategy(&mut self) {
        self.delete
            .sort_by_key(|d| (&d.backend.get_section(), &d.package));
        if !self.delete.is_empty() {
            println!("Will delete the following packages:");
            let mut iter = self.delete.iter().peekable();
            // while let Some(delete) = iter.next() {
            //     delete.
            // }
        }
    }

    fn execute(&self) -> Result<()> {
        todo!()
    }
}

pub(crate) fn review(todo_per_backend: ToDoPerBackend, groups: HashSet<Group>) -> Result<()> {
    let mut reviews = Reviews::new();
    let mut groups: Vec<_> = groups.into_iter().map(Rc::new).collect();
    groups.sort_unstable();

    if todo_per_backend.nothing_to_do_for_all_backends() {
        println!("nothing to do");
        return Ok(());
    }

    gather_reviews(todo_per_backend, groups, &mut reviews)?;

    reviews.show_strategy();

    if !get_user_confirmation() {
        return Ok(());
    }

    reviews.execute()
}

fn gather_reviews(
    todo_per_backend: ToDoPerBackend,
    groups: Vec<Rc<Group>>,
    reviews: &mut Reviews,
) -> Result<()> {
    for (backend, packages) in todo_per_backend.into_iter() {
        let backend = Rc::new(backend);
        for package in packages {
            println!("{}: {package}", backend.get_section());
            get_action_for_package(package, &groups, reviews, &backend)?;
        }
    }
    Ok(())
}

fn get_action_for_package(
    package: Package,
    groups: &[Rc<Group>],
    reviews: &mut Reviews,
    backend: &Rc<Box<dyn Backend>>,
) -> Result<()> {
    loop {
        match ask_user_action_for_package(backend)? {
            ReviewAction::AsDependency => {
                let as_dependency = AsDependency::new(backend.clone(), package);
                reviews.as_dependency.push(as_dependency);
                break;
            }
            ReviewAction::AssignGroupBackend => {
                if let Some(group) = assign_group(groups)? {
                    let assign = Assign::new(backend.clone(), package, group);
                    reviews.assign.push(assign);
                    break;
                };
            }
            ReviewAction::Delete => {
                let delete = Delete::new(backend.clone(), package);
                reviews.delete.push(delete);
                break;
            }
            ReviewAction::Info => backend.show_package_info(&package)?,
            ReviewAction::Invalid => (),
            ReviewAction::Skip => break,
            ReviewAction::Quit => bail!("user wants to quit"), // TODO requires an own error type?
        }
    }
    Ok(())
}

fn ask_user_action_for_package(backend: &Rc<Box<dyn Backend>>) -> Result<ReviewAction> {
    if backend.supports_assigning_packages_as_dependency() {
        ask_action_including_dependency()
    } else {
        ask_action_without_dependency()
    }
}

fn ask_action_without_dependency() -> Result<ReviewAction> {
    print!("assign to (g)roup, (d)elete, (s)kip, (i)nfo, (q)uit? ");
    match read_single_char_from_terminal()? {
        'd' => Ok(ReviewAction::Delete),
        'g' => Ok(ReviewAction::AssignGroupBackend),
        'i' => Ok(ReviewAction::Info),
        'q' => Ok(ReviewAction::Quit),
        's' => Ok(ReviewAction::Skip),
        _ => Ok(ReviewAction::Invalid),
    }
}

fn ask_action_including_dependency() -> Result<ReviewAction> {
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

fn print_enumerated_groups(groups: &[Rc<Group>]) {
    for (i, group) in groups.iter().enumerate() {
        println!("{i}: {}", group.name);
    }
}

fn ask_group(groups: &[Rc<Group>]) -> Result<Option<Rc<Group>>> {
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    let reply = buf.trim();

    let idx: usize = if let Ok(idx) = reply.parse() {
        idx
    } else {
        return Ok(None);
    };

    if idx < groups.len() {
        Ok(Some(groups[idx].clone()))
    } else {
        Ok(None)
    }
}

fn assign_group(groups: &[Rc<Group>]) -> Result<Option<Rc<Group>>> {
    print_enumerated_groups(groups);
    ask_group(groups)
}
