#[cfg(feature = "arch")]
pub mod arch;
#[cfg(feature = "debian")]
pub mod debian;
pub mod python;
pub mod rust;
#[cfg(feature = "flatpak")]
pub mod flatpak;
