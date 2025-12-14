use syn::{ImplItem, Path, Type};

/// Information extracted from a trait method signature
#[allow(dead_code)]
#[derive(Clone)]
pub struct TraitMethodInfo {
    pub name: syn::Ident,
    pub input_type: Type,
    pub output_type: Type,
    pub error_type: Type,
}

/// Parse a trait and extract all async methods with their signatures
#[allow(dead_code)]
pub fn parse_trait_methods(_trait_path: &Path) -> Vec<TraitMethodInfo> {
    // In a procedural macro context, we can't directly load and parse the trait
    // at compile time from another module. Instead, we expect the trait to be
    // in scope and we'll generate code that will fail to compile if methods don't match.
    //
    // For validation purposes in this implementation, we return empty with a note
    // that actual validation happens at compile time when the generated code is checked.
    //
    // A more sophisticated approach would use a build-time analysis or require
    // the trait definition to be provided inline, but that's beyond scope here.

    // Return empty - validation will happen when generated code compiles
    vec![]
}

/// Parse the api attribute from macro arguments
pub fn parse_path_attribute(attr_name: &str, meta: &syn::Meta) -> syn::Result<Option<Path>> {
    match meta {
        syn::Meta::NameValue(nv) if nv.path.is_ident(attr_name) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &nv.value
            {
                let path: Path = lit_str.parse()?;
                Ok(Some(path))
            } else {
                Err(syn::Error::new_spanned(
                    &nv.value,
                    format!("{} must be a string literal path", attr_name),
                ))
            }
        }
        _ => Ok(None),
    }
}

/// Parse string attribute from macro arguments
pub fn parse_string_attribute(attr_name: &str, meta: &syn::Meta) -> syn::Result<Option<String>> {
    match meta {
        syn::Meta::NameValue(nv) if nv.path.is_ident(attr_name) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &nv.value
            {
                Ok(Some(lit_str.value()))
            } else {
                Err(syn::Error::new_spanned(
                    &nv.value,
                    format!("{} must be a string literal", attr_name),
                ))
            }
        }
        _ => Ok(None),
    }
}

/// Validate that a path looks reasonable (has at least one segment)
pub fn validate_path(path: &Path, context: &str) -> syn::Result<()> {
    if path.segments.is_empty() {
        return Err(syn::Error::new_spanned(
            path,
            format!("Invalid path for {}", context),
        ));
    }
    Ok(())
}

/// Extract method information from an impl block (for service validation)
#[allow(dead_code)]
pub fn extract_impl_methods(impl_block: &syn::ItemImpl) -> Vec<syn::Ident> {
    impl_block
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                Some(method.sig.ident.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Parse a type from a string (for tonic client types)
#[allow(dead_code)]
pub fn parse_type_from_string(s: &str) -> syn::Result<Type> {
    syn::parse_str(s)
}
