use dada_ast::word::Word;
use std::sync::Arc;

#[salsa::jar(Manifest)]
pub struct Jar(source_text);

pub trait Manifest: salsa::DbWithJar<Jar> {
    fn source_text(&self, filename: Word) -> &Arc<String>;
    fn set_source_text(&mut self, filename: Word, text: Arc<String>);
}

impl<T> Manifest for T
where
    T: salsa::DbWithJar<Jar>,
{
    fn source_text(&self, filename: Word) -> &Arc<String> {
        source_text::get(self, filename)
    }

    fn set_source_text(&mut self, filename: Word, value: Arc<String>) {
        source_text::set(self, filename, value)
    }
}

#[salsa::memoized(in Jar)]
fn source_text(_db: &dyn Manifest, _filename: Word) -> Arc<String> {
    panic!("input")
}
