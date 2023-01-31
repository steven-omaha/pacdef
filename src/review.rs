use std::io::{self, stdin, stdout, Read, Write};

use anyhow::{bail, Result};
use termios::*;

use crate::backend::{Backend, ToDoPerBackend};
use crate::grouping::{Group, Package};
use crate::ui::get_user_confirmation;

#[derive(Debug)]
struct ReviewsPerBackend<'a>(Vec<(Box<dyn Backend>, Vec<ReviewAction<'a>>)>);

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
enum ReviewAction<'a> {
    AsDependency(Package),
    Delete(Package),
    AssignGroup(Package, &'a Group),
}

impl<'a> ReviewsPerBackend<'a> {
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
    dbg!(&todo_per_backend);

    let mut reviews = ReviewsPerBackend::new();
    let mut groups: Vec<_> = groups.into_iter().collect();
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
        return Ok(());
    }

    let strategy: Vec<Strategy> = reviews.into();

    if !get_user_confirmation() {
        return Ok(());
    }

    strategy.execute()
}

fn get_action_for_package<'a>(
    package: Package,
    groups: &'a [Group],
    reviews: &mut Vec<ReviewAction<'a>>,
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
#[derive(Debug)]
struct Strategy {
    backend: Box<dyn Backend>,
    delete: Vec<Package>,
    as_dependency: Vec<Package>,
    assign_group: Vec<(Package, Group)>,
}

impl Strategy {
    fn new(
        backend: Box<dyn Backend>,
        delete: Vec<Package>,
        as_dependency: Vec<Package>,
        assign_group: Vec<(Package, Group)>,
    ) -> Self {
        Self {
            backend,
            delete,
            as_dependency,
            assign_group,
        }
    }

    fn get_assign_to_group<'a>(actions: &'a mut [ReviewAction<'a>]) -> Vec<(Package, Group)> {
        let mut result: Vec<_> = actions
            .iter()
            .filter_map(|action| {
                if let ReviewAction::AssignGroup(p, g) = action {
                    todo!()
                    // Some((*p, **g))
                } else {
                    None
                }
            })
            .collect();
        result.sort();
        result
    }

    fn get_make_dependency<'a>(actions: &'a mut [ReviewAction<'a>]) -> Vec<Package> {
        let mut result: Vec<_> = actions
            .iter()
            .filter_map(|action| {
                if let ReviewAction::AsDependency(p) = action {
                    todo!()
                    // Some(*p)
                } else {
                    None
                }
            })
            .collect();
        result.sort();
        result
    }

    fn get_to_delete<'a>(actions: &'a [ReviewAction<'a>]) -> Vec<&'a Package> {
        let mut result: Vec<_> = actions
            .iter()
            .filter_map(|action| {
                if let ReviewAction::Delete(p) = action {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        result.sort();
        result
    }

    // fn print_strategy(&self) {
    //     for (backend, actions) in &self.0 {
    //         if actions.is_empty() {
    //             continue;
    //         }

    //         println!("[{}]", backend.get_section());

    //         let to_delete = get_to_delete(actions);
    //         let as_dependency = get_make_dependency(actions);
    //         let assign_group = get_assign_to_group(actions);

    //         if !to_delete.is_empty() {
    //             println!("delete:");
    //             for p in &to_delete {
    //                 println!("  {p}");
    //             }
    //         }

    //         if !as_dependency.is_empty() {
    //             println!("as dependency:");
    //             for p in &as_dependency {
    //                 println!("  {p}");
    //             }
    //         }

    //         if !assign_group.is_empty() {
    //             println!("assign group:");
    //             for &(p, g) in &assign_group {
    //                 println!("  {p} -> {}", g.name);
    //             }
    //         }
    //     }
    // }
    //
    fn execute(self) -> Result<()> {
        todo!()
    }
}

impl<'a> From<ReviewsPerBackend<'a>> for Vec<Strategy> {
    fn from(reviews: ReviewsPerBackend) -> Self {
        let mut result = vec![];

        for (backend, actions) in reviews.0 {
            let (to_delete, assign_group, as_dependency) = divide_actions(actions);
        }

        result
    }
}
