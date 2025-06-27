extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
use unwrapped_core::{ProcUsageOpts, unwrapped};

#[proc_macro_derive(Unwrapped, attributes(unwrapped))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    unwrapped(&input, None, ProcUsageOpts::default()).into()
}
