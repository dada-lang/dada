use std::sync::Arc;

mod text;
pub use text::Text;

#[salsa::jar(Manifest)]
pub struct Jar(source_text, text::Text);

pub trait Manifest: salsa::DbWithJar<Jar> {
    fn source_text(&self, filename: Text) -> &Arc<String>;
    fn set_source_text(&mut self, filename: Text, text: Arc<String>);
    fn intern_text(&self, s: String) -> Text;
    fn data(&self, t: Text) -> &str;
}

impl<T> Manifest for T
where
    T: salsa::DbWithJar<Jar>,
{
    fn source_text(&self, filename: text::Text) -> &Arc<String> {
        source_text::get(self, filename)
    }

    fn set_source_text(&mut self, filename: text::Text, value: Arc<String>) {
        source_text::set(self, filename, value)
    }

    fn intern_text(&self, string: String) -> Text {
        text::TextData { string }.intern(self)
    }

    fn data(&self, text: Text) -> &str {
        &text.data(self).string
    }
}

#[salsa::memoized(in Jar)]
fn source_text(_db: &dyn Manifest, _filename: text::Text) -> Arc<String> {
    panic!("input")
}
