# unwrapped-core

Core logic for generating unwrapped and wrapped struct variants. This crate is intended for proc-macro authors who want to generate these variants as part of their own macros.

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

## Customization

- **Naming**: `name`, `prefix`, and `suffix` are supported via `Opts` / `WrappedOpts` (and the `#[unwrapped(...)]` / `#[wrapped(...)]` attributes).
- **Per-field transforms**: `fields_to_unwrap` and `fields_to_wrap` control which fields are transformed.
- **Custom derives**: `with_derive` and `with_derives` add derives to the generated struct. If you add none, the core emits `#[derive()]` with no defaults.
- **Struct and field attributes**: `with_attr` adds struct-level attributes, `with_field_attr` adds per-field attributes.
- **Dynamic field attributes**: `with_field_attr_fn` lets you generate attributes from the `syn::Field` at macro time.
- **Per-field proc usage opts**: `with_field_opts` allows per-field attributes (for Unwrapped use `UnwrappedFieldProcOpts`, for Wrapped use `FieldProcOpts`).
- **Crate path override**: pass `lib_holder_name` to `UnwrappedProcUsageOpts::new` / `WrappedProcUsageOpts::new` if the `unwrapped` crate is re-exported under a different path.
- **bon builder helper**: when skipped fields are present and the input struct derives `bon::Builder` (or uses `#[builder(...)]`), the generated code adds `from_unwrapped` / `from_wrapped` helpers on the builder to pre-fill non-skipped fields.

The `CommonOpts` and `CommonProcUsageOpts` types are also exported for shared configuration across Unwrapped and Wrapped generation.
