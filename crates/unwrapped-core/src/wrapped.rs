use std::collections::HashMap;

use bon::Builder;
use darling::{FromDeriveInput, FromField};
use quote::quote;
use syn::DeriveInput;

use crate::utils::{
    CommonOpts, ProcUsageOpts, build_derive_output, collect_field_attrs, get_struct_data,
    is_option_type,
};

#[derive(Clone, Debug, Default, FromField)]
#[darling(default, attributes(wrapped))]
struct WrappedFieldOpts {
    skip: bool,
    default: Option<syn::Expr>, // Parse custom default expression
}

#[derive(Builder, Clone, Debug, FromDeriveInput)]
#[darling(attributes(wrapped), supports(struct_any))]
pub struct WrappedOpts {
    name: Option<syn::Ident>,
    prefix: Option<syn::Ident>,
    suffix: Option<syn::Ident>,

    /// Custom derives to add to the generated struct (in addition to Clone, Debug, Default)
    #[builder(default)]
    #[darling(skip)]
    struct_derives: Vec<proc_macro2::TokenStream>,

    /// Custom attributes to add to the generated struct
    #[builder(default)]
    #[darling(skip)]
    struct_attrs: Vec<proc_macro2::TokenStream>,

    /// Per-field attributes to add to specific fields
    #[builder(default)]
    #[darling(skip)]
    field_attrs: HashMap<String, Vec<proc_macro2::TokenStream>>,
}

impl WrappedOpts {
    pub fn wrapped_ident(&self, original_ident: &syn::Ident) -> syn::Ident {
        self.to_common().generate_ident(original_ident, "W")
    }

    /// Add a derive to the generated struct
    pub fn with_derive(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.struct_derives.push(tokens.into());
        self
    }

    /// Add multiple derives at once
    pub fn with_derives(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.struct_derives.push(tokens.into());
        self
    }

    /// Add a struct-level attribute
    pub fn with_attr(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.struct_attrs.push(tokens.into());
        self
    }

    /// Add an attribute to a specific field by name
    pub fn with_field_attr(
        mut self,
        field_name: impl AsRef<str>,
        tokens: impl Into<proc_macro2::TokenStream>,
    ) -> Self {
        let name = field_name.as_ref().to_string();
        self.field_attrs
            .entry(name)
            .or_default()
            .push(tokens.into());
        self
    }

    fn to_common(&self) -> CommonOpts {
        CommonOpts {
            name: self.name.clone(),
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
            struct_derives: self.struct_derives.clone(),
            struct_attrs: self.struct_attrs.clone(),
            field_attrs: self.field_attrs.clone(),
        }
    }
}

/// Per-field options for procedural macro usage
#[derive(Clone, Debug, Default)]
pub struct FieldProcOpts {
    pub wrap: bool,
    pub attrs: Vec<proc_macro2::TokenStream>,
    pub default_expr: Option<proc_macro2::TokenStream>,
}

impl FieldProcOpts {
    pub fn new(wrap: bool) -> Self {
        Self {
            wrap,
            attrs: Vec::new(),
            default_expr: None,
        }
    }

    pub fn with_attr(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.attrs.push(tokens.into());
        self
    }

