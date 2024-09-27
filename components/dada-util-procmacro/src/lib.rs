use proc_macro::TokenStream;
use quote::quote;
use synstructure::decl_derive;

decl_derive!([FromImpls, attributes(no_from_impl)] => from_impls_derive);

fn from_impls_derive(s: synstructure::Structure) -> TokenStream {
    let result = s
        .variants()
        .iter()
        .map(|variant| {
            let variant_name = &variant.ast().ident;
            let fields = &variant.ast().fields;

            for attr in variant
                .ast()
                .attrs
                .iter()
                .filter(|a| a.meta.path().is_ident("no_from_impl"))
            {
                if let Err(_) = attr.meta.require_path_only() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "`no_from_impl` does not accept arguments",
                    ));
                }
            }

            if variant
                .ast()
                .attrs
                .iter()
                .any(|a| a.meta.path().is_ident("no_from_impl"))
            {
                return Ok(quote!());
            }

            if fields.len() != 1 {
                return Err(syn::Error::new_spanned(
                    &variant.ast().ident,
                    "each variant must have exactly one field",
                ));
            }

            let field_ty = &fields.iter().next().unwrap().ty;
            Ok(s.gen_impl(quote! {
                gen impl From<#field_ty> for @Self {
                    fn from(value: #field_ty) -> Self {
                        Self::#variant_name(value)
                    }
                }

            }))
        })
        .collect::<syn::Result<proc_macro2::TokenStream>>();

    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
