# SoftSecAbuseFrames

## Project Overview
This software provides a resilient, multi-user environment for storing and sharing textual notes. It is built with a focus on defense-in-depth security countermeasures to prevent data leaks, unauthorized access, and service abuse.

## Tech Stack
- **Backend:** Rust (Axum + SQLx)
- **Frontend:** HTML5, CSS3, Vanilla JavaScript
- **Database:** PostgreSQL with Primary/Replica Streaming Replication
- **Security:** Argon2id (Hashing), JWT (Auth), Ammonia (XSS Sanitization), Tower-Governor (Rate Limiting), Parametize Query (SQL Injection)

## Security Countermeasures
1. **Advanced Password Hashing:** The system uses the Argon2id algorithm, which is memory-hard and resistant to GPU brute-force attacks.
Salts are generated using cryptographically secure random number generators (OsRng).
Password verification is handled securely on the backend before any session is initiated.

2. **Multi-Layer Authentication & Session Management:** JWT Middleware: All note-related routes are protected by a JSON Web Token middleware.
Short-Lived Sessions: Tokens expire after 20 minutes to limit the window of opportunity for stolen credentials.
Client-Side Cooldown: The frontend implements a mandatory cooldown timer after failed attempts to prevent local brute-force scripts.

3. **Protection Against Injection & Web Attacks:** SQL Injection (SQLi): The project uses SQLx with the query! macro, ensuring all user input is parameterized and never executed as raw SQL code.
Cross-Site Scripting (XSS):
Backend: All note content is sanitized using the Ammonia library before being stored in the database.
Frontend: The UI utilizes .textContent to ensure the browser treats content as literal text rather than executable HTML.
Content Security Policy (CSP): A strict CSP header is enforced globally: default-src 'self'; script-src 'self'; style-src 'self';.

4. **Concurrency & Data Isolation:** Multi-User Isolation: Every database query explicitly filters by the user_id extracted from the JWT, preventing users from accessing or guessing other users' data.
Note Locking: To prevent race conditions during edits, write access is managed via a locked_by mechanism.
Lock Expiration: Locks automatically expire after 15 minutes to prevent permanent deadlocks if a user disconnects abruptly.

5. **API Resilience & Rate Limiting:** Rate Limiting: Implemented via tower-governor, allowing 2 requests per second with a burst capacity of 5.
CORS Policy: Strictly restricted to authorized origins (e.g., http://localhost:8080).
Payload Limits: A global DefaultBodyLimit is set to 1MB to prevent memory exhaustion attacks.

## Resilience & Replication
To ensure high availability and data durability:
- **Database Replication:** Two PostgreSQL nodes are configured in Primary (Writer) and Replica (Reader) mode.
- **Connection Pooling:** The backend uses PgPoolOptions to manage resilient database connections independently for read and write operations.

## Security Testing
- **Unauthorized Access Test:** Attempting to GET a note with a token from a different user returns a 401 Unauthorized.
- **Locking Test:** If a note is locked_by another user, PUT requests return a 409 Conflict.
- **XSS Test:** Passing <script>alert(1)</script> into a note will result in a sanitized string being stored and displayed.


## How to Run

To ensure you can test the full functionality without manually installing PostgreSQL, we have containerized the entire stack.

### Setup & Launch
0. Ensure you have Docker install on your machine

1. Open a terminal in the project root.

2. Go to backend/rust_server (where there is the docker-compose file)

3. Run the following command:
   ```bash
   docker-compose up --build

4. wait for this line to appear on your terminal :
   ```bash
   backend-1     | Secure server listening on 0.0.0.0:3000
   This ensure the server and application are both running

5. Go to localhost:8080

## For Windows Users
Install **Docker Desktop** for Windows and ensure it is running before doing 3.