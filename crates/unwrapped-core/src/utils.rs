use std::collections::{HashMap, HashSet};

use ident_case::RenameRule;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{DeriveInput, Expr, GenericParam, Meta, Path};

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
#[derive(Clone, Debug, Default)]
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

/// Per-field options for procedural macro usage
#[derive(Clone, Debug, Default)]
pub struct FieldProcOpts {
    pub transform: bool,
    pub attrs: Vec<proc_macro2::TokenStream>,
}

impl FieldProcOpts {
    pub fn new(transform: bool) -> Self {
        Self {
            transform,
            attrs: Vec::new(),
        }
    }

    pub fn with_attr(mut self, tokens: impl Into<proc_macro2::TokenStream>) -> Self {
        self.attrs.push(tokens.into());
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

#[derive(Default)]
struct BonBuilderConfig {
    builder_type: Option<syn::Ident>,
    state_mod: Option<syn::Ident>,
}

pub(crate) struct BonBuilderInfo {
    pub(crate) builder_ident: syn::Ident,
    pub(crate) state_mod_ident: syn::Ident,
}

fn derives_builder(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let paths = attr
            .parse_args_with(syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated);
        let Ok(paths) = paths else {
            return false;
        };
        paths.iter().any(|path| {
            path.segments
                .last()
                .map(|seg| seg.ident == "Builder")
                .unwrap_or(false)
        })
    })
}

fn has_builder_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("builder"))
}

fn parse_builder_config(attrs: &[syn::Attribute]) -> BonBuilderConfig {
    let mut config = BonBuilderConfig::default();

    for attr in attrs {
        if !attr.path().is_ident("builder") {
            continue;
        }
        let meta = match &attr.meta {
            Meta::List(list) => list,
            _ => continue,
        };
        let Some(nested) = parse_meta_list(meta.tokens.clone()) else {
            continue;
        };

        for item in nested {
            if let Some(ident) = parse_builder_item_ident(&item, "builder_type") {
                config.builder_type = Some(ident);
            }
            if let Some(ident) = parse_builder_item_ident(&item, "state_mod") {
                config.state_mod = Some(ident);
            }
        }
    }

    config
}

fn parse_builder_item_ident(item: &Meta, key: &str) -> Option<syn::Ident> {
    match item {
        Meta::NameValue(nv) if nv.path.is_ident(key) => parse_meta_value_ident(&nv.value),
        Meta::List(list) if list.path.is_ident(key) => {
            let nested = parse_meta_list(list.tokens.clone())?;
            for inner in nested {
                if let Meta::NameValue(nv) = inner
                    && nv.path.is_ident("name")
                    && let Some(ident) = parse_meta_value_ident(&nv.value)
                {
                    return Some(ident);
                }
            }
            None
        },
        _ => None,
    }
}

fn parse_meta_list(
    tokens: proc_macro2::TokenStream,
) -> Option<syn::punctuated::Punctuated<Meta, syn::Token![,]>> {
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    parser.parse2(tokens).ok()
}

fn parse_meta_value_ident(expr: &Expr) -> Option<syn::Ident> {
    match expr {
        Expr::Path(path) => path.path.segments.last().map(|seg| seg.ident.clone()),
        Expr::Lit(lit) => {
            if let syn::Lit::Str(lit_str) = &lit.lit {
                syn::parse_str::<syn::Ident>(&lit_str.value()).ok()
            } else {
                None
            }
        },
        _ => None,
    }
}

pub(crate) fn bon_builder_info(input: &DeriveInput) -> Option<BonBuilderInfo> {
    if !derives_builder(&input.attrs) && !has_builder_attr(&input.attrs) {
        return None;
    }

    let config = parse_builder_config(&input.attrs);

    let builder_ident = config
        .builder_type
        .unwrap_or_else(|| format_ident!("{}Builder", input.ident));

    let state_mod_ident = config
        .state_mod
        .unwrap_or_else(|| pascal_to_snake_ident(&builder_ident));

    Some(BonBuilderInfo {
        builder_ident,
        state_mod_ident,
    })
}

pub(crate) fn raw_ident_name(ident: &syn::Ident) -> String {
    ident
        .to_string()
        .strip_prefix("r#")
        .unwrap_or(&ident.to_string())
        .to_string()
}

fn pascal_to_snake_ident(ident: &syn::Ident) -> syn::Ident {
    let renamed = RenameRule::SnakeCase.apply_to_variant(raw_ident_name(ident));
    syn::Ident::new(&renamed, proc_macro2::Span::call_site())
}

pub(crate) fn snake_to_pascal_ident(ident: &syn::Ident) -> syn::Ident {
    let renamed = RenameRule::PascalCase.apply_to_field(raw_ident_name(ident));
    syn::Ident::new(&renamed, proc_macro2::Span::call_site())
}

pub(crate) fn unique_state_ident(generics: &syn::Generics) -> syn::Ident {
    let mut existing = HashSet::new();
    for param in generics.params.iter() {
        match param {
            GenericParam::Type(param) => {
                existing.insert(param.ident.to_string());
            },
            GenericParam::Lifetime(param) => {
                existing.insert(param.lifetime.ident.to_string());
            },
            GenericParam::Const(param) => {
                existing.insert(param.ident.to_string());
            },
        }
    }

    let base = "__UnwrappedBuilderState";
    if !existing.contains(base) {
        return syn::Ident::new(base, proc_macro2::Span::call_site());
    }

    let mut i = 0;
    loop {
        let candidate = format!("{base}{i}");
        if !existing.contains(&candidate) {
            return syn::Ident::new(&candidate, proc_macro2::Span::call_site());
        }
        i += 1;
    }
}

pub(crate) fn generic_args(generics: &syn::Generics) -> Vec<proc_macro2::TokenStream> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote! { #lifetime }
            },
            GenericParam::Const(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
        })
        .collect()
}
