mod actual;
mod backend_trait;
mod iter;
mod macros;
mod todo_per_backend;

pub use backend_trait::Backend;
pub use iter::BackendIter;
pub use todo_per_backend::ToDoPerBackend;

use ::macros::Register;

#[derive(Debug, Register)]
pub(crate) enum Backends {
    Pacman,
    Rust,
}
