use std::collections::HashMap;

use bon::Builder;
use darling::{FromDeriveInput, FromField};
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::utils::{get_struct_data, is_option_type};

#[derive(Clone, Debug, Default, FromField)]
#[darling(default, attributes(wrapped))]
struct WrappedFieldOpts {
    skip: bool,
}

#[derive(Builder, Clone, Debug, FromDeriveInput)]
#[darling(attributes(wrapped), supports(struct_any))]
pub struct WrappedOpts {
    name: Option<syn::Ident>,
    prefix: Option<syn::Ident>,
    suffix: Option<syn::Ident>,
}

impl WrappedOpts {
    pub fn wrapped_ident(&self, original_ident: &syn::Ident) -> syn::Ident {
        let base = self.name.as_ref().unwrap_or(original_ident);
        let prefix = &self
            .prefix
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let suffix = &self
            .suffix
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let new = format_ident!("{}{}{}", prefix, base, suffix);

        if &new == original_ident {
            format_ident!("{}W", original_ident)
        } else {
            new
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WrappedProcUsageOpts {
    pub fields_to_wrap: HashMap<String, bool>,
    lib_holder_name: Option<syn::Ident>,
}

impl WrappedProcUsageOpts {
    pub fn new(fields_to_wrap: HashMap<String, bool>, lib_holder_name: Option<syn::Ident>) -> Self {
        Self {
            fields_to_wrap,
            lib_holder_name,
        }
    }
    pub fn lib_path(&self) -> syn::Path {
        if let Some(name) = &self.lib_holder_name {
            syn::parse_str(&format!("{}::unwrapped", name)).unwrap()
        } else {
            syn::parse_str("unwrapped").unwrap()
        }
    }
}

pub fn wrapped(
    input: &DeriveInput,
    options: Option<WrappedOpts>,
    proc_usage_opts: WrappedProcUsageOpts,
) -> proc_macro2::TokenStream {
    let opts =
        options.unwrap_or_else(|| WrappedOpts::from_derive_input(input).expect("Wrong options"));
    let lib_path = proc_usage_opts.lib_path();

    let original_ident = &input.ident;
    let wrapped_ident = &opts.wrapped_ident(original_ident);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let s = get_struct_data(input);

    // Generate wrapped struct fields - all non-Option<T> fields become Option<T>
    let fields = s.fields.iter().map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_skip = field_opts.skip
            || !proc_usage_opts
                .fields_to_wrap
                .get(&name_str)
                .unwrap_or(&true);

        if is_already_option || should_skip {
            quote! { pub #name: #ty }
        } else {
            quote! { pub #name: Option<#ty> }
        }
    });

    // Generate From<Wrapped> for Original - unwrap values (or use default)
    let from_fields = s.fields.iter().map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_skip = field_opts.skip
            || !proc_usage_opts
                .fields_to_wrap
                .get(&name_str)
                .unwrap_or(&true);

        if is_already_option || should_skip {
            quote! { #name: from.#name }
        } else {
            quote! { #name: from.#name.unwrap_or_default() }
        }
    });

    // Generate From<Original> for Wrapped - wrap values in Some()
    let to_wrapped_fields = s.fields.iter().map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_skip = field_opts.skip
            || !proc_usage_opts
                .fields_to_wrap
                .get(&name_str)
                .unwrap_or(&true);

        if is_already_option || should_skip {
            quote! { #name: from.#name }
        } else {
            quote! { #name: Some(from.#name) }
        }
    });

    // Generate try_from method for Wrapped -> Original (returns error if any required field is None)
    let try_from_fields = s.fields.iter().map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_skip = field_opts.skip
            || !proc_usage_opts.fields_to_wrap.get(&name_str).unwrap_or(&true);

        if is_already_option || should_skip {
            quote! { #name: from.#name }
        } else {
            let field_name_str = name.as_ref().unwrap().to_string();
            quote! { #name: from.#name.ok_or(::#lib_path::UnwrappedError{ field_name: #field_name_str })? }
        }
    });

    // Generate where clause with Default bounds for types that get unwrapped
    let mut try_from_where_clause = where_clause.cloned();
    let new_predicates: Vec<syn::WherePredicate> = s
        .fields
        .iter()
        .filter_map(|f| {
            let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
            let ty = &f.ty;
            let name_str = f.ident.as_ref().unwrap().to_string();
            let should_wrap = !field_opts.skip
                && *proc_usage_opts
                    .fields_to_wrap
                    .get(&name_str)
                    .unwrap_or(&true);

            if is_option_type(ty).is_none() && should_wrap {
                return Some(syn::parse_quote!(#ty: ::core::default::Default));
            }
            None
        })
        .collect();

    if !new_predicates.is_empty() {
        let wc = try_from_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
        wc.predicates.extend(new_predicates);
    }

    quote! {
        #[derive(Clone, Debug, Default)]
        pub struct #wrapped_ident #ty_generics #where_clause {
            #(#fields),*
        }

        impl #impl_generics From<#original_ident #ty_generics> for #wrapped_ident #ty_generics #where_clause {
            fn from(from: #original_ident #ty_generics) -> Self {
                Self {
                    #(#to_wrapped_fields),*
                }
            }
        }

        impl #impl_generics From<#wrapped_ident #ty_generics> for #original_ident #ty_generics #try_from_where_clause {
            fn from(from: #wrapped_ident #ty_generics) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }

        impl #impl_generics ::#lib_path::Wrapped for #original_ident #ty_generics #where_clause {
            type Wrapped = #wrapped_ident #ty_generics;
        }

        impl #impl_generics #wrapped_ident #ty_generics #where_clause {
            pub fn try_from(from: #wrapped_ident #ty_generics) -> Result<#original_ident #ty_generics, ::#lib_path::UnwrappedError> {
                Ok(#original_ident {
                    #(#try_from_fields),*
                })
            }
        }
    }
}
