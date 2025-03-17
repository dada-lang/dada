use proc_macro2::Span;
use syn::{
    ItemFn,
    parse::{Error, Parse, ParseStream, Result},
};

pub struct AsyncItem(pub ItemFn);

impl Parse for AsyncItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let item: ItemFn = input.parse()?;

        // Check that this is an async function
        if item.sig.asyncness.is_none() {
            return Err(Error::new(Span::call_site(), "expected an async function"));
        }

        Ok(AsyncItem(item))
    }
}
