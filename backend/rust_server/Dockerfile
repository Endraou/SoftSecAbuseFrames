# Step 1: Build the application
FROM rust:1.88 AS builder
WORKDIR /app

# Copy manifest and lock file first to cache dependencies
COPY Cargo.toml Cargo.lock ./ 

COPY . .

# Force SQLx to use the sqlx-data.json file instead of trying to connect to a DB
ENV SQLX_OFFLINE=true

RUN cargo build --release
# We use a release build for better performance and security
RUN cargo build --release

# Step 2: Run the application
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/secure_notes_backend .
# Ensure the backend can talk to the DB
CMD ["./secure_notes_backend"]