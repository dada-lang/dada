use crate::{
    check::{env::Env, predicates::Predicate},
    ir::types::SymPlace,
};

use super::RedLink;

struct NoGlb;
type Glb<T> = Result<T, NoGlb>;

fn glb_link<'db>(env: &Env<'db>, l1: RedLink<'db>, l2: RedLink<'db>) -> Glb<RedLink<'db>> {
    match (l1, l2) {
        (RedLink::Our, RedLink::RefDead(_))
        | (RedLink::Our, RedLink::RefLive(_))
        | (RedLink::RefDead(_), RedLink::Our)
        | (RedLink::RefLive(_), RedLink::Our)
        | (RedLink::Our, RedLink::Our) => Ok(RedLink::Our),
        (RedLink::Our, RedLink::Var(v)) | (RedLink::Var(v), RedLink::Our)
            if env.var_is_declared_to_be(v, Predicate::Copy) =>
        {
            Ok(RedLink::Our)
        }
        (RedLink::RefDead(p1), RedLink::RefDead(p2))
        | (RedLink::RefLive(p1), RedLink::RefDead(p2))
        | (RedLink::RefDead(p1), RedLink::RefLive(p2)) => match glb_places(env, p1, p2) {
            Ok(p3) => Ok(RedLink::RefDead(p3)),
            Err(NoGlb) => Ok(RedLink::Our),
        },
        (RedLink::RefLive(p1), RedLink::RefLive(p2)) => match glb_places(env, p1, p2) {
            Ok(p3) => Ok(RedLink::RefLive(p3)),
            Err(NoGlb) => Ok(RedLink::Our),
        },
        (RedLink::MutDead(p1), RedLink::MutDead(p2))
        | (RedLink::MutLive(p1), RedLink::MutDead(p2))
        | (RedLink::MutDead(p1), RedLink::MutLive(p2)) => {
            let p3 = glb_places(env, p1, p2)?;
            Ok(RedLink::MutDead(p3))
        }
        (RedLink::MutLive(p1), RedLink::MutLive(p2)) => {
            let p3 = glb_places(env, p1, p2)?;
            Ok(RedLink::MutDead(p3))
        }
        (RedLink::Var(v1), RedLink::Var(v2)) => {
            if v1 == v2 {
                Ok(RedLink::Var(v1))
            } else {
                Err(NoGlb)
            }
        }

        // Subtle: this *would* be true for vars known to equal our, but we already canonical that
        (RedLink::RefLive(_) | RedLink::RefDead(_), RedLink::Var(_))
        | (RedLink::Var(_), RedLink::RefLive(_) | RedLink::RefDead(_)) => Err(NoGlb),

        // Subtle: this *would* be true for vars known to equal our, but we already canonical that
        (RedLink::Our, RedLink::Var(_)) | (RedLink::Var(_), RedLink::Our) => Err(NoGlb),

        // No type is a subtype of both our/mut at same time.
        (RedLink::Our, RedLink::MutLive(_) | RedLink::MutDead(_))
        | (RedLink::MutLive(_) | RedLink::MutDead(_), RedLink::Our) => Err(NoGlb),

        // No type is a subtype of both ref/mut at same time.
        (RedLink::RefLive(_) | RedLink::RefDead(_), RedLink::MutLive(_) | RedLink::MutDead(_))
        | (RedLink::MutLive(_) | RedLink::MutDead(_), RedLink::RefLive(_) | RedLink::RefDead(_)) => {
            Err(NoGlb)
        }

        // No type is a subtype of both var/mut at same time.
        (RedLink::MutLive(_) | RedLink::MutDead(_), RedLink::Var(_))
        | (RedLink::Var(_), RedLink::MutLive(_) | RedLink::MutDead(_)) => Err(NoGlb),
    }
}

fn glb_places<'db>(env: &Env<'db>, p1: SymPlace<'db>, p2: SymPlace<'db>) -> Glb<SymPlace<'db>> {
    if p1.is_prefix_of(env.db(), p2) {
        Ok(p2)
    } else if p2.is_prefix_of(env.db(), p1) {
        Ok(p1)
    } else {
        Err(NoGlb)
    }
}
