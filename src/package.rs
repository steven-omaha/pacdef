use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::hash::Hash;

#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Package {
    pub name: String,
    repo: Option<String>,
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        let trimmed = remove_all_but_package_name(s);

        let (name, repo) = Self::split_into_name_and_repo(trimmed);
        Self { name, repo }
    }
}

impl From<String> for Package {
    fn from(value: String) -> Self {
        Package::from(value.as_ref())
    }
}

fn remove_all_but_package_name(s: &str) -> &str {
    s.split('#') // remove comment
        .next()
        .expect("line contains something")
        .trim() // remove whitespace
}

impl Package {
    pub(crate) fn from_lines(
        lines: impl Iterator<Item = Result<String, std::io::Error>>,
    ) -> HashSet<Self> {
        lines
            .into_iter()
            .map(|l| Package::from(l.unwrap()))
            .collect()
    }

    fn split_into_name_and_repo(s: &str) -> (String, Option<String>) {
        let mut iter = s.split('/').rev();
        let name = iter.next().unwrap().to_string();
        let repo = iter.next().map(|s| s.to_string());
        (name, repo)
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && match &self.repo {
                None => true,
                Some(r) => match &other.repo {
                    None => true,
                    Some(r2) => r == r2,
                },
            }
    }
}

impl Hash for Package {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.repo {
            None => (),
            Some(repo) => {
                f.write_str(repo)?;
                f.write_char('/')?;
            }
        }
        f.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::Package;

    #[test]
    fn split_into_name_and_repo() {
        let x = "repo/name".to_string();
        let (name, repo) = Package::split_into_name_and_repo(&x);
        assert_eq!(name, "name");
        assert_eq!(repo, Some("repo".to_string()));

        let x = "something".to_string();
        let (name, repo) = super::Package::split_into_name_and_repo(&x);
        assert_eq!(name, "something");
        assert_eq!(repo, None);
    }

    #[test]
    fn from() {
        let x = "myrepo/somepackage  #  ".to_string();
        let p = Package::from(x);
        assert_eq!(p.name, "somepackage");
        assert_eq!(p.repo, Some("myrepo".to_string()));
    }
}
