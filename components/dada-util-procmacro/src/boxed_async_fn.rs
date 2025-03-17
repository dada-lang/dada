mod parse;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Transforms an async fn to return a `Box<dyn Future<Output = T>>`.
///
/// Originally based on the [`async_recursion`](https://crates.io/crates/async-recursion) crate
/// authored by Robert Usher and licensed under MIT/APACHE-2.0.
pub fn boxed_async_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let parse::AsyncItem(mut item) = parse_macro_input!(input as parse::AsyncItem);
    let _args = parse_macro_input!(args as syn::parse::Nothing);

    let block = item.block;
    item.block = syn::parse2(quote!({Box::pin(async move #block).await})).unwrap();

    TokenStream::from(quote!(#item))
}
