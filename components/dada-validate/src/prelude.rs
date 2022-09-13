use dada_ir::{
    code::validated, function::Function, input_file::InputFile, item::Item, parameter::Parameter,
};

#[extension_trait::extension_trait]
pub impl DadaValidateInputFileExt for InputFile {
    fn validate_root(self, db: &dyn crate::Db) {
        crate::validate::root_definitions(db, self);
    }
}

#[extension_trait::extension_trait]
pub impl DadaValidateFunctionExt for Function {
    fn parameters(self, db: &dyn crate::Db) -> &Vec<Parameter> {
        crate::validate::validate_parameter(db, self)
    }

    fn validated_tree(self, db: &dyn crate::Db) -> validated::Tree {
        crate::validate::validate_function(db, self)
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
