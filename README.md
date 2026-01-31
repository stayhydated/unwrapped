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
  c : Option<String>,
}
```

## Conversions

### Defaulting

Uses `unwrap_or_default()` on `Option` fields, which requires the field's `T` to implement `Default`.

```rust
use unwrapped::Unwrapped;

#[derive(Debug, PartialEq, Unwrapped)]
struct WithDefaults {
    val1: Option<i32>,       // i32::default() is 0
    val2: Option<String>,    // String::default() is ""
    val3: String,            // Not an Option, so it's unchanged
    val4: Option<Vec<u8>>,   // Vec::default() is an empty vector
}

let original = WithDefaults {
    val1: None,
    val2: Some("hello".to_string()),
    val3: "world".to_string(),
    val4: None,
};

let unwrapped: WithDefaultsUw = original.into();

assert_eq!(unwrapped.val1, 0);
assert_eq!(unwrapped.val2, "hello".to_string());
assert_eq!(unwrapped.val3, "world".to_string());
assert_eq!(unwrapped.val4, Vec::<u8>::new());
```

### Fallible

```rust
use unwrapped::Unwrapped;

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

let result = SimpleUw::try_from(original_fail);
assert!(result.is_err());
assert_eq!(
    result.unwrap_err(),
    UnwrappedError {
        field_name: "field1"
    }
);
```

## Customizing the Generated Struct Name

You can specify a custom name for the generated struct using the `unwrapped` attribute.

```rust
use unwrapped::Unwrapped;

#[derive(Debug, PartialEq, Unwrapped)]
#[unwrapped(prefix = "A", name = UserUnwrapped, suffix = "c")]
struct User0;

#[allow(dead_code)]
type S0 = UserUnwrapped;

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
  name: String,
}
```

## Conversions

### Defaulting

Uses `unwrap_or_default()` on `Option` fields when converting back to the original struct.

```rust
use unwrapped::Wrapped;

#[derive(Debug, PartialEq, Wrapped)]
struct Config {
    timeout: u64,
    retries: i32,
}

let wrapped = ConfigW {
    timeout: Some(30),
    retries: None,
};

let config: Config = wrapped.into();

assert_eq!(config.timeout, 30);
assert_eq!(config.retries, 0);  // Default i32
```

### Fallible

```rust
use unwrapped::{Wrapped, UnwrappedError};

#[derive(Debug, PartialEq, Wrapped)]
struct Config {
    timeout: u64,
    retries: i32,
}

let wrapped_missing = ConfigW {
    timeout: Some(30),
    retries: None,
};

let result = ConfigW::try_from(wrapped_missing);
assert!(result.is_err());
assert_eq!(
    result.unwrap_err(),
    UnwrappedError {
        field_name: "retries"
    }
);
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
unwrapped-core = { version = "0.1.0" }
```

You can then use it in your own proc-macro:

```rust
use syn::DeriveInput;

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();

    let model_options = unwrapped_core::Opts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    // Generate the unwrapped data model struct with a custom suffix
    let model_struct = unwrapped_core::unwrapped(&derive_input, Some(model_options));

    // ... your macro's logic ...

    let expanded = quote! {
        #model_struct
        pub struct #components_holder_name {
            #(#field_structure_tokens)*
        }

        #shape_impl

        pub struct #components_base_declarations_name;

        impl #components_base_declarations_name {
          #(#field_base_declarations_tokens)*
        }
    };

    expanded.into()
}
```

Or for the Wrapped variant:

```rust
use std::collections::HashMap;
use syn::DeriveInput;

#[proc_macro_derive(MyWrappedMacro)]
pub fn my_wrapped_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), false);

    let model_options = unwrapped_core::WrappedOpts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options = unwrapped_core::WrappedProcUsageOpts::new(fields_to_wrap, None);

    // Generate the wrapped data model struct with a custom suffix
    let model_struct = unwrapped_core::wrapped(&derive_input, Some(model_options), macro_options);

    // ... your macro's logic ...

    expanded.into()
}
```
