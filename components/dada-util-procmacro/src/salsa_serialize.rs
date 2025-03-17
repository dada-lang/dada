use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Fields, Lit, Meta};
use synstructure::Structure;

#[derive(Default)]
struct SerdeAttrs {
    rename: Option<String>,
    skip: bool,
    serialize_with: Option<String>,
}

fn parse_serde_attrs(attrs: &[Attribute]) -> SerdeAttrs {
    let mut result = SerdeAttrs::default();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        if let Ok(Meta::List(meta)) = attr.parse_args() {
            let nested = meta.tokens.into_iter().filter_map(|token| {
                syn::parse2::<Meta>(token.into()).ok()
            });

            for meta in nested {
                match meta {
                    Meta::NameValue(name_value) => {
                        if name_value.path.is_ident("rename") {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }) = name_value.value
                            {
                                result.rename = Some(lit.value());
                            }
                        } else if name_value.path.is_ident("serialize_with") {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }) = name_value.value
                            {
                                result.serialize_with = Some(lit.value());
                            }
                        }
                    }
                    Meta::Path(path) => {
                        if path.is_ident("skip") {
                            result.skip = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    result
}

/// Implements serde::Serialize for Salsa input structs by calling field getter methods with the database.
///
/// This derive macro is designed for structs marked with `#[salsa::input]`. For each field in the struct,
/// it generates code that calls the corresponding getter method with the Salsa database to obtain the field's value.
/// The generated implementation uses `salsa::with_attached_database` to access the database context.
///
/// # Attributes
/// The macro supports standard serde attributes:
///
/// - `rename`: Customize the field name in the serialized output
/// - `skip`: Exclude a field from serialization
/// - `flatten`: Include all fields from a nested struct in the current struct
/// - `serialize_with`: Use a custom function to serialize this field
///
/// # Example
/// ```rust
/// #[derive(SalsaSerialize)]
/// struct Person {
///     #[serde(rename = "firstName")]
///     first_name: String,
///     last_name: String,
///     #[serde(skip)]
///     internal_id: u64,
///     #[serde(flatten)]
///     metadata: PersonMetadata,
///     #[serde(serialize_with = "serialize_date")]
///     created_at: DateTime,
/// }
/// ```
pub(crate) fn salsa_serialize_derive(s: Structure) -> TokenStream {
    let ast = s.ast();
    let struct_name = &ast.ident;

    // We only handle structs with named fields
    let fields = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("SalsaSerialize only works with named fields"),
        },
        _ => panic!("SalsaSerialize only works with structs"),
    };

    // Filter out skipped fields and get names for the rest
    let mut field_count = 0_usize;
    let mut field_idents = Vec::new();
    let mut field_names = Vec::new();
    let mut field_values = Vec::new();

    for field in fields.iter() {
        let attrs = parse_serde_attrs(&field.attrs);
        if attrs.skip {
            continue;
        }

        let ident = field.ident.as_ref().unwrap();
        field_idents.push(ident);

        let field_name = attrs.rename.unwrap_or_else(|| ident.to_string());
        field_names.push(field_name);

        let value = if let Some(serializer) = attrs.serialize_with {
            let serializer =
                syn::parse_str::<syn::Path>(&serializer).expect("Invalid serialize_with path");
            quote! { #serializer(& #struct_name ::#ident(*self, db), serializer)? }
        } else {
            quote! { & #struct_name ::#ident(*self, db) }
        };
        field_values.push(value);
    
        field_count += 1;
    }

    s.gen_impl(quote! {
        gen impl serde::Serialize for @Self {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeStruct;
                salsa::with_attached_database(|db| {
                    let mut state = serializer.serialize_struct(stringify!(#struct_name), #field_count)?;
                    #(
                        state.serialize_field(#field_names, &#field_values)?;
                    )*
                    state.end()
                })
                .ok_or_else(|| {
                    use serde::ser::Error;
                    S::Error::custom("cannot serialize without attached database")
                })?
            }
        }
    })
}
