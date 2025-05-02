use crate::check::env::Env;

use super::RedLink;

struct NoGlb;
type Glb<T> = Result<T, NoGlb>;

fn glb_link<'db>(env: &Env<'db>, l1: RedLink<'db>, l2: RedLink<'db>) -> Glb<RedLink<'db>> {}
