use dada_ir::{
    class::Class,
    code::validated,
    function::Function,
    input_file::InputFile,
    item::Item,
    signature::{ClassStructure, Signature},
};

#[extension_trait::extension_trait]
pub impl DadaValidateInputFileExt for InputFile {
    fn validate_root(self, db: &dyn crate::Db) {
        crate::validate::root_definitions(db, self);
    }
}

#[extension_trait::extension_trait]
pub impl DadaValidateFunctionExt for Function {
    fn signature(self, db: &dyn crate::Db) -> &Signature {
        crate::signature::validate_function_signature(db, self)
    }

    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree {
        crate::validate::validate_function(db, self)
    }
}

#[extension_trait::extension_trait]
pub impl DadaValidateClassExt for Class {
    fn signature(self, db: &dyn crate::Db) -> &Signature {
        crate::signature::validate_class_signature(db, self)
    }

    fn structure(self, db: &dyn crate::Db) -> &ClassStructure {
        crate::signature::validate_class_structure(db, self)
    }
}

#[extension_trait::extension_trait]
pub impl DadaValidateItemExt for Item {
    fn validated_tree(self, db: &dyn crate::Db) -> Option<validated::Tree> {
        match self {
            Item::Function(f) => Some(f.validated_tree(db)),
            Item::Class(_) => None,
        }
    }
}
