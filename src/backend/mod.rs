mod backend_trait;
mod iter;
mod macros;
mod pacman;
mod rust;
mod todo_per_backend;
mod types;

pub(crate) use backend_trait::Backend;
pub(crate) use iter::BackendIter;
pub(crate) use todo_per_backend::ToDoPerBackend;

use pacdef_macro::Register;

#[derive(Debug, Register)]
pub(crate) enum Backends {
    Pacman,
    Rust,
}
