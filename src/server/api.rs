use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::query::QueryExecutor;

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
    user_id: String,
}

#[derive(Serialize)]
struct QueryResponse {
    result: String,
}

struct AppState {
    query_executor: Arc<QueryExecutor>,
}

async fn execute_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    let result = state.query_executor.execute(&payload.query, &payload.user_id)
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));

    Json(QueryResponse { result })
}

pub async fn run_server(port: u16, query_executor: QueryExecutor) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState {
        query_executor: Arc::new(query_executor),
    });

    let app = Router::new()
        .route("/query", post(execute_query))
        .with_state(app_state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("AI-First DB Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}