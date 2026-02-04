[![Build Status](https://github.com/stayhydated/unwrapped/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/unwrapped/actions/workflows/ci.yml)
[![Docs](https://docs.rs/unwrapped/badge.svg)](https://docs.rs/unwrapped/)
[![Crates.io](https://img.shields.io/crates/v/unwrapped.svg)](https://crates.io/crates/unwrapped)

# Unwrapped

Creates a new struct, changing each field `Option<T> -> T`

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

Fields marked with `#[unwrapped(skip)]` are completely removed from the generated struct. When any field has `skip`, the `From` trait implementations are not generated (since conversion is impossible without all fields).

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

#### Partial Builder Pattern with `bon` (More Ergonomic)

While we cannot directly return a partial `bon` builder due to Rust's type system limitations with bon's type-state pattern, you can achieve the same ergonomic result by destructuring the unwrapped struct inline:

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

// Destructure and use bon's builder - ergonomic with named parameters!
let UserFormUw { name, email } = form;
let original = UserForm::builder()
    .name(name)                // Non-skipped fields
    .email(email)              // bon's into conversion handles wrapping
    .created_at(1234567890)    // Skipped fields
    .id(42)                    // Can be in any order!
    .build();

assert_eq!(original.name, Some("Alice".to_string()));
assert_eq!(original.email, Some("alice@example.com".to_string()));
assert_eq!(original.created_at, 1234567890);
assert_eq!(original.id, 42);
```

**Tip:** Use destructuring (`let UserFormUw { name, email } = form;`) to extract non-skipped fields, then pass them to the builder. This gives you the ergonomics of a partial builder!

## Conversions

**Important: No panics, no defaults!** All conversions are explicit and fallible.

- **NO `From<Original> for Unwrapped`** - Would panic if any Option is None
- **Use `try_from()` instead** - Returns `Result`, fails if any Option field is None
- **`From<Unwrapped> for Original`** - Always safe (wraps fields in Some)

### Fallible Conversion

```rust
use unwrapped::{Unwrapped, UnwrappedError};

#[derive(Debug, PartialEq, Unwrapped)]
struct Simple {
    field1: Option<i32>,
    field2: String,
    field3: Option<u64>,
}

let original_fail = Simple {
    field1: None,
    field2: "world".to_string(),
    field3: Some(200),
};

// try_from returns Err if any Option field is None
let result = SimpleUw::try_from(original_fail);
assert!(result.is_err());
match result {
    Err(e) => assert_eq!(e.field_name, "field1"),
    Ok(_) => panic!("Expected error"),
}

// Convert back (always safe - wraps in Some)
let simple_uw = SimpleUw {
    field1: 42,
    field2: "test".to_string(),
    field3: 100,
};
let back_to_original: Simple = simple_uw.into();
assert_eq!(back_to_original.field1, Some(42));
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

# Wrapped

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

Note: `into_original` returns a `Result` because wrapped fields might be `None`.

#### Using with `bon` Builders

Similar to `Unwrapped`, you can manually use `bon` builders with `Wrapped` structs for a more ergonomic API.

## Conversions

**Important: No panics, no defaults!** All conversions are explicit and fallible.

- **`From<Original> for Wrapped`** - Always safe (wraps fields in Some)
- **NO `From<Wrapped> for Original`** - Would panic if any Option is None
- **Use `try_from()` instead** - Returns `Result`, fails if any Option field is None

### Fallible Conversion

```rust
use unwrapped::{Wrapped, UnwrappedError};

#[derive(Debug, PartialEq, Wrapped)]
struct Config {
    timeout: u64,
    retries: i32,
}

// try_from returns Err if any wrapped field is None
let wrapped_missing = ConfigW {
    timeout: Some(30),
    retries: None,
};

let result = ConfigW::try_from(wrapped_missing);
assert!(result.is_err());
match result {
    Err(e) => assert_eq!(e.field_name, "retries"),
    Ok(_) => panic!("Expected error"),
}

// Success when all fields are Some
let wrapped_ok = ConfigW {
    timeout: Some(30),
    retries: Some(3),
};
let config: Config = ConfigW::try_from(wrapped_ok).unwrap();
assert_eq!(config.timeout, 30);
assert_eq!(config.retries, 3);
```

## Customizing the Generated Struct Name

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

| Feature                               | WITHOUT `skip`          | WITH `skip`              |
| ------------------------------------- | ----------------------- | ------------------------ |
| **Generated struct**                  | All fields transformed  | Skipped fields removed   |
| **`From<Original> for Generated`**    | ❌ Never (would panic)  | ❌ Never                 |
| **`From<Generated> for Original`**    | ✅ Yes (Unwrapped only) | ❌ No                    |
| **`From<Original> for Generated`**    | ✅ Yes (Wrapped only)   | ❌ No                    |
| **`try_from` (Original → Generated)** | ✅ Yes                  | ✅ Yes (ignores skipped) |
| **`try_from` (Generated → Original)** | ✅ Yes (Wrapped only)   | ❌ No                    |
| **`into_original` (Generated → Original)** | ❌ No              | ✅ Yes (requires skipped field params) |
| **Panics**                            | ❌ Never                | ❌ Never                 |
| **Defaults**                          | ❌ Never                | ❌ Never                 |
