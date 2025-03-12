use std::{path::{Path, PathBuf}, sync::Arc};

use axum::{routing::get, Router};

pub fn main(port: u32, path: &Path) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(main_async(port, path))?;
    Ok(())
}

async fn main_async(port: u32, path: &Path) -> anyhow::Result<()> {
    let _events = crate::watch::EventStream::new(path)?;
    let path = path.to_path_buf();

    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/view/{*path}", get(view))
        .with_state(Arc::new(State { path: path.to_path_buf() }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    
    axum::serve(listener, app).await?;

    Ok(())
}

pub struct State {
    pub path: PathBuf,
}

// basic handler that responds with a static string
async fn root(
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> &'static str {
    "Hello, World!"
}

// basic handler that responds with a static string
async fn view(
    axum::extract::Path(path): axum::extract::Path<PathBuf>,
    axum::extract::State(state): axum::extract::State<Arc<State>>,
) -> String {
    crate::view::try_view(&path, &*state).await.unwrap()
}
