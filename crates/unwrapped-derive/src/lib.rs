#[doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
use unwrapped_core::{UnwrappedProcUsageOpts, WrappedProcUsageOpts, unwrapped, wrapped};

#[proc_macro_derive(Unwrapped, attributes(unwrapped))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    unwrapped(&input, None, UnwrappedProcUsageOpts::default()).into()
}

#[proc_macro_derive(Wrapped, attributes(wrapped))]
pub fn derive_wrapped(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    wrapped(&input, None, WrappedProcUsageOpts::default()).into()
}
