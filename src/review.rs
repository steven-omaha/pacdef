use std::collections::{HashMap, HashSet};
use std::io::{self, stdin, stdout, Read, Write};
use std::rc::Rc;

use anyhow::{anyhow, bail, Context, Result};
use termios::*;

use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::grouping::{Group, Package, Section};
use crate::ui::get_user_confirmation;

#[derive(Debug)]
struct BackendVectorMap<I>(Vec<(Rc<Box<dyn Backend>>, Vec<I>)>);

impl<I> BackendVectorMap<I> {
    fn new() -> Self {
        Self(vec![])
    }

    fn get(&self, backend: &Rc<Box<dyn Backend>>) -> Option<&[I]> {
        for (b, value) in &self.0 {
            if b.get_section() == backend.get_section() {
                return Some(value);
            }
        }
        return None;
    }

    fn get_mut(&mut self, backend: &Rc<Box<dyn Backend>>) -> Option<&mut [I]> {
        for (b, value) in &mut self.0 {
            if b.get_section() == backend.get_section() {
                return Some(value);
            }
        }
        return None;
    }

    fn push(&mut self, val: I) {
        todo!()
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, packages)| packages.is_empty())
    }

    fn iter(&self) -> impl Iterator<Item = &I> {
        self.0.iter().flat_map(|(_, items)| items)
    }
}

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
    pub as_dependency: BackendVectorMap<Package>,
    pub assign: BackendVectorMap<(Package, &'a Group)>,
    pub delete: BackendVectorMap<Package>,
}

impl<'a> Reviews<'a> {
    fn new() -> Self {
        Self {
            as_dependency: BackendVectorMap::new(),
            assign: BackendVectorMap::new(),
            delete: BackendVectorMap::new(),
        }
    }

    // order will be: per backend, first assign, then delete, then as dependency
    fn run_strategy(self) -> Result<()> {
        todo!()
    }

    fn print_strategy(&self) {
        println!("delete:");
        if !self.delete.is_empty() {
            for (backend, package) in self.delete.iter() {
                println!("{} {}", backend.get_section(), package.name);
            }
        }

        if !self.assign.is_empty() {
            println!("assign:");
            for (backend, package, group) in &self.assign {
                println!("{} {} {}", backend.get_section(), package.name, group.name);
            }
        }

        if !self.as_dependency.is_empty() {
            println!("as dependency:");
            for (backend, package) in &self.as_dependency {
                println!("{} {}", backend.get_section(), package.name);
            }
        }
    }

    pub(crate) fn nothing_to_do(&self) -> bool {
        self.as_dependency.is_empty() && self.assign.is_empty() && self.delete.is_empty()
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

    if reviews.nothing_to_do() {
        return Ok(());
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
                // reviews.as_dependency.insert(_);
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
