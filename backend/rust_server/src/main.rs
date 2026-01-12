use std::sync::Arc;
use axum::{
    routing::{get, post}, 
    Router, 
    middleware, 
    http::header::{CONTENT_SECURITY_POLICY, HeaderValue},
    extract::DefaultBodyLimit
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::cors::CorsLayer; // Add to Cargo.toml: tower-http = { version = "0.5", features = ["cors"] }
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

mod handlers;
mod auth;
mod models;
mod error;

pub struct AppState {
    pub writer_pool: sqlx::PgPool,
    pub reader_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let primary_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let replica_url = std::env::var("REPLICA_URL").expect("REPLICA_URL must be set");

    let writer_pool = PgPoolOptions::new().max_connections(5).connect(&primary_url).await.unwrap();
    let reader_pool = PgPoolOptions::new().max_connections(10).connect(&replica_url).await.unwrap();

    let state = std::sync::Arc::new(AppState {
        writer_pool,
        reader_pool,
    });

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap()
    );

    let csp_layer = SetResponseHeaderLayer::if_not_present(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self';"),
    );

    // Inside your main() function:
    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
        .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        .allow_origin("http://localhost:8080".parse::<axum::http::HeaderValue>().unwrap());

    // Inside your main() function:

    // 1. Define routes that REQUIRE authentication
    let protected_routes = Router::new()
        .route("/notes", get(handlers::handlers::list_notes).post(handlers::handlers::create_note))
        .route("/notes/:id", get(handlers::handlers::get_note).put(handlers::handlers::update_note).delete(handlers::handlers::delete_note))
        .route("/notes/:id/lock", post(handlers::handlers::lock_note))
        .route("/notes/:id/share", post(handlers::handlers::share_note))
        .route("/notes/:id/unlock", post(handlers::handlers::unlock_note))
        .layer(middleware::from_fn(auth::authorize))    // Apply auth ONLY here
        .with_state(Arc::clone(&state));

    // 2. Define routes that are PUBLIC
    let auth_routes = Router::new()
        .route("/register", post(handlers::auth_handlers::register))
        .route("/login", post(handlers::auth_handlers::login))
        .with_state(Arc::clone(&state));

    // 3. Combine them into the main app
    let app = Router::new()
        .nest("/auth", auth_routes)  // Becomes /auth/register and /auth/login
        .merge(protected_routes)
        .layer(GovernorLayer { config: governor_conf }) // Protection globale
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(cors) // CORS remains at the very bottom to cover everything
        .layer(csp_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Secure server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}