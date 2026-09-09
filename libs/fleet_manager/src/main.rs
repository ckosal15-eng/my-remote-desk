use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::{Basic, Bearer}},
};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

mod db;

struct AppState {
    conn: Mutex<rusqlite::Connection>,
    secret_token: String,
}

#[tokio::main]
async fn main() {
    let conn = db::init_db().expect("Failed to initialize database");
    let secret_token = std::env::var("FLEET_SECRET_TOKEN").unwrap_or_else(|_| "secret".to_string());

    let shared_state = Arc::new(AppState {
        conn: Mutex::new(conn),
        secret_token,
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/devices", get(get_devices))
        .route("/api/heartbeat", post(heartbeat))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Fleet Manager running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler(
    auth_header: Option<TypedHeader<Authorization<Basic>>>,
) -> Result<Html<&'static str>, (StatusCode, &'static str)> {
    if let Some(TypedHeader(auth)) = auth_header {
        if auth.username() == "admin" && auth.password() == "password" {
            return Ok(Html(include_str!("index.html")));
        }
    }
    
    // Request basic auth
    Err((StatusCode::UNAUTHORIZED, "Unauthorized. Use admin/password"))
}

async fn get_devices(
    auth_header: Option<TypedHeader<Authorization<Basic>>>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<db::Device>>, (StatusCode, &'static str)> {
    if let Some(TypedHeader(auth)) = auth_header {
        if auth.username() == "admin" && auth.password() == "password" {
            let conn = state.conn.lock().unwrap();
            let devices = db::get_all_devices(&conn).unwrap_or_default();
            return Ok(Json(devices));
        }
    }
    Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
}

#[derive(serde::Deserialize)]
struct HeartbeatReq {
    id: String,
    hostname: String,
    os: String,
}

async fn heartbeat(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HeartbeatReq>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    if let Some(TypedHeader(auth)) = auth_header {
        if auth.token() == state.secret_token {
            let conn = state.conn.lock().unwrap();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let device = db::Device {
                id: payload.id,
                hostname: payload.hostname,
                os: payload.os,
                last_seen: now,
            };
            db::upsert_device(&conn, &device).unwrap();
            return Ok(StatusCode::OK);
        }
    }
    Err((StatusCode::UNAUTHORIZED, "Invalid secret token"))
}
