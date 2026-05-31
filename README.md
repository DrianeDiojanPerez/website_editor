# website_editor

Small Rust API using **Axum**, **SQLx (SQLite)**, **tracing**, and the **repository pattern**. Layered after the Go `smart_net_api` project: `src/api/` for HTTP delivery, `src/packages/` for reusable domain layers.

## Stack

- [axum](https://crates.io/crates/axum) — HTTP router
- [sqlx](https://crates.io/crates/sqlx) — async SQLite with compile-time migrations
- [tracing](https://crates.io/crates/tracing) + [tracing-appender](https://crates.io/crates/tracing-appender) — structured logs, daily JSON files
- [dotenvy](https://crates.io/crates/dotenvy) — `.env` loading

## Layout

```
src/
  main.rs                    bootstrap (env, logger, db, handler, router, serve)
  api/                       HTTP delivery (Go's cmd/api/)
    handler/    mod.rs + per-resource files (item.rs ...)
    middlewares/
    routes/
  packages/                  reusable domain layers (Go's packages/)
    codes/                   error code constants
    dto/                     request/response shapes + Response<T>
    lib/                     env helpers + tracing setup
    model/                   DB entities (sqlx::FromRow)
    repository/              Store trait + SqliteStore + per-resource repos
    service/                 ServiceError + per-resource business logic
    store/                   DB pool factory
  utils/
database/migrations/         sqlx migrations
```

Flow per request: **handler → service → repository → SQLx → SQLite**. DTOs cross the handler/service boundary; models stay below the service line.

## Setup

```bash
cp .env.example .env
cargo run
```

The migration in `database/migrations/` runs automatically on startup; SQLite file is created at `data.db` (per `DATABASE_URL`).

## Env vars

| var | default | meaning |
|---|---|---|
| `DATABASE_URL` | *required* | e.g. `sqlite://data.db?mode=rwc` |
| `SERVER_ADDR`  | `127.0.0.1:3000` | bind address |
| `RUST_LOG`     | `info`          | per-target log filter |
| `LOG_DIR`      | `logs`          | daily JSON log directory |

## Endpoints

```
GET    /health
GET    /api/v1/items
POST   /api/v1/items          { "name": "...", "description": "..." }
GET    /api/v1/items/:id
PATCH  /api/v1/items/:id
DELETE /api/v1/items/:id
```

## API docs

Interactive API reference is served by the running app:

- **`/`** — [Scalar](https://github.com/scalar/scalar) UI (try requests in the browser)
- **`/openapi.yaml`** — the raw OpenAPI spec

Both are embedded into the binary at compile time from `docs/`, so they ship with the executable.

Prefer a desktop API client? Open the `bruno/` folder as a collection in [Bruno](https://www.usebruno.com/) (**Open Collection** → pick `bruno/`). Select the **Local** environment, then run `Auth > Register` (or `Login`) once — its script stores the access/refresh tokens automatically, and every protected request inherits the bearer token.

## Logs

- **stdout** — human-readable
- **file** — `logs/website_editor.log.YYYY-MM-DD`, one JSON event per line, rolled at UTC midnight

## Adding a new resource

For a new resource `foo`, drop five files and wire them in:

1. `src/packages/model/foo.rs` — DB entity
2. `src/packages/dto/foo.rs` — request/response DTOs
3. `src/packages/repository/foo.rs` — `FooRepository` trait + SQLite impl
4. `src/packages/service/foo.rs` — `FooService` trait + impl
5. `src/api/handler/foo.rs` — Axum handlers

Then:

- add `pub mod foo;` to each parent `mod.rs`
- add a `fn foo_store(&self)` accessor to `Store` in `src/packages/repository/mod.rs`
- add a `pub foo_service` field to `Handler` and wire it in `configure_handlers` (`src/api/handler/mod.rs`)
- register routes in `src/api/routes/mod.rs`
- add a migration in `database/migrations/`

## Other Rust testing tools worth knowing (didn't add, but here's the menu)

| Crate | Use case |
|---|---|
| `pretty_assertions` | Pretty-printed diffs for `assert_eq!` on complex values |
| `insta` | Snapshot testing (compare output to a checked-in golden file) |
| `proptest` / `quickcheck` | Property-based testing — generate hundreds of inputs |
| `serial_test` | Force certain tests to run sequentially (for env-mutating tests) |
| `wiremock` | Mock outgoing HTTP for tests that call external APIs |
| `cargo-nextest` | A faster test runner (parallel by default, better output) — install with `cargo install cargo-nextest`, run with `cargo nextest run` |
