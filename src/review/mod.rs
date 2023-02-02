mod datastructures;
mod strategy;

use std::io::{stdin, stdout, Write};
use std::rc::Rc;

use anyhow::Result;

use crate::backend::{Backend, ToDoPerBackend};
use crate::ui::{get_user_confirmation, read_single_char_from_terminal};
use crate::{Group, Package};

use self::datastructures::{ContinueWithReview, ReviewAction, ReviewIntention, ReviewsPerBackend};
use self::strategy::Strategy;

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
            match get_action_for_package(package, &groups, &mut actions, &*backend)? {
                ContinueWithReview::Yes => continue,
                ContinueWithReview::No => return Ok(()),
            }
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
) -> Result<ContinueWithReview> {
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
            ReviewIntention::Quit => return Ok(ContinueWithReview::No),
        }
    }
    Ok(ContinueWithReview::Yes)
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
