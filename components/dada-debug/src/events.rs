use axum::http::header::ACCEPT;

use crate::server::State;

pub async fn events(
    headers: &axum::http::header::HeaderMap,
    state: &State,
) -> anyhow::Result<Vec<crate::root::RootEvent>> {
    // Check the request mime type
    let Some(accept) = headers.get(&ACCEPT) else {
        anyhow::bail!("header `{ACCEPT}` required");
    };

    if accept.to_str()? != "application/json" {
        anyhow::bail!("this endpoint only returns `application/json`");
    }

    crate::root::root_data(&state).await
}
