use axum::{routing::{get, post}, Router, middleware};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer; // Add to Cargo.toml: tower-http = { version = "0.5", features = ["cors"] }

mod handlers;
mod auth;
mod models;
mod error;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connection Pool (Resilience: you'd point this to your replicated DBs)
    let mut retry_count = 0;
    let pool = loop {
        match PgPoolOptions::new()
            .max_connections(5)
            // Ensure this points to "db_primary" in your docker-compose
            .connect(&db_url) 
            .await 
        {
            Ok(pool) => break pool,
            Err(e) => {
                if retry_count >= 10 {
                    panic!("Failed to connect to Postgres after 10 retries: {}", e);
                }
                retry_count += 1;
                println!("Database not ready, retrying in 2s... (Attempt {})", retry_count);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

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
        .layer(middleware::from_fn(auth::authorize)); // Apply auth ONLY here

    // 2. Define routes that are PUBLIC
    let auth_routes = Router::new()
        .route("/register", post(handlers::auth_handlers::register))
        .route("/login", post(handlers::auth_handlers::login));

    // 3. Combine them into the main app
    let app = Router::new()
        .nest("/auth", auth_routes)  // Becomes /auth/register and /auth/login
        .merge(protected_routes)
        .with_state(pool)
        .layer(cors); // CORS remains at the very bottom to cover everything

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Secure server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}