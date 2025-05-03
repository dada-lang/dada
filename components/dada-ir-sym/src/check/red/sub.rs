use crate::check::env::Env;

use super::{RedChain, RedLink};

pub fn chain_sub_chain<'db>(
    env: &Env<'db>,
    lower_chain: RedChain<'db>,
    upper_chain: RedChain<'db>,
) -> bool {
    let db = env.db();
    links_sub_links(env, lower_chain.links(db), upper_chain.links(db))
}

fn links_sub_links<'db>(
    env: &Env<'db>,
    lower_links: &[RedLink<'db>],
    upper_links: &[RedLink<'db>],
) -> bool {
    macro_rules! rules {
        ($($pat:pat => $cond:expr,)*) => {
            match (lower_links, upper_links) {
                $(
                    $pat if $cond => true,
                )*
                _ => false,
            }
        };
    }

    rules! {
        ([], []) => true,

        ([RedLink::Our], links_u) => RedLink::are_copy(env, links_u),

        ([RedLink::Our, tail_l @ ..], [head_u, tail_u @ ..]) => {
            head_u.is_copy(env)
            && links_sub_links(env, tail_l, tail_u)
        },

        (
            [
                RedLink::Ref(_, place_l),
                tail_l @ ..,
            ],
            [
                RedLink::Ref(_, place_u),
                tail_u @ ..,
            ],
        )
        | (
            [
                RedLink::Mut(_, place_l),
                tail_l @ ..,
            ],
            [
                RedLink::Mut(_, place_u),
                tail_u @ ..,
            ],
        )
        | (
            [
                RedLink::Ref(_, place_l),
                tail_l @ ..,
            ],
            [
                RedLink::Our,
                RedLink::Mut(_, place_u),
                tail_u @ ..,
            ],
        ) => {
            place_u.is_prefix_of(env.db(), *place_l)
            && links_sub_links(env, tail_l, tail_u)
        },

        ([RedLink::Var(var_l), tail_l @ ..], [RedLink::Var(var_u), tail_u @ ..]) => {
            var_l == var_u && links_sub_links(env, tail_l, tail_u)
        },


    }
}
