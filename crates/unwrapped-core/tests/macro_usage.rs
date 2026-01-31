use std::collections::HashMap;

use quote::{format_ident, quote};
use syn::DeriveInput;
use unwrapped_core::{
    Opts, UnwrappedProcUsageOpts, WrappedOpts, WrappedProcUsageOpts, unwrapped, wrapped,
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
