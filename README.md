# SoftSecAbuseFrames

## Project Overview
This software provides a resilient, multi-user environment for storing and sharing textual notes. It is built with a focus on security countermeasures to prevent data leaks and unauthorized access.

## Tech Stack
- **Backend:** Rust (Axum + SQLx)
- **Database:** PostgreSQL (with Streaming Replication)
- **Security:** Argon2id (Hashing), JWT (Authentication), Parameterized Queries (SQLi prevention)

## Security Countermeasures
1. **Password Safety:** We use the **Argon2id** algorithm to hash passwords. This is memory-hard and resistant to GPU brute-force attacks.
2. **Authentication:** All note-related routes are protected by a **JWT Middleware**. Access is denied unless a valid `Bearer` token is provided.
3. **SQL Injection Prevention:** We use **SQLx** with the `query!` macro. This ensures all user input is parameterized and never executed as raw SQL code.
4. **Multi-User Isolation:** Every database query explicitly filters by the `user_id` extracted from the JWT. A user cannot access or guess the ID of another user's notes.
5. **Note Locking:** Write access is managed via a `locked_by` mechanism in the database. This prevents simultaneous edits and race conditions.

## Resilience & Replication
To ensure data durability, the system supports:
- **Database Replication:** Two PostgreSQL nodes configured in Primary/Replica mode.
- **Connection Pooling:** The backend uses `PgPoolOptions` to manage resilient database connections.

## Security Testing
- **Unauthorized Access Test:** Attempting to `GET /notes/{id}` with a token from User B on a note owned by User A returns a `401 Unauthorized`.
- **SQLi Test:** Passing `' OR 1=1 --` as a search parameter is treated as a literal string by SQLx and fails to return unauthorized data.
- **Locking Test:** If `locked_by` is set for a different UUID, `PUT` requests return a `409 Conflict`.


## How to Run

To ensure you can test the full functionality without manually installing PostgreSQL, we have containerized the entire stack.

### Setup & Launch
1. Open a terminal in the project root.
2. Run the following command:
   ```bash
   docker-compose up --build

   # 📝 Secure Personal Notes Manager

A resilient, multi-user note-taking system built with **Rust (Axum)** and **SQL (PostgreSQL)**.

## For Windows Users
1. Install **Docker Desktop** for Windows.
2. Open PowerShell or Command Prompt in the project folder.
3. Run the following command:
   ```powershell
   docker-compose up --build