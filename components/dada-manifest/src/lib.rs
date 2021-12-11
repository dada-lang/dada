use dada_ir::word::Word;

#[salsa::jar(Db)]
pub struct Jar(source_text);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> {}

#[salsa::memoized(in Jar ref)]
pub fn source_text(_db: &dyn Db, _filename: Word) -> String {
    panic!("input")
}
