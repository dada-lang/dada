use dada_util::Text;

#[salsa::input]
pub struct SourceFile {
    pub path: Text,

    #[return_ref]
    pub contents: String,
}
