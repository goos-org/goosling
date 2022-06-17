pub mod traits;
pub mod x86_64;

// Will be arch-dependent anyway
pub use x86_64 as native;

#[derive(Debug)]
pub enum Error {
    MisAligned,
    Unsupported,
    Uninitialized,
}
