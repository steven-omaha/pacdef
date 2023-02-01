use std::io::{self, stdin, stdout, Read, Write};
use std::rc::Rc;

use anyhow::{bail, Result};
use termios::*;

use crate::backend::{Backend, ToDoPerBackend};
use crate::grouping::{Group, Package};
use crate::ui::get_user_confirmation;

#[derive(Debug)]
struct ReviewsPerBackend(Vec<(Box<dyn Backend>, Vec<ReviewAction>)>);

#[derive(Debug)]
enum ReviewIntention {
    AsDependency,
    AssignGroup,
    Delete,
    Info,
    Invalid,
    Skip,
    Quit,
}

#[derive(Debug, PartialEq)]
enum ReviewAction {
    AsDependency(Package),
    Delete(Package),
    AssignGroup(Package, Rc<Group>),
}

impl ReviewsPerBackend {
    fn new() -> Self {
        Self(vec![])
    }

    pub(crate) fn nothing_to_do(&self) -> bool {
        self.0.iter().all(|(_, vec)| vec.is_empty())
    }
}

pub(crate) fn review(
    todo_per_backend: ToDoPerBackend,
    groups: impl IntoIterator<Item = Group>,
) -> Result<()> {
    let mut reviews = ReviewsPerBackend::new();
    let mut groups: Vec<Rc<Group>> = groups.into_iter().map(Rc::new).collect();
    groups.sort_unstable();

    if todo_per_backend.nothing_to_do_for_all_backends() {
        println!("nothing to do");
        return Ok(());
    }

    for (backend, packages) in todo_per_backend.into_iter() {
        let mut actions = vec![];
        for package in packages {
            println!("{}: {package}", backend.get_section());
            get_action_for_package(package, &groups, &mut actions, &*backend)?;
        }
        reviews.0.push((backend, actions));
    }

    if reviews.nothing_to_do() {
        println!("nothing to do");
        return Ok(());
    }

    let strategies: Vec<Strategy> = reviews.into();

    for strat in &strategies {
        strat.show();
    }

    if !get_user_confirmation() {
        return Ok(());
    }

    for strat in strategies {
        strat.execute()?;
    }

    Ok(())
}

fn get_action_for_package(
    package: Package,
    groups: &[Rc<Group>],
    reviews: &mut Vec<ReviewAction>,
    backend: &dyn Backend,
) -> Result<()> {
    loop {
        match ask_user_action_for_package()? {
            ReviewIntention::AsDependency => {
                reviews.push(ReviewAction::AsDependency(package));
                break;
            }
            ReviewIntention::AssignGroup => {
                if let Ok(Some(group)) = ask_group(groups) {
                    reviews.push(ReviewAction::AssignGroup(package, group));
                    break;
                };
            }
            ReviewIntention::Delete => {
                reviews.push(ReviewAction::Delete(package));
                break;
            }
            ReviewIntention::Info => {
                backend.show_package_info(&package)?;
            }
            ReviewIntention::Invalid => (),
            ReviewIntention::Skip => break,
            // TODO custom return type
            ReviewIntention::Quit => bail!("user wants to quit"),
        }
    }
    Ok(())
}

fn ask_user_action_for_package() -> Result<ReviewIntention> {
    print!("assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? ");
    stdout().lock().flush()?;
    match read_single_char_from_terminal()? {
        'a' => Ok(ReviewIntention::AsDependency),
        'd' => Ok(ReviewIntention::Delete),
        'g' => Ok(ReviewIntention::AssignGroup),
        'i' => Ok(ReviewIntention::Info),
        'q' => Ok(ReviewIntention::Quit),
        's' => Ok(ReviewIntention::Skip),
        _ => Ok(ReviewIntention::Invalid),
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
        Ok(Some(groups[idx].clone()))
    } else {
        Ok(None)
    }
}
#[derive(Debug)]
struct Strategy {
    backend: Box<dyn Backend>,
    delete: Vec<Package>,
    as_dependency: Vec<Package>,
    assign_group: Vec<(Package, Rc<Group>)>,
}

impl Strategy {
    fn new(
        backend: Box<dyn Backend>,
        delete: Vec<Package>,
        as_dependency: Vec<Package>,
        assign_group: Vec<(Package, Rc<Group>)>,
    ) -> Self {
        Self {
            backend,
            delete,
            as_dependency,
            assign_group,
        }
    }

    fn execute(self) -> Result<()> {
        if !self.delete.is_empty() {
            self.backend.remove_packages(&self.delete)?;
        }

        if !self.as_dependency.is_empty() {
            self.backend.make_dependency(&self.as_dependency)?;
        }

        if !self.assign_group.is_empty() {
            self.backend.assign_group(self.assign_group);
        }

        Ok(())
    }

    fn show(&self) {
        if self.nothing_to_do() {
            return;
        }

        println!("[{}]", self.backend.get_section());

        if !self.delete.is_empty() {
            println!("delete:");
            for p in &self.delete {
                println!("  {p}");
            }
        }

        if !self.as_dependency.is_empty() {
            println!("as depdendency:");
            for p in &self.as_dependency {
                println!("  {p}");
            }
        }

        if !self.assign_group.is_empty() {
            println!("assign groups:");
            for (p, g) in &self.assign_group {
                println!("  {p} -> {}", g.name);
            }
        }
    }

    fn nothing_to_do(&self) -> bool {
        self.delete.is_empty() && self.as_dependency.is_empty() && self.assign_group.is_empty()
    }
}

impl From<ReviewsPerBackend> for Vec<Strategy> {
    fn from(reviews: ReviewsPerBackend) -> Self {
        let mut result = vec![];

        for (backend, actions) in reviews.0 {
            let (to_delete, assign_group, as_dependency) = divide_actions(actions);

            result.push(Strategy::new(
                backend,
                to_delete,
                as_dependency,
                assign_group,
            ));
        }

        result
    }
}

fn divide_actions(
    actions: Vec<ReviewAction>,
) -> (Vec<Package>, Vec<(Package, Rc<Group>)>, Vec<Package>) {
    let mut to_delete = vec![];
    let mut assign_group = vec![];
    let mut as_dependency = vec![];

    for action in actions {
        match action {
            ReviewAction::Delete(package) => to_delete.push(package),
            ReviewAction::AssignGroup(package, group) => assign_group.push((package, group)),
            ReviewAction::AsDependency(package) => as_dependency.push(package),
        }
    }

    (to_delete, assign_group, as_dependency)
}
