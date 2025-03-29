use axum::http::header::ACCEPT;

use crate::server::State;

fn check_accept_header(headers: &axum::http::header::HeaderMap) -> anyhow::Result<()> {
    // Check the request mime type
    let Some(value) = headers.get(&ACCEPT) else {
        anyhow::bail!("header `{ACCEPT}` required");
    };

    if value.to_str()? != "application/json" {
        anyhow::bail!("this endpoint only returns `application/json`");
    }

    Ok(())
}

pub async fn events(
    headers: &axum::http::header::HeaderMap,
    state: &State,
) -> anyhow::Result<Vec<crate::root::RootEvent>> {
    check_accept_header(headers)?;
    crate::root::root_data(&state).await
}

pub async fn try_event_data(
    headers: &axum::http::header::HeaderMap,
    event_index: usize,
    state: &State,
) -> anyhow::Result<serde_json::Value> {
    check_accept_header(headers)?;
    crate::view::try_view_data(event_index, state).await
}
