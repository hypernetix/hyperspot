use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Lit, Meta};

pub fn expand_derive_odata_schema(input: &DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => abort!(input, "ODataSchema only supports structs with named fields"),
        },
        _ => abort!(input, "ODataSchema can only be derived for structs"),
    };

    let mut field_variants = Vec::new();
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut field_constructors = Vec::new();

    for field in fields {
        let Some(field_ident) = field.ident.as_ref() else {
            abort!(field, "ODataSchema requires named fields");
        };
        let field_type = &field.ty;

        let odata_name = extract_odata_name(&field.attrs, field_ident);

        let variant_name = to_pascal_case(&field_ident.to_string());
        let variant_ident = Ident::new(&variant_name, field_ident.span());

        field_variants.push(variant_ident.clone());
        field_names.push((variant_ident.clone(), odata_name.clone()));
        field_types.push((field_ident.clone(), field_type.clone()));
        field_constructors.push((field_ident.clone(), variant_ident, field_type.clone()));
    }

    let field_enum_name = Ident::new(&format!("{struct_name}Field"), struct_name.span());
    let schema_struct_name = Ident::new(&format!("{struct_name}Schema"), struct_name.span());
    let module_name = Ident::new(&struct_name.to_string().to_snake_case(), struct_name.span());

    let field_enum = quote! {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub enum #field_enum_name {
            #(#field_variants,)*
        }
    };

    let field_name_arms = field_names.iter().map(|(variant, name)| {
        quote! {
            #field_enum_name::#variant => #name
        }
    });

    let schema_impl = quote! {
        pub struct #schema_struct_name;

        impl ::modkit_odata::schema::Schema for #schema_struct_name {
            type Field = #field_enum_name;

            fn field_name(field: Self::Field) -> &'static str {
                match field {
                    #(#field_name_arms,)*
                }
            }
        }
    };

    let constructor_fns = field_constructors
        .iter()
        .map(|(field_ident, variant, field_type)| {
            let fn_name = field_ident.clone();
            quote! {
                #[must_use]
                pub fn #fn_name() -> ::modkit_odata::schema::FieldRef<super::#schema_struct_name, #field_type> {
                    ::modkit_odata::schema::FieldRef::new(super::#field_enum_name::#variant)
                }
            }
        });

    let constructor_module = quote! {
        pub mod #module_name {
            #(#constructor_fns)*
        }
    };

    quote! {
        #field_enum
        #schema_impl
        #constructor_module
    }
}

fn extract_odata_name(attrs: &[syn::Attribute], field_ident: &Ident) -> String {
    for attr in attrs {
        if attr.path().is_ident("odata") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = &meta_list.tokens;
                let parsed: Result<Meta, _> = syn::parse2(tokens.clone());

                if let Ok(Meta::NameValue(nv)) = parsed {
                    if nv.path.is_ident("name") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                return lit_str.value();
                            }
                        }
                    }
                }
            }
        }
    }

    field_ident.to_string()
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}
