use axum::{
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::Mutex as AsyncMutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::process;
use std::panic;


#[derive(Deserialize)]
struct MathQuery {
    a: u64,
    b: u64,
    operation: String,
}

#[derive(Serialize)]
struct MathResult {
    result: u64,
}

type Storage = Arc<AsyncMutex<HashMap<String, serde_json::Value>>>;

async fn calculate(Json(payload): Json<MathQuery>) -> Json<MathResult> {
    let result = match payload.operation.as_str() {
        "addition" => payload.a + payload.b,
        "subtraction" => payload.a - payload.b,
        "multiplication" => payload.a * payload.b,
        "division" => payload.a / payload.b,
        _ => u64::MAX,
    };

    Json(MathResult { result })
}

async fn store_data(
    Json(data): Json<serde_json::Value>,
    storage: Arc<AsyncMutex<HashMap<String, serde_json::Value>>>,
) -> String {
    let mut store = storage.lock().await;
    let key = format!("entry_{}", store.len() + 1);
    store.insert(key.clone(), data);
    format!("Data stored with key: {}", key)
}

async fn retrieve_all(storage: Arc<AsyncMutex<HashMap<String, serde_json::Value>>>) -> Json<HashMap<String, serde_json::Value>> {
    let store = storage.lock().await;
    Json(store.clone())
}

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|_| {
        eprintln!("A thread panicked! Aborting the process...");
        process::abort();
    }));
    let storage: Storage = Arc::new(AsyncMutex::new(HashMap::new()));

    let app = Router::new()
        .route("/math", post(calculate))
        .route("/store", post({
            let storage = storage.clone();
            move |json| store_data(json, storage.clone())
        }))
        .route("/store/all", axum::routing::get({
            let storage = storage.clone();
            move || retrieve_all(storage.clone())
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
