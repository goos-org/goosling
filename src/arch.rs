pub mod traits;
pub mod x86_64;

// Will be arch-dependent anyway
pub use x86_64 as native;

type InterruptHandler =
    fn(error_code: Option<usize>, interrupt_number: usize, instruction_pointer: &mut usize);

#[derive(Debug)]
pub enum Error {
    MisAligned,
    Unsupported,
    Uninitialized,
}
