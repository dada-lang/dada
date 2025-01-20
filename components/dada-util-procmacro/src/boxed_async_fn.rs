mod expand;
mod parse;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Transforms an async fn to return a `Box<dyn Future<Output = T>>`.
///
/// Adapted from the [`async_recursion`](https://crates.io/crates/async-recursion) crate authored by
/// Robert Usher and licensed under MIT/APACHE-2.0.
pub fn boxed_async_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as parse::AsyncItem);
    let args = parse_macro_input!(args as parse::RecursionArgs);

    expand::expand(&mut item, &args);

    TokenStream::from(quote!(#item))
}
