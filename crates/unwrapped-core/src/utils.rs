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
