#[doc = include_str!("../README.md")]

/// Error returned by `try_from()` when an `Option` field is `None`.
///
/// Contains the name of the field that failed to unwrap, useful for debugging
/// and error reporting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnwrappedError {
    /// The name of the field that was `None`.
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

/// Trait that associates a struct with its unwrapped variant.
///
/// Automatically implemented by `#[derive(Unwrapped)]`. The associated type
/// `Unwrapped` is the generated struct where all `Option<T>` fields become `T`.
pub trait Unwrapped {
    /// The unwrapped variant of this type.
    type Unwrapped;
}

/// Trait that associates a struct with its wrapped variant.
///
/// Automatically implemented by `#[derive(Wrapped)]`. The associated type
/// `Wrapped` is the generated struct where all `T` fields become `Option<T>`.
pub trait Wrapped {
    /// The wrapped variant of this type.
    type Wrapped;
}

#[cfg(feature = "derive")]
pub use unwrapped_derive::*;
