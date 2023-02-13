use std::collections::HashSet;
use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use regex::Regex;

use crate::grouping::{Group, Package, Section};

pub const NO_PACKAGES_FOUND: &str = "no packages matching query";

pub(crate) fn search_packages(args: &ArgMatches, groups: &HashSet<Group>) -> Result<()> {
    let search_string = args
        .get_one::<String>("string")
        .context("getting search string from arg")?;

    let re = Regex::new(search_string)?;

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
        bail!(NO_PACKAGES_FOUND)
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
    *g0 = g.name.clone();
    *s0 = s.name.clone();
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

fn print_section_if_changed(s: &Section, s0: &String) {
    if s.name != *s0 {
        println!("[{}]", s.name);
    }
}

fn print_group_if_changed(g: &Group, g0: &String, s0: &mut String) {
    if g.name != *g0 {
        println!("{}", g.name);
        for _ in 0..g.name.len() {
            print!("-");
        }
        println!();
        s0.clear();
    }
}
