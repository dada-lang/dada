use std::{sync::{mpsc::Receiver, Arc, Mutex}, time::Duration};

use axum::{routing::get, Router};
use dada_ir_ast::DebugEvent;

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

    let state = Arc::new(State { debug_events: Default::default(), shutdown: Default::default() });

    std::thread::spawn({
        let state = state.clone();
        move || record_events(debug_rx, state)
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/view/{event_index}", get(view))
        .with_state(state.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    
    axum::serve(listener, app).await?;

    *state.shutdown.lock().unwrap() = true;

    Ok(())
}

async fn root(
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> String {
    crate::error::maybe_error(crate::root::root(&state).await)
}

pub struct State {
    pub debug_events: Mutex<Vec<Arc<DebugEvent>>>,
    pub shutdown: Mutex<bool>,
}

// basic handler that responds with a static string
async fn view(
    axum::extract::Path(event_index): axum::extract::Path<usize>,
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> String {
    crate::error::maybe_error(crate::view::try_view(event_index, &*state).await)
}

fn record_events(debug_rx: Receiver<DebugEvent>, state: Arc<State>) {
    while !*state.shutdown.lock().unwrap() {
        if let Ok(event) = debug_rx.recv_timeout(Duration::from_secs(1)) {
            state.debug_events.lock().unwrap().push(Arc::new(event));
        }
    }
}