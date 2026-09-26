mod auth;
mod db;
mod game;
mod ws;

use astrobrawl_shared::TICK_INTERVAL_MS;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use game::GameWorld;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use ws::{AppState, SharedState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("⚡ Démarrage du serveur AstroBrawl...");

    // Database configuration
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "astrobrawl.db".to_string());
    let pool = db::init_db(&db_path)?;
    info!("Base de données SQLite connectée avec succès: {}", db_path);

    // Shared game world & authoritative state
    let world = Arc::new(RwLock::new(GameWorld::new(pool.clone())));

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "astrobrawl-super-secret-key-change-in-prod".to_string());

    let state: SharedState = Arc::new(AppState {
        world: world.clone(),
        db_pool: pool.clone(),
        jwt_secret,
    });

    // Authoritative 30 Hz Game Loop
    let loop_world = world.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(TICK_INTERVAL_MS));
        info!("Boucle de jeu autoritaire active à 30 Hz (dt = 33ms)");
        loop {
            interval.tick().await;
            let mut w = loop_world.write().await;
            w.tick_simulation();
        }
    });

    // CORS configuration for web clients
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let dist_dir = std::path::Path::new("client/dist");
    let app = if dist_dir.exists() {
        info!("📂 Distribution Web WASM trouvée dans client/dist, servie sur http://0.0.0.0:3000");
        Router::new()
            .route("/health", get(health_check))
            .route("/api/auth/github", get(github_login_redirect))
            .route("/api/auth/github/callback", get(github_callback))
            .route("/api/auth/guest", post(guest_login))
            .route("/ws", get(ws::ws_handler))
            .fallback_service(tower_http::services::ServeDir::new("client/dist"))
            .layer(cors)
            .with_state(state)
    } else {
        Router::new()
            .route("/health", get(health_check))
            .route("/api/auth/github", get(github_login_redirect))
            .route("/api/auth/github/callback", get(github_callback))
            .route("/api/auth/guest", post(guest_login))
            .route("/ws", get(ws::ws_handler))
            .layer(cors)
            .with_state(state)
    };

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 AstroBrawl Server prêt sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok", "game": "AstroBrawl" })))
}

async fn github_login_redirect() -> impl IntoResponse {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    if client_id.is_empty() {
        return Redirect::temporary("/?error=github_client_id_not_configured");
    }

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&scope=read:user",
        client_id
    );
    Redirect::temporary(&auth_url)
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
}

async fn github_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let code = match params.code {
        Some(c) => c,
        None => return Redirect::temporary("/?error=missing_code"),
    };

    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "/".to_string());

    match auth::exchange_github_code(&client_id, &client_secret, &code).await {
        Ok(github_user) => {
            let github_id_str = github_user.id.to_string();
            let db_player = match db::get_or_create_github_player(
                &state.db_pool,
                &github_id_str,
                &github_user.login,
            ) {
                Ok(p) => p,
                Err(e) => {
                    error!("Erreur base de données lors de la création GitHub: {:?}", e);
                    return Redirect::temporary("/?error=database_error");
                }
            };

            let token = match auth::create_jwt(db_player.id, &db_player.username, &state.jwt_secret) {
                Ok(t) => t,
                Err(e) => {
                    error!("Erreur génération JWT: {:?}", e);
                    return Redirect::temporary("/?error=jwt_error");
                }
            };

            info!("Authentification GitHub réussie pour {}", db_player.username);
            let redirect_target = if frontend_url.contains('?') {
                format!("{}&token={}", frontend_url, token)
            } else {
                format!("{}?token={}", frontend_url, token)
            };

            Redirect::temporary(&redirect_target)
        }
        Err(e) => {
            error!("Erreur OAuth GitHub: {:?}", e);
            Redirect::temporary("/?error=oauth_failed")
        }
    }
}

async fn guest_login(State(state): State<SharedState>) -> impl IntoResponse {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let guest_id = format!("{:08x}", rng.gen::<u32>());
    let guest_username = format!("Pilote-{}", &guest_id[..4]);

    match db::get_or_create_guest_player(&state.db_pool, &guest_id, &guest_username) {
        Ok(player) => match auth::create_jwt(player.id, &player.username, &state.jwt_secret) {
            Ok(token) => (
                StatusCode::OK,
                Json(json!({
                    "token": token,
                    "username": player.username,
                    "player_id": player.id,
                    "credits": player.credits,
                    "minerals": player.minerals
                })),
            ),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "jwt_error" })),
            ),
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "db_error" })),
        ),
    }
}
