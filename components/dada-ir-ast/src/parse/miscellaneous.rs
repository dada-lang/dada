use crate::ast::Path;

use super::{OrNotPresent, ParseFail, ParseTokens, TokenStream};

impl<'db> ParseTokens<'db> for Path<'db> {
    fn parse(
        _db: &'db dyn crate::Db,
        tokens: &mut TokenStream<'_, 'db>,
    ) -> Result<Self, ParseFail<'db>> {
        let id = tokens.eat_spanned_id().or_not_present()?;
        let mut ids = vec![id];

        while tokens.eat_op('.').is_ok() {
            if let Ok(id) = tokens.eat_spanned_id() {
                ids.push(id);
            } else {
                break;
            }
        }

        Ok(Path { ids })
    }
}
