use dada_util::vecset::VecSet;

use crate::ir::red_terms::RedTerm;

use predicates::Predicate;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Bounds<'db> {
    pub(crate) is: VecSet<Predicate>,
    pub(crate) isnt: VecSet<Predicate>,
    pub(crate) lower_bound: Option<RedTerm<'db>>,
    pub(crate) upper_bound: Option<RedTerm<'db>>,
}
