use std::collections::HashMap;

use quote::{format_ident, quote};
use syn::DeriveInput;

/// Check if a type is `Option<T>` and return the inner type if so
pub fn is_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
        && seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Extract the struct data from a DeriveInput, panicking if it's not a struct
pub fn get_struct_data(input: &DeriveInput) -> &syn::DataStruct {
    if let syn::Data::Struct(s) = &input.data {
        s
    } else {
        unreachable!("Expected a struct")
    }
}

/// Common options struct for both Unwrapped and Wrapped
#[derive(Clone, Debug)]
pub struct CommonOpts {
    pub name: Option<syn::Ident>,
    pub prefix: Option<syn::Ident>,
    pub suffix: Option<syn::Ident>,
    pub struct_derives: Vec<proc_macro2::TokenStream>,
    pub struct_attrs: Vec<proc_macro2::TokenStream>,
    pub field_attrs: HashMap<String, Vec<proc_macro2::TokenStream>>,
}

impl CommonOpts {
    /// Generate the new identifier based on name/prefix/suffix, with a fallback suffix if unchanged
    pub fn generate_ident(&self, original_ident: &syn::Ident, fallback_suffix: &str) -> syn::Ident {
        let base = self.name.as_ref().unwrap_or(original_ident);
        let prefix = self
            .prefix
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let suffix = self
            .suffix
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_default();
        let new = format_ident!("{}{}{}", prefix, base, suffix);

        if &new == original_ident {
            format_ident!("{}{}", original_ident, fallback_suffix)
        } else {
            new
        }
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
}

impl Default for CommonOpts {
    fn default() -> Self {
        Self {
            name: None,
            prefix: None,
            suffix: None,
            struct_derives: Vec::new(),
            struct_attrs: Vec::new(),
            field_attrs: HashMap::new(),
        }
    }
}

/// Per-field options for procedural macro usage
#[derive(Clone, Debug, Default)]
pub struct FieldProcOpts {
    pub transform: bool,
    pub attrs: Vec<proc_macro2::TokenStream>,
    pub default_expr: Option<proc_macro2::TokenStream>,
}

impl FieldProcOpts {
    pub fn new(transform: bool) -> Self {
        Self {
            transform,
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

/// Common procedural usage options
#[derive(Clone, Debug, Default)]
pub struct ProcUsageOpts {
    pub fields_to_transform: HashMap<String, bool>,
    pub lib_holder_name: Option<syn::Ident>,
    pub field_opts: HashMap<String, FieldProcOpts>,
    pub field_attr_fn: Option<fn(&syn::Field) -> Option<proc_macro2::TokenStream>>,
}

impl ProcUsageOpts {
    pub fn new(
        fields_to_transform: HashMap<String, bool>,
        lib_holder_name: Option<syn::Ident>,
    ) -> Self {
        Self {
            fields_to_transform,
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
}

/// Collect field attributes from all sources
pub fn collect_field_attrs(
    f: &syn::Field,
    opts: &CommonOpts,
    proc_usage_opts: &ProcUsageOpts,
) -> Vec<proc_macro2::TokenStream> {
    let name_str = f.ident.as_ref().unwrap().to_string();
    let mut attrs = Vec::new();

    // From CommonOpts field_attrs
    if let Some(opts_attrs) = opts.field_attrs.get(&name_str) {
        attrs.extend(opts_attrs.clone());
    }

    // From ProcUsageOpts field_opts
    if let Some(field_opts) = proc_usage_opts.field_opts.get(&name_str) {
        attrs.extend(field_opts.attrs.clone());
    }

    // From dynamic field_attr_fn
    if let Some(attr_fn) = proc_usage_opts.field_attr_fn
        && let Some(attr) = attr_fn(f)
    {
        attrs.push(attr);
    }

    attrs
}

/// Build the derive output based on struct_derives
pub fn build_derive_output(
    struct_derives: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    if struct_derives.is_empty() {
        quote! { #[derive()] }
    } else {
        quote! { #[derive(#(#struct_derives),*)] }
    }
}
