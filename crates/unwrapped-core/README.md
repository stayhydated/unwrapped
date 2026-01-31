# unwrapped-core

Core logic for generating unwrapped struct variants. This crate is intended for proc-macro authors who want to generate unwrapped structs as part of their own macros.

## Usage

```rs
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;
use std::collections::HashMap;
use unwrapped_core::{Opts, UnwrappedProcUsageOpts, unwrapped};

#[proc_macro_derive(MyUnwrappedMacro)]
pub fn my_unwrapped_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();

    let model_options = Opts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), false);

    let macro_options = UnwrappedProcUsageOpts::new(fields_to_unwrap, None);

    // Generate the unwrapped data model struct with a custom suffix
    let model_struct = unwrapped(&derive_input, Some(model_options), macro_options);

    // ... your macro's logic ...

    let expanded = quote! {
        #model_struct
    };

    expanded.into()
}
```

Or for the Wrapped variant:

```rs
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;
use std::collections::HashMap;
use unwrapped_core::{WrappedOpts, WrappedProcUsageOpts, wrapped};

#[proc_macro_derive(MyWrappedMacro)]
pub fn my_wrapped_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), false);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, None);

    // Generate the wrapped data model struct with a custom suffix
    let model_struct = wrapped(&derive_input, Some(model_options), macro_options);

    // ... your macro's logic ...

    let expanded = quote! {
        #model_struct
    };

    expanded.into()
}
```