    /// Set custom default expression
    pub fn with_default(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.default_expr = Some(tokens.into());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct WrappedProcUsageOpts {
    pub fields_to_wrap: HashMap<String, bool>,
    lib_holder_name: Option<syn::Ident>,
    /// Field transformations: name -> (should_wrap, attributes)
    pub field_opts: HashMap<String, FieldProcOpts>,
    /// Dynamic field attribute generator
    pub field_attr_fn: Option<fn(&syn::Field) -> Option<proc_macro2::TokenStream>>,
}

impl WrappedProcUsageOpts {
    pub fn new(fields_to_wrap: HashMap<String, bool>, lib_holder_name: Option<syn::Ident>) -> Self {
        Self {
            fields_to_wrap,
            lib_holder_name,
            field_opts: HashMap::new(),
            field_attr_fn: None,
        }
    }

    pub fn lib_path(&self) -> syn::Path {
        if let Some(name) = &self.lib_holder_name {
            syn::parse_str(&format!("{}::unwrapped", name)).unwrap()
        } else {
            syn::parse_str("unwrapped").unwrap()
        }
    }

    /// Set options for a specific field
    pub fn with_field_opts(mut self, field_name: impl AsRef<str>, opts: FieldProcOpts) -> Self {
        self.field_opts
            .insert(field_name.as_ref().to_string(), opts);
        self
    }

    /// Set a dynamic field attribute generator
    pub fn with_field_attr_fn(
        mut self,
        f: fn(&syn::Field) -> Option<proc_macro2::TokenStream>,
    ) -> Self {
        self.field_attr_fn = Some(f);
        self
    }

    fn to_common(&self) -> ProcUsageOpts {
        let mut field_opts = HashMap::new();
        for (name, opts) in &self.field_opts {
            field_opts.insert(
                name.clone(),
                crate::utils::FieldProcOpts {
                    transform: opts.wrap,
                    attrs: opts.attrs.clone(),
                    default_expr: opts.default_expr.clone(),
                },
            );
        }
        ProcUsageOpts {
            fields_to_transform: self.fields_to_wrap.clone(),
            lib_holder_name: self.lib_holder_name.clone(),
            field_opts,
            field_attr_fn: self.field_attr_fn,
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
    let common_opts = opts.to_common();
    let common_proc_opts = proc_usage_opts.to_common();

    let original_ident = &input.ident;
    let wrapped_ident = &opts.wrapped_ident(original_ident);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let s = get_struct_data(input);

    // Check if any field has skip attribute
    let has_skipped_fields = s.fields.iter().any(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
        field_opts.skip
    });

    // Generate wrapped struct fields - all non-Option<T> fields become Option<T>
    let fields = s.fields.iter().filter_map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");

        // Skip this field entirely if skip attribute is present
        if field_opts.skip {
            return None;
        }
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_process = *proc_usage_opts
            .fields_to_wrap
            .get(&name_str)
            .unwrap_or(&true);

        // Collect field attributes
        let field_attrs = collect_field_attrs(f, &common_opts, &common_proc_opts);

        if is_already_option || !should_process {
            Some(quote! { #(#field_attrs)* pub #name: #ty })
        } else {
            Some(quote! { #(#field_attrs)* pub #name: Option<#ty> })
        }
    });

    // Generate From<Wrapped> for Original - unwrap values (no defaults)
    let _from_fields = s.fields.iter().filter_map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");

        // Skip this field if skip attribute is present
        if field_opts.skip {
            return None;
        }
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_process = *proc_usage_opts
            .fields_to_wrap
            .get(&name_str)
            .unwrap_or(&true);

        if is_already_option || !should_process {
            Some(quote! { #name: from.#name })
        } else {
            let field_name_str = name.as_ref().unwrap().to_string();
            Some(quote! { #name: from.#name.ok_or(::#lib_path::UnwrappedError{ field_name: #field_name_str })? })
        }
    });

    // Generate From<Original> for Wrapped - wrap values in Some()
    let to_wrapped_fields = s.fields.iter().filter_map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");

        // Skip this field if skip attribute is present
        if field_opts.skip {
            return None;
        }
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_process = *proc_usage_opts
            .fields_to_wrap
            .get(&name_str)
            .unwrap_or(&true);

        if is_already_option || !should_process {
            Some(quote! { #name: from.#name })
        } else if let Some(default_expr) = &field_opts.default {
            // If value equals default, store as None
            Some(quote! {
                #name: if from.#name == (#default_expr) {
                    None
                } else {
                    Some(from.#name)
                }
            })
        } else {
            Some(quote! { #name: Some(from.#name) })
        }
    });

    // Generate try_from method for Wrapped -> Original (returns error if any required field is None)
    let try_from_fields = s.fields.iter().filter_map(|f| {
        let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");

        // Skip this field if skip attribute is present
        if field_opts.skip {
            return None;
        }
        let name = &f.ident;
        let ty = &f.ty;
        let name_str = name.as_ref().unwrap().to_string();

        let is_already_option = is_option_type(ty).is_some();
        let should_process = *proc_usage_opts.fields_to_wrap.get(&name_str).unwrap_or(&true);

        if is_already_option || !should_process {
            Some(quote! { #name: from.#name })
        } else {
            let field_name_str = name.as_ref().unwrap().to_string();
            Some(quote! { #name: from.#name.ok_or(::#lib_path::UnwrappedError{ field_name: #field_name_str })? })
        }
    });

    // Build struct-level attributes and derives
    let struct_attrs = &opts.struct_attrs;
    let derive_output = build_derive_output(&opts.struct_derives);

    // Only generate From implementations if there are no skipped fields
    if has_skipped_fields {
        // Collect skipped fields for into_original method
        let skipped_params = s.fields.iter().filter_map(|f| {
            let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
            if field_opts.skip {
                let name = &f.ident;
                let ty = &f.ty;
                Some(quote! { #name: #ty })
            } else {
                None
            }
        });

        // Build field assignments for into_original
        let into_original_fields = s.fields.iter().map(|f| {
            let field_opts = WrappedFieldOpts::from_field(f).expect("Wrong field options");
            let name = &f.ident;
            let ty = &f.ty;
            let name_str = name.as_ref().unwrap().to_string();

            if field_opts.skip {
                // Skipped fields come from parameters
                quote! { #name }
            } else {
                let is_already_option = is_option_type(ty).is_some();
                let should_process = *proc_usage_opts.fields_to_wrap.get(&name_str).unwrap_or(&true);

                if is_already_option || !should_process {
                    // Already Option or not processed -> keep as is
                    quote! { #name: self.#name }
                } else if let Some(default_expr) = &field_opts.default {
                    // Unwrap with default value
                    quote! { #name: self.#name.unwrap_or_else(|| #default_expr) }
                } else {
                    // Unwrap Option, return error if None
                    let field_name_str = name.as_ref().unwrap().to_string();
                    quote! { #name: self.#name.ok_or(::#lib_path::UnwrappedError{ field_name: #field_name_str })? }
                }
            }
        });

        quote! {
            #(#struct_attrs)*
            #derive_output
            pub struct #wrapped_ident #ty_generics #where_clause {
                #(#fields),*
            }

            impl #impl_generics ::#lib_path::Wrapped for #original_ident #ty_generics #where_clause {
                type Wrapped = #wrapped_ident #ty_generics;
            }

            impl #impl_generics #wrapped_ident #ty_generics #where_clause {
                /// Convert back to the original struct by providing values for skipped fields.
                ///
                /// This method takes the skipped fields as parameters and reconstructs
                /// the original struct with non-skipped fields from `self`.
                ///
                /// Returns an error if any non-skipped wrapped field is `None` (unless it has a default).
                pub fn into_original(self, #(#skipped_params),*) -> Result<#original_ident #ty_generics, ::#lib_path::UnwrappedError> {
                    Ok(#original_ident {
                        #(#into_original_fields),*
                    })
                }
            }
        }
    } else {
        quote! {
            #(#struct_attrs)*
            #derive_output
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
}
