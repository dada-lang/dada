#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

use url::Url;

#[macro_use]
mod macro_rules;

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod span;

#[salsa::db]
pub trait Db: salsa::Database {
    /// Convert the url into a string suitable for showing the user.
    fn url_display(&self, url: &Url) -> String;
}
