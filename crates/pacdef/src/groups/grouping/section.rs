use std::collections::BTreeSet;
use std::fmt::{Display, Write};
use std::hash::Hash;
use std::iter::Peekable;

use anyhow::{ensure, Context, Result};

use crate::prelude::*;

pub type Sections = BTreeSet<Section>;

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub packages: Packages,
}
