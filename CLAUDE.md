# CatColab Development Guide

## Build & Run Commands
- **Setup Environment**: `just setup`
- **Setup Database**: `just db-setup`
- **Run App**: `just run` (backend on port 8000, frontend on port 3000)
- **Run Backend Only**: `just run-backend`
- **Run Frontend Only**: `just run-frontend`
- **Run Without Database**: `just run-staging`

## Code Style Guidelines
- **Frontend**: Use SolidJS patterns with TypeScript
- **Backend**: Rust with axum/tokio frameworks
- **Formatting**: Biome with 4-space indentation, 100 char line width
- **Database**: PostgreSQL with sqlx migrations
- **WebAssembly**: Located in packages/catlog-wasm directory

## Project Structure
- **Monorepo**: Uses workspaces with packages/ subdirectories
- **Environment**: Uses Flox for dependency management
- **Package Manager**: pnpm with npm fallback for frontend

## Testing
- **Frontend Tests**: `cd packages/frontend && pnpm test`
- **Backend Tests**: `cd packages/backend && cargo test`
