# unwrapped-core

Core logic for generating unwrapped struct variants. This crate is intended for proc-macro authors who want to generate unwrapped structs as part of their own macros.

## Usage

```rust
use syn::DeriveInput;
use unwrapped_core::{Opts, ProcUsageOpts, unwrapped};

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();

    let opts = Opts::builder()
        .suffix(format_ident!("Unwrapped"))
        .build();

    let generated = unwrapped(&derive_input, Some(opts), ProcUsageOpts::default());

    // ... combine with your macro's output ...
}
```
