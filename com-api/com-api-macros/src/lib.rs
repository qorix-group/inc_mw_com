use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, Generics, Meta, Type, TypePath, parse_macro_input, parse_quote,
};

const PRIMITIVES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
    "char",
];

#[proc_macro_derive(Reloc)]
pub fn derive_reloc(input: TokenStream) -> TokenStream {
    let mInput = parse_macro_input!(input as DeriveInput);
    let mName = &mInput.ident;
    let mut mGenerics = add_reloc_bounds(mInput.generics.clone());

    // Ensure #[repr(C)] on the struct itself
    if !has_repr_c(&mInput.attrs) {
        return syn::Error::new_spanned(
            &mName,
            "The #[derive(Reloc)] macro requires #[repr(C)] on the type",
        )
        .to_compile_error()
        .into();
    }

    // Collect field types to add as where bounds
    let field_types = collect_field_types(&mInput.data, &mInput.generics);
    if !field_types.is_empty() {
        let where_clause = mGenerics.make_where_clause();
        for ty in field_types {
            where_clause
                .predicates
                .push(parse_quote!(#ty: com_api::prelude::Reloc));
        }
    }

    let (impl_generics, type_generics, where_clause) = mGenerics.split_for_impl();

    let expanded = quote! {
        unsafe impl #impl_generics iceoryx2::prelude::ZeroCopySend for #mName #type_generics #where_clause {}
        unsafe impl #impl_generics com_api::prelude::Reloc for #mName #type_generics #where_clause {}
    };

    expanded.into()
}

/// Add `T: Reloc` bounds to all generic parameters
fn add_reloc_bounds(mut generics: Generics) -> Generics {
    for param in generics.type_params_mut() {
        param.bounds.push(parse_quote!(com_api::prelude::Reloc));
    }
    generics
}

/// Check for #[repr(C)]
fn has_repr_c(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("repr") {
            if let Meta::List(list) = &attr.meta {
                let tokens = list.tokens.to_string();
                if tokens.split(',').any(|t| t.trim() == "C") {
                    return true;
                }
            }
        }
    }
    false
}

/// Collect field types, skipping generic parameters and primitives
fn collect_field_types<'a>(data: &'a Data, generics: &Generics) -> Vec<&'a Type> {
    let mut out = Vec::new();
    let generic_idents: Vec<_> = generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect();

    if let Data::Struct(data_struct) = data {
        let fields_iter = match &data_struct.fields {
            Fields::Named(fields) => fields.named.iter(),
            Fields::Unnamed(fields) => fields.unnamed.iter(),
            Fields::Unit => return out,
        };

        for f in fields_iter {
            if let Type::Path(TypePath { path, .. }) = &f.ty {
                // Skip generic parameters
                if path.segments.len() == 1 {
                    let ident_str = path.segments[0].ident.to_string();
                    if generic_idents.contains(&ident_str)
                    // || PRIMITIVES.contains(&ident_str.as_str())
                    {
                        continue;
                    }
                }
            }
            out.push(&f.ty);
        }
    }

    out
}
