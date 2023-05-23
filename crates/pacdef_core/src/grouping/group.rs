use std::collections::HashSet;
use std::fmt::Display;
use std::fs::{create_dir, read_to_string, File};
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::path::get_relative_path;

use super::{Package, Section};

/// Representation of a group file, composed of a name (file name from which it
/// was read), the sections in the file, and the absolute path of the original
/// file.
#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) sections: HashSet<Section>,
    pub(crate) path: PathBuf,
}

impl Group {
    /// Load all group files from the pacdef group dir by traversing through the group dir.
    ///
    /// This method will print a warning if `warn_not_symlinks` is true and a group
    /// file is not a symlink.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the files under `group_dir` cannot
    /// be accessed.
    pub fn load(group_dir: &Path, warn_not_symlinks: bool) -> Result<HashSet<Self>> {
        let mut result = HashSet::new();

        if !group_dir.is_dir() {
            // we only need to create the innermost dir. The rest was already created from when
            // we loaded the config
            create_dir(group_dir).context("group dir does not exist, creating")?;
        }

        for entry in WalkDir::new(group_dir).follow_links(true).min_depth(1) {
            let file = entry?;
            let path = file.path();

            if path.is_dir() {
                continue;
            }

            if warn_not_symlinks && !path.is_symlink() {
                // TODO is there an efficient way to make sure *any* of the elements in the path
                // is a symlink?
                eprintln!(
                    "WARNING: group file {} is not a symlink",
                    path.to_string_lossy()
                );
            }

            let group = Self::try_from(path, group_dir)
                .with_context(|| format!("reading group file {path:?}"))?;

            result.insert(group);
        }

        Ok(result)
    }
}

impl PartialOrd for Group {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(core::cmp::Ordering::Equal) => None,
            ord => ord,
        }
    }
}

impl Ord for Group {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Hash for Group {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Group {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Group {
    /// Load the group from `path`. Determine the name from the path relative to the
    /// `group_dir`.
    ///
    /// # Warnings
    ///
    /// This function will print a warning if any section in the group file cannot
    /// be processed, or the file contains no sections.
    ///
    /// # Errors
    ///
    /// This function will return an error if the group file cannot be read.
    fn try_from<P>(path: P, group_dir: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let content = read_to_string(path).context("reading file content")?;

        let name = extract_group_name(path, group_dir.as_ref());

        let mut lines = content.lines().peekable();
        let mut sections = HashSet::new();

        while lines.peek().is_some() {
            let result = Section::try_from_lines(&mut lines).context("reading section");
            match result {
                Ok(section) => {
                    sections.insert(section);
                }
                Err(e) => {
                    let err = e.root_cause();
                    eprintln!("WARNING: could not process a section under group '{name}': {err}");
                }
            }
        }

        if sections.is_empty() {
            eprintln!("WARNING: no sections found in group '{name}'");
        }

        let path = path.into();

        Ok(Self {
            name,
            sections,
            path,
        })
    }

    /// Add the new `packages` to the group file under the section `section_header`. If
    /// the section header does not yet exist, it is created. The packages are written
    /// in the provided order immediately after the header.
    ///
    /// # Errors
    ///
    /// This function returns an error if the group file cannot be read, or if the
    /// file cannot be written to.
    pub(crate) fn save_packages(&self, section_header: &str, packages: &[Package]) -> Result<()> {
        let mut content = read_to_string(&self.path)
            .with_context(|| format!("reading existing file contents from {:?}", &self.path))?;

        if content.contains(section_header) {
            write_packages_to_existing_section(&mut content, section_header, packages)
                .context("existing section")?;
        } else {
            add_new_section_with_packages(&mut content, section_header, packages);
        }

        let mut file = File::create(&self.path)
            .with_context(|| format!("creating descriptor to output file {:?}", &self.path))?;

        write!(file, "{content}").with_context(|| format!("writing file {:?}", &self.path))
    }
}

/// Extract the group name from its path relative to the group path.
/// All subdirectories are concatenated using `'/'`.
///
/// # Example
///
/// If the group dir is `~/.config/pacdef/groups`, and the group file is
/// `~/.config/pacdef/groups/generic/base`, then the group name is
/// `"generic/base"`.
///
/// # Panics
///
/// Panics if `path` and `group_path` are identical.
fn extract_group_name(path: &Path, group_path: &Path) -> String {
    get_relative_path(path, group_path)
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .reduce(|mut a, b| {
            a.push('/');
            a.push_str(&b);
            a
        })
        .expect("must have at least one element")
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sections: Vec<_> = self.sections.iter().collect();
        sections.sort_unstable();

        let mut iter = sections.into_iter().peekable();

        while let Some(section) = iter.next() {
            section.fmt(f)?;
            if iter.peek().is_some() {
                f.write_str("\n\n")?;
            }
        }
        Ok(())
    }
}

/// Add some packages to an existing section in the content of a group file.
///
/// # Errors
///
/// This function will return an error if the header cannot be found in
/// the file content.
fn write_packages_to_existing_section(
    group_file_content: &mut String,
    section_header: &str,
    packages: &[Package],
) -> Result<()> {
    let idx_of_first_package_line_in_section =
        find_first_package_line_in_section(group_file_content, section_header)?;

    let after = group_file_content.split_off(idx_of_first_package_line_in_section);

    for p in packages {
        group_file_content.push_str(&format!("{p}\n"));
    }

    group_file_content.push_str(&after);
    Ok(())
}

/// Find the index to the first line in `group_file_content` after the
/// given `section_header`.
///
/// # Errors
///
/// This function will return an error if the `section_header` does not
/// exist in `group_file_content`, or if the line containing the
/// `section_header` is not newline-terminated.
fn find_first_package_line_in_section(
    group_file_content: &str,
    section_header: &str,
) -> Result<usize> {
    let section_start = group_file_content
        .find(section_header)
        .context("finding first package after section header")?;

    let distance_to_next_newline = group_file_content[section_start..]
        .find('\n')
        .context("getting next newline")?;

    Ok(section_start + distance_to_next_newline + 1) // + 1 to be after the newline
}

/// Append a new section with some packages to the content of a group file.
fn add_new_section_with_packages(
    group_file_content: &mut String,
    section_header: &str,
    packages: &[Package],
) {
    group_file_content.push('\n');
    group_file_content.push_str(section_header);
    group_file_content.push('\n');
    for p in packages {
        group_file_content.push_str(&format!("{p}\n"));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn extract_group_name() {
        let path = PathBuf::from("/a/b/c/d/e");
        let group_path = PathBuf::from("/a/b/c");
        let expected = String::from("d/e");

        let result = super::extract_group_name(&path, &group_path);
        assert_eq!(result, expected);
    }
}
