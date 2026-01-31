use std::collections::HashMap;

use quote::{format_ident, quote};
use syn::DeriveInput;
use unwrapped_core::{
    FieldProcOpts, Opts, UnwrappedFieldProcOpts, UnwrappedProcUsageOpts, WrappedOpts,
    WrappedProcUsageOpts, unwrapped, wrapped,
};

#[test]
fn test_macro_usage() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
            name: Option<String>
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), false);

    let model_options = Opts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options = UnwrappedProcUsageOpts::new(fields_to_unwrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let expected = quote! {
        #[derive(Clone, Debug, Default)]
        pub struct ThingFormValueHolder {
            pub id: i32,
            pub name: Option<String>
        }
    };

    assert!(model_struct.to_string().contains(&expected.to_string()));
}

#[test]
fn test_wrapped_macro_usage() {
    let thing = quote! {
        struct Thing {
            id: i32,
            name: String
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), false);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let expected = quote! {
        #[derive(Clone, Debug, Default)]
        pub struct ThingFormValueHolder {
            pub id: Option<i32>,
            pub name: String
        }
    };

    assert!(model_struct.to_string().contains(&expected.to_string()));
}

#[test]
fn test_unwrapped_with_struct_derives() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
            name: Option<String>
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), false);

    let model_options = Opts::builder()
        .suffix(format_ident!("Unwrapped"))
        .build()
        .with_derive(quote! { PartialEq })
        .with_derive(quote! { Eq });

    let macro_options = UnwrappedProcUsageOpts::new(fields_to_unwrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("PartialEq"));
    assert!(output.contains("Eq"));
}

#[test]
fn test_wrapped_with_struct_derives() {
    let thing = quote! {
        struct Thing {
            id: i32,
            name: String
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), false);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("Wrapped"))
        .build()
        .with_derive(quote! { PartialEq })
        .with_derive(quote! { Eq });

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("PartialEq"));
    assert!(output.contains("Eq"));
}

#[test]
fn test_unwrapped_with_struct_attrs() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);

    let model_options = Opts::builder()
        .suffix(format_ident!("Unwrapped"))
        .build()
        .with_attr(quote! { #[repr(C)] });

    let macro_options = UnwrappedProcUsageOpts::new(fields_to_unwrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("repr (C)"));
}

#[test]
fn test_wrapped_with_struct_attrs() {
    let thing = quote! {
        struct Thing {
            id: i32,
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("Wrapped"))
        .build()
        .with_attr(quote! { #[repr(C)] });

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("repr (C)"));
}

#[test]
fn test_unwrapped_with_field_attrs() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
            name: Option<String>,
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), true);

    let model_options = Opts::builder()
        .suffix(format_ident!("Unwrapped"))
        .build()
        .with_field_attr("id", quote! { #[validate(min = 1)] });

    let macro_options = UnwrappedProcUsageOpts::new(fields_to_unwrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("validate (min = 1)"));
}

#[test]
fn test_wrapped_with_field_attrs() {
    let thing = quote! {
        struct Thing {
            id: i32,
            name: String,
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), true);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("Wrapped"))
        .build()
        .with_field_attr("id", quote! { #[validate(min = 1)] });

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, None);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("validate (min = 1)"));
}

#[test]
fn test_unwrapped_with_proc_field_opts() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
            name: Option<String>,
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), true);

    let model_options = Opts::builder().suffix(format_ident!("Unwrapped")).build();

    let field_opts =
        UnwrappedFieldProcOpts::new(true).with_attr(quote! { #[serde(rename = "user_id")] });

    let macro_options =
        UnwrappedProcUsageOpts::new(fields_to_unwrap, None).with_field_opts("id", field_opts);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("serde (rename = \"user_id\")"));
}

#[test]
fn test_wrapped_with_proc_field_opts() {
    let thing = quote! {
        struct Thing {
            id: i32,
            name: String,
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), true);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("Wrapped"))
        .build();

    let field_opts = FieldProcOpts::new(true).with_attr(quote! { #[serde(rename = "user_id")] });

    let macro_options =
        WrappedProcUsageOpts::new(fields_to_wrap, None).with_field_opts("id", field_opts);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("serde (rename = \"user_id\")"));
}

#[test]
fn test_unwrapped_with_field_attr_fn() {
    let thing = quote! {
        struct Thing {
            id: Option<i32>,
            name: Option<String>,
        }
    };

    let mut fields_to_unwrap: HashMap<String, bool> = HashMap::new();
    fields_to_unwrap.insert("id".to_owned(), true);
    fields_to_unwrap.insert("name".to_owned(), true);

    let model_options = Opts::builder().suffix(format_ident!("Unwrapped")).build();

    fn attr_generator(field: &syn::Field) -> Option<proc_macro2::TokenStream> {
        let name = field.ident.as_ref()?.to_string();
        if name == "id" {
            Some(quote! { #[primary_key] })
        } else {
            None
        }
    }

    let macro_options =
        UnwrappedProcUsageOpts::new(fields_to_unwrap, None).with_field_attr_fn(attr_generator);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = unwrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("primary_key"));
}

#[test]
fn test_wrapped_with_field_attr_fn() {
    let thing = quote! {
        struct Thing {
            id: i32,
            name: String,
        }
    };

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    fields_to_wrap.insert("id".to_owned(), true);
    fields_to_wrap.insert("name".to_owned(), true);

    let model_options = WrappedOpts::builder()
        .suffix(format_ident!("Wrapped"))
        .build();

    fn attr_generator(field: &syn::Field) -> Option<proc_macro2::TokenStream> {
        let name = field.ident.as_ref()?.to_string();
        if name == "id" {
            Some(quote! { #[primary_key] })
        } else {
            None
        }
    }

    let macro_options =
        WrappedProcUsageOpts::new(fields_to_wrap, None).with_field_attr_fn(attr_generator);

    let parsed: DeriveInput = syn::parse2(thing).unwrap();

    let model_struct = wrapped(&parsed, Some(model_options), macro_options);

    let output = model_struct.to_string();
    assert!(output.contains("primary_key"));
}
