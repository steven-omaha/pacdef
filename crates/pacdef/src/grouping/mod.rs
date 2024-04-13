/*!
This module reflects the relationship between groups, sections / backends and
packages.

A [`Group`] contains one (strictly spoken zero, but this doesn't make sense) or
more [`Section`]s, which relate to individual backends. Each section contains
one (strictly spoken zero) or more [`Package`]s. On start-up `pacdef` will load
all groups using [`Group::load`], which in turn will get all packages from all
sections.
*/
mod group;
mod package;
mod section;

pub use group::Group;
pub use package::Package;
pub use section::Section;
