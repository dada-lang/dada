use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    time::Duration,
};

use axum::{Json, Router, routing::get};
use dada_ir_ast::DebugEvent;
use serde::{Deserialize, Serialize};

pub fn main(port: u32, debug_rx: Receiver<DebugEvent>) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(main_async(port, debug_rx))?;
    Ok(())
}

async fn main_async(port: u32, debug_rx: Receiver<DebugEvent>) -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let state = Arc::new(State {
        debug_events: Default::default(),
        shutdown: Default::default(),
    });

    std::thread::spawn({
        let state = state.clone();
        move || record_events(debug_rx, state)
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/view/{event_index}", get(view))
        .route("/assets/{file}", get(assets))
        .route("/source/{*path}", get(source))
        .route("/events", get(events))
        .route("/events/{event_index}", get(event_data))
        .with_state(state.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    axum::serve(listener, app).await?;

    *state.shutdown.lock().unwrap() = true;

    Ok(())
}

fn respond_ok_or_500<B: Into<String>>(body: anyhow::Result<B>) -> axum::http::Response<String> {
    match body {
        Ok(body) => axum::http::Response::builder()
            .status(200)
            .body(body.into())
            .unwrap(),
        Err(err) => axum::http::Response::builder()
            .status(500)
            .body(crate::error::error(err))
            .unwrap(),
    }
}

fn respond_json_or_500<T: Serialize>(result: anyhow::Result<T>) -> axum::response::Result<Json<T>> {
    match result {
        Ok(data) => Ok(Json(data)),
        Err(err) => Err(axum::response::Response::builder()
            .status(500)
            .body(crate::error::error(err))
            .unwrap()
            .into()),
    }
}

async fn root(
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> axum::http::Response<String> {
    respond_ok_or_500(crate::root::root(&state).await)
}

async fn view(
    axum::extract::Path(event_index): axum::extract::Path<usize>,
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> axum::http::Response<String> {
    respond_ok_or_500(crate::view::try_view(event_index, &state).await)
}

async fn assets(
    axum::extract::Path(file): axum::extract::Path<String>,
) -> axum::http::Response<String> {
    respond_ok_or_500(crate::assets::try_asset(&file))
}

async fn events(
    headers: axum::http::header::HeaderMap,
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> axum::response::Result<Json<Vec<crate::root::RootEvent>>> {
    respond_json_or_500(crate::events::events(&headers, &state).await)
}

async fn event_data(
    headers: axum::http::header::HeaderMap,
    axum::extract::Path(event_index): axum::extract::Path<usize>,
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> axum::response::Result<Json<serde_json::Value>> {
    respond_json_or_500(crate::events::try_event_data(&headers, event_index, &state).await)
}

#[derive(Deserialize, Debug)]
struct SourceQueryArgs {
    line: u32,
    column: u32,
}

async fn source(
    axum::extract::Path(path): axum::extract::Path<String>,
    axum::extract::Query(line_col): axum::extract::Query<SourceQueryArgs>,
) -> axum::http::Response<String> {
    respond_ok_or_500(crate::source::try_source(
        &path,
        line_col.line,
        line_col.column,
    ))
}

pub struct State {
    pub debug_events: Mutex<Vec<Arc<DebugEvent>>>,
    pub shutdown: Mutex<bool>,
}

fn record_events(debug_rx: Receiver<DebugEvent>, state: Arc<State>) {
    while !*state.shutdown.lock().unwrap() {
        if let Ok(event) = debug_rx.recv_timeout(Duration::from_secs(1)) {
            state.debug_events.lock().unwrap().push(Arc::new(event));
        }
    }
}
