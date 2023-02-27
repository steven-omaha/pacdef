mod actual;
mod backend_trait;
mod iter;
mod macros;
mod todo_per_backend;

pub use backend_trait::Backend;
pub use iter::BackendIter;
pub use todo_per_backend::ToDoPerBackend;

use pacdef_macros::Register;

#[derive(Debug, Register)]
pub enum Backends {
    #[cfg(feature = "arch")]
    Arch,
    #[cfg(feature = "debian")]
    Debian,
    Python,
    Rust,
}
