use std::{
    collections::HashSet,
    fmt::{Display, Write},
    hash::Hash,
    iter::Peekable,
};

use anyhow::{ensure, Context, Result};

use super::Package;

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub packages: HashSet<Package>,
}

impl Section {
    pub(crate) fn new(name: String, packages: HashSet<Package>) -> Self {
        Self { name, packages }
    }

    pub(crate) fn try_from_lines<'a>(
        iter: &mut Peekable<(impl Iterator<Item = &'a str> + std::fmt::Debug)>,
    ) -> Result<Self> {
        let name = iter
            .find(|line| line.starts_with('['))
            .context("finding beginning of next section")?
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();

        let mut packages = HashSet::new();
        // `while let` chains are unstable, unfortunately
        while iter.peek().is_some()
            && !iter
                .peek()
                .expect("we checked this is some")
                .starts_with('[')
        {
            if let Some(package) = Package::try_from(iter.next().expect("we checked this is some"))
            {
                packages.insert(package);
            }
        }

        ensure!(!packages.is_empty());

        Ok(Self::new(name, packages))
    }
}

impl Hash for Section {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Section {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Section {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for Section {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Section {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}]\n", &self.name))?;

        let mut packages: Vec<_> = self.packages.iter().collect();
        packages.sort_unstable();

        let mut iter = packages.iter().peekable();

        while let Some(package) = iter.next() {
            package.fmt(f)?;
            if iter.peek().is_some() {
                f.write_char('\n')?;
            }
        }

        Ok(())
    }
}
