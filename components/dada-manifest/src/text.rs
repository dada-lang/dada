use super::{Jar, Manifest};

#[salsa::interned(Text in Jar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextData {
    pub string: String,
}

impl Text {
    pub fn from(db: &dyn Manifest, string: String) -> Self {
        TextData { string }.intern(db)
    }
}
