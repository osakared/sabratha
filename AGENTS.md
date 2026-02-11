# AGENTS.md - Sabratha

Sabratha is the headless core of the Tripoli datalogging/machine control system.
It exposes a GraphQL API over HTTP (port 8000) backed by an embedded SQLite
database (via Turbosql). Written in Rust (edition 2024, MSRV 1.87.0).

Key crates: `async-graphql`, `poem`, `turbosql`, `tokio`, `rerun`, `serde`.

## Build / Test / Lint Commands

```bash
# Build
cargo build
cargo build --profile release

# Run all tests
cargo test

# Run a single test by name (substring match)
cargo test adding_rows_works
cargo test connection_fails
cargo test schema_check

# Run tests in a specific file
cargo test --test schema          # integration tests in tests/schema.rs
cargo test --lib                  # unit tests only (inline mod tests)

# Run tests in a specific module
cargo test store::tests::
cargo test connection::tests::

# Lint
cargo clippy
cargo clippy -- -D warnings       # treat warnings as errors (CI doesn't enforce this yet)

# Format
cargo fmt
cargo fmt -- --check              # check only, no changes

# Run the server
cargo run                         # starts on 0.0.0.0:8000, GraphiQL at /
```

CI runs `cargo build --profile release && cargo test --profile release` on every
push (see `.github/workflows/ci.yml`). There is no CI enforcement of clippy or
rustfmt, but you should run both before committing.

## Project Structure

```
src/
  main.rs          - Entry point: Poem HTTP server, GraphiQL endpoint
  lib.rs           - Library root, re-exports all modules
  schema.rs        - GraphQL schema: QueryRoot, MutationRoot, Root (app state)
  forms.rs         - Form and FormField models (Turbosql ORM + GraphQL types)
  connection.rs    - Connection model with status tracking, Arc<Mutex<Slab>> storage
  store.rs         - Data store abstraction: SimpleStore, Cell, time-series support
tests/
  schema.rs        - Integration test for the GraphQL schema
```

## Code Style Guidelines

This project follows [standard Rust style](https://rust-lang.github.io/api-guidelines/).
Use default `rustfmt` and `clippy` settings (no config files exist).

### Formatting

- 4-space indentation (no tabs)
- Standard Rust brace style (opening brace on same line)
- Keep lines under ~100 characters
- Run `cargo fmt` before committing

### Imports

- Group `use` statements in order: (1) `std`, (2) external crates, (3) local (`crate::`)
- Separate groups with a blank line
- Use `crate::` paths for local imports; use `super::` in test modules
- Glob imports (`use async_graphql::*`) are acceptable when many derive macros
  or attribute macros are needed from a single crate
- Prefer explicit imports (`use async_graphql::{Context, Object}`) when only a
  few items are needed
- Test modules scope their own imports inside `#[cfg(test)] mod tests { ... }`

### Naming Conventions

- Modules: `snake_case` (e.g., `connection`, `forms`, `store`)
- Structs/Enums: `PascalCase` (e.g., `Connection`, `FormFieldType`)
- Functions/methods: `snake_case` (e.g., `create_or_update`, `add_row`)
- Variables: `snake_case` (e.g., `form_id`, `col_name`)
- Type aliases: `PascalCase` (e.g., `type Storage = Arc<Mutex<Slab<Connection>>>`)
- Named constructors use patterns: `new()`, `of_text()`, `of_float()`, `of_int()`
- GraphQL root types: `QueryRoot`, `MutationRoot`

### Types and Generics

- Prefer type inference for local variables (`let x = Foo::new()`)
- Use explicit type annotations only in signatures and struct definitions
- Use type aliases for complex types (e.g., `pub type Storage = Arc<Mutex<Slab<Connection>>>`)

### Error Handling

- Functions that can fail return `Result<T, E>` with an appropriate error type
- Database layer uses `Result<T, turbosql::Error>`
- GraphQL resolvers return `Result<T, async_graphql::Error>`
- The `?` operator is preferred for propagation; avoid verbose `match` that just
  re-wraps the same `Result`
- `unwrap()` is acceptable in test code only
- `unwrap_or(vec![])` is used for database queries that may fail when an empty
  result is acceptable, but prefer propagating errors when possible
- The `ConvertsToGQLError` trait in `schema.rs` bridges `turbosql::Error` to
  `async_graphql::Error` via `.to_string()`
- `TripoliError` in `store.rs` is the project's custom error enum (currently minimal)

### GraphQL Patterns

- `QueryRoot` and `MutationRoot` are unit structs with `#[Object]` impl blocks
- Use `#[derive(SimpleObject)]` for types whose fields map directly to GraphQL fields
- Use `#[derive(InputObject)]` for GraphQL input types; combine with `SimpleObject`
  using `#[graphql(input_name = "...")]` to reuse a single struct for both
- Use `#[ComplexObject]` on a separate impl block to add computed/async resolvers
  to a `SimpleObject`
- Use `#[graphql(skip)]` to hide internal fields (e.g., `rowid`) from the schema
- Access shared state via `ctx.data_unchecked::<Storage>()` (prefer `ctx.data()`
  with error handling in new code)
- `EmptySubscription` is used (no subscriptions implemented)

### Derive Macros

Common derive patterns in this codebase:
- Database models: `#[derive(Turbosql, SimpleObject, InputObject, Clone)]`
- GraphQL enums: `#[derive(Enum, Copy, Clone, Eq, PartialEq)]`
- Data/store types: `#[derive(Serialize, Deserialize, Debug)]`
- Use `serde::Serialize, serde::Deserialize` (qualified) when `async_graphql::*`
  is glob-imported to avoid ambiguity

### Async Patterns

- Tokio is the async runtime (`#[tokio::main]`, `#[tokio::test]`)
- Shared mutable state uses `Arc<futures_util::lock::Mutex<...>>` (not
  `std::sync::Mutex` or `tokio::sync::Mutex`)
- All GraphQL resolver methods are `async fn` even if the underlying operation
  is synchronous (required by async-graphql)

### Testing

- Unit tests: inline `#[cfg(test)] mod tests` within source files
- Integration tests: `tests/*.rs`
- Test function names are descriptive snake_case phrases (e.g., `adding_rows_works`)
- No `test_` prefix needed (the `#[test]` attribute is sufficient)
- Use `assert_eq!` with `async_graphql::value!` macro for asserting GraphQL responses
- Tests construct the system manually (no shared test fixtures)
- New functionality should have unit tests (per CONTRIBUTING.md)

### Documentation

- Use `///` doc comments on public functions describing what they do
- Use backtick-quoted type names in doc comments (e.g., `` `Form` ``)
- Use `//` inline comments for design rationale and non-obvious decisions

### Contributing

- The canonical repository is on [Codeberg](https://codeberg.org/osakared/sabratha),
  not GitHub (GitHub is a mirror)
- Ensure an issue exists for the bug/feature before creating a pull request
- Pull requests go against the `main` branch
- New functionality must have unit tests
- Code must match the style of the file it's in
