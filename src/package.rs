use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Write;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::io::Lines;

#[derive(Debug, Eq, PartialOrd, Ord)]
pub(crate) struct Package {
    pub(crate) name: String,
    repo: Option<String>,
}

impl From<String> for Package {
    fn from(mut s: String) -> Self {
        s.remove_comment();
        s.remove_whitespace();
        let (name, repo) = Self::split_into_name_and_repo(s);
        Self { name, repo }
    }
}

impl Package {
    pub(crate) fn from_lines(lines: Lines<BufReader<File>>) -> HashSet<Self> {
        lines
            .into_iter()
            .map(|l| Package::from(l.unwrap()))
            .collect()
    }

    fn split_into_name_and_repo(mut s: String) -> (String, Option<String>) {
        match s.find('/') {
            None => (s, None),
            Some(pos) => {
                let mut name = s.split_off(pos);
                name = name.split_off(1);
                (name, Some(s))
            }
        }
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

trait Whitespace {
    fn remove_comment(&mut self) {}
    fn remove_whitespace(&mut self) {}
}

impl Whitespace for String {
    fn remove_comment(&mut self) {
        match self.find('#') {
            None => (),
            Some(idx) => self.truncate(idx),
        }
    }

    fn remove_whitespace(&mut self) {
        match self.find(char::is_whitespace) {
            None => (),
            Some(idx) => self.truncate(idx),
        }
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
        let (name, repo) = Package::split_into_name_and_repo(x);
        assert_eq!(name, "name");
        assert_eq!(repo, Some("repo".to_string()));

        let x = "something".to_string();
        let (name, repo) = super::Package::split_into_name_and_repo(x);
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
