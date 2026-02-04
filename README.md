[![Build Status](https://github.com/stayhydated/unwrapped/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/unwrapped/actions/workflows/ci.yml)
[![Docs](https://docs.rs/unwrapped/badge.svg)](https://docs.rs/unwrapped/)
[![Crates.io](https://img.shields.io/crates/v/unwrapped.svg)](https://crates.io/crates/unwrapped)

# Unwrapped

Generate struct variants with different optionality semantics.

## #[derive(Unwrapped)]

Creates a new struct, changing each field `Option<T> -> T`.

```rs
#[derive(Unwrapped)]
pub struct Ab {
  a : Option<Ab>,
  b : u8,
  #[unwrapped(skip)]
  c : Option<String>,
}
```

->

```rs
pub struct AbUw {
  a : Ab,
  b : u8,
  // c is not included - skip removes the field entirely
}
```

Fields marked with `#[unwrapped(skip)]` are completely removed from the generated struct. When any field has `skip`, `From` trait implementations are not generated (since conversion is impossible without all fields).

### Conversions

**Important: No panics, no defaults!** All conversions are explicit and fallible.

- `Unwrapped::try_from(original)` is always generated. It returns `Err(UnwrappedError)` if any non-skipped `Option` field is `None`.
- `From<Unwrapped> for Original` is generated only when no fields are skipped.
- With skipped fields, use `into_original(self, skipped...)` to reconstruct the original type.

### Converting Back with Skipped Fields

When fields are skipped, an `into_original` helper method is generated that allows you to reconstruct the original struct by providing values for the skipped fields:

```rust
use unwrapped::Unwrapped;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(name = UserFormUw)]
struct UserForm {
    name: Option<String>,
    email: Option<String>,
    #[unwrapped(skip)]
    created_at: i64,
    #[unwrapped(skip)]
    id: u64,
}

// Create an unwrapped struct (without skipped fields)
let form = UserFormUw {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

// Convert back to original using into_original, providing skipped fields
let original = form.into_original(1234567890, 42);

assert_eq!(original.name, Some("Alice".to_string()));
assert_eq!(original.email, Some("alice@example.com".to_string()));
assert_eq!(original.created_at, 1234567890);
assert_eq!(original.id, 42);
```

### Using `bon` Builders (Optional)

If the original struct uses `bon::Builder` (via `#[derive(bon::Builder)]` or `#[builder(...)]`) and you also use `skip`, the macro adds a helper on the builder:

- `from_unwrapped(self, uw)` pre-fills the builder with the non-skipped fields.

```rust
use unwrapped::Unwrapped;

#[derive(Debug, PartialEq, Unwrapped, bon::Builder)]
#[unwrapped(name = UserFormUw)]
#[builder(on(Option<String>, into))]
struct UserForm {
    name: Option<String>,
    email: Option<String>,
    #[unwrapped(skip)]
    created_at: i64,
    #[unwrapped(skip)]
    id: u64,
}

let form = UserFormUw {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

let original = UserForm::builder()
    .from_unwrapped(form)
    .created_at(1234567890)
    .id(42)
    .build();

assert_eq!(original.name, Some("Alice".to_string()));
assert_eq!(original.email, Some("alice@example.com".to_string()));
assert_eq!(original.created_at, 1234567890);
assert_eq!(original.id, 42);
```

If you are not using `bon`, you can still destructure the unwrapped struct and pass fields manually.

## #[derive(Wrapped)]

Creates a new struct, changing each field `T -> Option<T>`. This is the inverse of `Unwrapped`.

```rs
#[derive(Wrapped)]
pub struct Config {
  timeout: u64,
  retries: i32,
  #[wrapped(skip)]
  name: String,
}
```

->

```rs
pub struct ConfigW {
  timeout: Option<u64>,
  retries: Option<i32>,
  // name is not included - skip removes the field entirely
}
```

Fields marked with `#[wrapped(skip)]` are completely removed from the generated struct. When any field has `skip`, the `From` trait implementations are not generated (since conversion is impossible without all fields).

### Conversions

**Important: No panics, no defaults!** All conversions are explicit and fallible.

- `From<Original> for Wrapped` is generated only when no fields are skipped.
- `Wrapped::try_from(wrapped)` is generated only when no fields are skipped and returns `Err(UnwrappedError)` if any required wrapped field is `None`.
- With skipped fields, use `into_original(self, skipped...) -> Result<Original, UnwrappedError>`.

### Converting Back with Skipped Fields

When fields are skipped, an `into_original` helper method is generated that allows you to reconstruct the original struct by providing values for the skipped fields:

```rust
use unwrapped::Wrapped;

#[derive(Debug, PartialEq, Wrapped)]
#[wrapped(name = ConfigW)]
struct Config {
    timeout: u64,
    retries: i32,
    #[wrapped(skip)]
    created_at: i64,
    #[wrapped(skip)]
    version: String,
}

// Create a wrapped struct (without skipped fields)
let wrapped = ConfigW {
    timeout: Some(30),
    retries: Some(3),
};

// Convert back to original using into_original, providing skipped fields
let original = wrapped
    .into_original(1234567890, "v1.0".to_string())
    .unwrap();

assert_eq!(original.timeout, 30);
assert_eq!(original.retries, 3);
assert_eq!(original.created_at, 1234567890);
assert_eq!(original.version, "v1.0".to_string());
```

### Using `bon` Builders (Optional)

If the original struct uses `bon::Builder` (via `#[derive(bon::Builder)]` or `#[builder(...)]`) and you also use `skip`, the macro adds a helper on the builder:

- `from_wrapped(self, w)` pre-fills the builder and returns `Result<Builder, UnwrappedError>`.

```rust
use unwrapped::Wrapped;

#[derive(Debug, PartialEq, Wrapped, bon::Builder)]
#[wrapped(name = UserFormW)]
#[builder(on(Option<String>, into))]
struct UserForm {
    name: String,
    email: String,
    note: Option<String>,
    #[wrapped(skip)]
    created_at: i64,
    #[wrapped(skip)]
    id: u64,
}

let wrapped = UserFormW {
    name: Some("Alice".to_string()),
    email: Some("alice@example.com".to_string()),
    note: Some("hello".to_string()),
};

let original = UserForm::builder()
    .from_wrapped(wrapped)
    .unwrap()
    .created_at(1234567890)
    .id(42)
    .build();

assert_eq!(original.name, "Alice".to_string());
assert_eq!(original.email, "alice@example.com".to_string());
assert_eq!(original.note, Some("hello".to_string()));
assert_eq!(original.created_at, 1234567890);
assert_eq!(original.id, 42);
```

## Customizing the Generated Struct Name

You can specify a custom name for the generated struct using the `unwrapped` attribute.

```rust
use unwrapped::Unwrapped;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(prefix = "A", name = UserUnwrapped, suffix = "c")]
struct User0;

#[allow(dead_code)]
type S0 = AUserUnwrappedc;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(prefix = "Bad")]
struct User1;

#[allow(dead_code)]
type S1 = BadUser1;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(suffix = "Something")]
struct User2;

#[allow(dead_code)]
type S2 = User2Something;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(prefix = "Bad", suffix = "Something")]
struct User3;

#[allow(dead_code)]
type S3 = BadUser3Something;
```

You can specify a custom name for the generated struct using the `wrapped` attribute.

```rust
use unwrapped::Wrapped;

#[derive(Debug, PartialEq, Wrapped)]
#[wrapped(prefix = "A", name = UserWrapped, suffix = "c")]
struct User0;

#[allow(dead_code)]
type S0 = AUserWrappedc;

#[derive(Debug, PartialEq, Wrapped)]
#[wrapped(prefix = "Bad")]
struct User1;

#[allow(dead_code)]
type S1 = BadUser1;

#[derive(Debug, PartialEq, Wrapped)]
#[wrapped(suffix = "Something")]
struct User2;

#[allow(dead_code)]
type S2 = User2Something;

#[derive(Debug, PartialEq, Wrapped)]
#[wrapped(prefix = "Bad", suffix = "Something")]
struct User3;

#[allow(dead_code)]
type S3 = BadUser3Something;
```

## For Proc-Macro Authors

```toml
[dependencies]
unwrapped-core = { version = "*" }
```

You can then use it in your own proc-macro:

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
