use std::collections::HashSet;
use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::{bail, Result};
use regex::Regex;

use crate::grouping::{Group, Package, Section};

/// Find all packages in all groups whose name match the regex from the
/// command-line arguments. Print the name of the packages per group and
/// section.
///
/// # Errors
///
/// This function will return an error if
/// - an invalid regex was provided, or
/// - no matching packages could be found.
pub fn search_packages(regex_str: &str, groups: &HashSet<Group>) -> Result<()> {
    if groups.is_empty() {
        eprintln!("WARNING: no group files found");
        bail!(crate::errors::Error::NoPackagesFound);
    }

    let re = Regex::new(regex_str)?;

    let mut vec = vec![];

    for group in groups {
        for section in &group.sections {
            for package in &section.packages {
                if re.is_match(&package.name) {
                    vec.push((group, section, package));
                }
            }
        }
    }

    if vec.is_empty() {
        bail!(crate::errors::Error::NoPackagesFound);
    }

    print_triples(vec);

    Ok(())
}

fn print_triples(mut vec: Vec<(&Group, &Section, &Package)>) {
    vec.sort_unstable();

    let mut g0 = String::new();
    let mut s0 = String::new();

    let mut iter = vec.into_iter().peekable();

    while let Some((g, s, p)) = iter.next() {
        print_group_if_changed(g, &g0, &mut s0);
        print_section_if_changed(s, &s0);
        println!("{p}");
        save_group_and_section_name(&mut g0, g, &mut s0, s);
        print_separator_unless_exhausted(&mut iter, &g0);
    }
}

fn save_group_and_section_name(g0: &mut String, g: &Group, s0: &mut String, s: &Section) {
    g0.clone_from(&g.name);
    s0.clone_from(&s.name);
}

fn print_separator_unless_exhausted(
    iter: &mut Peekable<IntoIter<(&Group, &Section, &Package)>>,
    g0: &String,
) {
    if let Some((g, _, _)) = iter.peek() {
        if g.name != *g0 {
            println!();
        }
    }
}

fn print_section_if_changed(current: &Section, previous_name: &String) {
    if current.name != *previous_name {
        println!("[{}]", current.name);
    }
}

fn print_group_if_changed(
    current_group: &Group,
    previous_group_name: &String,
    previous_section_name: &mut String,
) {
    if current_group.name != *previous_group_name {
        println!("{}", current_group.name);
        for _ in 0..current_group.name.len() {
            print!("-");
        }
        println!();
        previous_section_name.clear();
    }
}
