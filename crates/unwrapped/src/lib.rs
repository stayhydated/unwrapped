#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnwrappedError {
    pub field_name: &'static str,
}

impl std::fmt::Display for UnwrappedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to unwrap an Option for field '{}', found None",
            self.field_name
        )
    }
}

impl std::error::Error for UnwrappedError {}

pub trait Unwrapped {
    type Unwrapped;
}

#[cfg(feature = "derive")]
pub use unwrapped_derive::*;
