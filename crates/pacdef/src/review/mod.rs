mod datastructures;
mod strategy;

use std::io::{stdin, stdout, Write};

use anyhow::Result;

use crate::prelude::*;
use crate::ui::{get_user_confirmation, read_single_char_from_terminal};

use self::datastructures::{ContinueWithReview, ReviewAction, ReviewIntention, ReviewsPerBackend};
use self::strategy::Strategy;

pub fn review(todo_per_backend: ToDoPerBackend, groups: &Groups) -> Result<()> {
    let mut reviews = ReviewsPerBackend::new();

    if todo_per_backend.nothing_to_do_for_all_backends() {
        println!("nothing to do");
        return Ok(());
    }

    'outer: for (backend, packages) in todo_per_backend {
        let mut actions = vec![];
        for package in packages {
            println!("{}: {package}", backend.backend_info().section);
            match get_action_for_package(package, groups, &mut actions, &backend)? {
                ContinueWithReview::Yes => continue,
                ContinueWithReview::No => return Ok(()),
                ContinueWithReview::NoAndApply => {
                    reviews.push((backend, actions));
                    break 'outer;
                }
            }
        }
        reviews.push((backend, actions));
    }

    if reviews.nothing_to_do() {
        println!("nothing to do");
        return Ok(());
    }

    let strategies: Vec<Strategy> = reviews.into_strategies();

    println!();
    let mut iter = strategies.iter().peekable();

    while let Some(strategy) = iter.next() {
        strategy.show();

        if iter.peek().is_some() {
            println!();
        }
    }

    println!();
    if !get_user_confirmation()? {
        return Ok(());
    }

    for strategy in strategies {
        strategy.execute()?;
    }

    Ok(())
}

fn get_action_for_package(
    package: Package,
    groups: &Groups,
    reviews: &mut Vec<ReviewAction>,
    backend: &dyn Backend,
) -> Result<ContinueWithReview> {
    loop {
        match ask_user_action_for_package(backend.supports_as_dependency())? {
            ReviewIntention::AsDependency => {
                assert!(
                    backend.supports_as_dependency(),
                    "backend does not support dependencies"
                );
                reviews.push(ReviewAction::AsDependency(package));
                break;
            }
            ReviewIntention::AssignGroup => {
                if let Ok(Some(group)) = ask_group(groups) {
                    reviews.push(ReviewAction::AssignGroup(package, group.clone()));
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
            ReviewIntention::Apply => return Ok(ContinueWithReview::NoAndApply),
        }
    }
    Ok(ContinueWithReview::Yes)
}

/// Ask the user for the desired action, and return the associated
/// [`ReviewIntention`]. The query depends on the capabilities of the backend.
///
/// # Errors
///
/// This function will return an error if stdin or stdout cannot be accessed.
fn ask_user_action_for_package(supports_as_dependency: bool) -> Result<ReviewIntention> {
    print_query(supports_as_dependency)?;

    match read_single_char_from_terminal()?.to_ascii_lowercase() {
        'a' if supports_as_dependency => Ok(ReviewIntention::AsDependency),
        'd' => Ok(ReviewIntention::Delete),
        'g' => Ok(ReviewIntention::AssignGroup),
        'i' => Ok(ReviewIntention::Info),
        'q' => Ok(ReviewIntention::Quit),
        's' => Ok(ReviewIntention::Skip),
        'p' => Ok(ReviewIntention::Apply),
        _ => Ok(ReviewIntention::Invalid),
    }
}

/// Print a space-terminated string that asks the user for the desired action.
/// The items of the string depend on whether the backend supports dependent
/// packages.
///
/// # Errors
///
/// This function will return an error if stdout cannot be flushed.
fn print_query(supports_as_dependency: bool) -> Result<()> {
    let mut query = String::from("assign to (g)roup, (d)elete, (s)kip, (i)nfo, ");

    if supports_as_dependency {
        query.push_str("(a)s dependency, ");
    }

    query.push_str("a(p)ply, (q)uit? ");

    print!("{query}");
    stdout().lock().flush()?;
    Ok(())
}

fn print_enumerated_groups(groups: &Groups) {
    let number_digits = get_amount_of_digits_for_number(groups.len());

    for (i, group) in groups.iter().enumerate() {
        println!("{i:>number_digits$}: {}", group.name);
    }
}

fn get_amount_of_digits_for_number(number: usize) -> usize {
    number.to_string().len()
}

fn ask_group(groups: &Groups) -> Result<Option<&Group>> {
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
        Ok(groups.iter().nth(idx))
    } else {
        Ok(None)
    }
}
