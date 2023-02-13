mod actual;
mod backend_trait;
mod iter;
mod macros;
mod todo_per_backend;

pub(crate) use backend_trait::Backend;
pub(crate) use iter::BackendIter;
pub(crate) use todo_per_backend::ToDoPerBackend;

use ::macros::Register;

#[derive(Debug, Register)]
pub(crate) enum Backends {
    Pacman,
    Rust,
}
