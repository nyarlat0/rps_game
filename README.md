# RPS Game

A small full-stack Rock-Paper-Scissors web game built with Rust.

The project is not just a button-click toy: it includes user registration, cookie-based authentication, PostgreSQL persistence, a simple forum, online user stats, and real-time matchmaking/game updates over WebSocket.

![Demo](assets/demo.gif)

## Live demo

You can try the project here: [nyarlat.org](https://nyarlat.org)

## Features

- Rock-Paper-Scissors matchmaking between online users
- Real-time game state updates through WebSocket
- User registration and login
- JWT stored in an HTTP-only cookie
- PostgreSQL-backed users, forum posts, reactions, and game history
- Shared Rust types between backend and frontend
- Leptos single-page frontend
- Actix Web backend
- Simple Caddy reverse-proxy setup for local serving

## Tech stack

### Backend

- Rust
- Actix Web
- Actix actors
- actix-ws
- SQLx
- PostgreSQL
- Argon2 password hashing
- JWT authentication

### Frontend

- Rust
- Leptos
- Trunk
- WebAssembly
- Fluent localization
- CSS

### Shared

The `shared` crate contains common request/response types used by both the backend and the frontend.

## Project structure

```text
.
├── backend/       # Actix Web backend
├── frontend/      # Leptos frontend compiled to WASM
├── shared/        # Shared types for backend and frontend
├── Caddyfile      # Local reverse proxy config
├── Cargo.toml     # Rust workspace
└── Makefile.toml  # cargo-make tasks
```

## Requirements

Install:

- Rust
- PostgreSQL
- Trunk
- cargo-make
- cargo-watch
- Caddy, optional but useful for the provided dev setup

Example:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk cargo-make cargo-watch
```

## Configuration

The backend reads environment variables from:

```text
/etc/rps_game/.env
```

and also tries to load a local `.env` file.

Create one of them with:

```env
DATABASE_URL=postgres://user:password@localhost/rps_game
JWT_SECRET=base64-encoded-secret-here
```

`JWT_SECRET` must be base64-encoded because the backend uses it as an HS256 key.

You can generate one with:

```bash
openssl rand -base64 64
```

## Running locally

Clone the repository:

```bash
git clone https://github.com/nyarlat0/rps_game.git
cd rps_game
```

Start the full dev stack:

```bash
cargo make dev
```

This runs:

- Caddy on `:3000`
- backend on `127.0.0.1:8081`
- frontend through Trunk

Then open:

```text
http://localhost:3000
```

## Running parts manually

Backend:

```bash
cargo run -p backend
```

Frontend:

```bash
cd frontend
trunk serve
```

Production frontend build:

```bash
cd frontend
trunk build --release
```

Backend release build:

```bash
cargo build --release -p backend
```

## API overview

The backend exposes its API under `/api`.

### Auth

```text
POST /api/auth/register
POST /api/auth/login
POST /api/auth/logout
GET  /api/auth/me
```

Authentication is cookie-based. After login, the server sets an `auth_token` cookie.

### Forum

```text
POST /api/forum
```

The forum endpoint accepts JSON commands such as:

- create post
- fetch posts
- like post
- dislike post
- undo reaction
- delete post

### WebSocket

```text
GET /api/ws
```

The WebSocket connection is used for:

- online user stats
- matchmaking
- submitting Rock-Paper-Scissors moves
- receiving game state updates
- receiving new forum post notifications

## Game flow

1. User registers or logs in.
2. User opens a WebSocket connection.
3. User joins the Rock-Paper-Scissors queue.
4. When another player joins, the backend creates a game.
5. Both players submit their moves.
6. The backend resolves the result and sends the final state to both players.

## Development tasks

This project uses `cargo-make`.

```bash
cargo make dev
```

Run Caddy, backend, and frontend in parallel.

```bash
cargo make dev-backend
```

Run backend with `cargo watch`.

```bash
cargo make dev-frontend
```

Run frontend with `trunk watch`.

```bash
cargo make build-backend
```

Build backend in release mode.

```bash
cargo make build-frontend
```

Build frontend in release mode.

```bash
cargo make deploy
```

Build frontend, build backend, and install the backend binary.

## Notes

This is a learning/full-stack playground project, so the goal is not to reinvent Rock-Paper-Scissors as a billion-dollar goblin machine. The interesting parts are the architecture around it:

- shared Rust types across client and server
- WebSocket game updates
- cookie auth
- database-backed users and forum data
- separation between domain, application, and infrastructure layers

## TODO

- Add database migrations or schema setup instructions
- Add tests
- Add CI
- Add a production deployment guide
- Add screenshots or more demo GIFs

## License

This project is released under The Unlicense.
