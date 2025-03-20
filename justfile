# CatColab minimal justfile

# Show available commands
default:
    @just --list

# Simple setup
setup:
    @echo "Creating flox.toml..."
    @echo '[env]' > flox.toml
    @echo 'packages = [' >> flox.toml
    @echo '  "pnpm",' >> flox.toml
    @echo '  "node",' >> flox.toml
    @echo '  "wasm-pack"' >> flox.toml
    @echo ']' >> flox.toml
    @echo "Installing frontend dependencies..."
    @cd packages/frontend && pnpm install
    @echo "✅ Setup complete! Try 'just run-staging' to run without a database"

# Run in staging mode (no database required)
run-staging:
    @echo "Running CatColab with staging backend (no local DB)..."
    @cd packages/frontend && pnpm run dev --mode staging

# Run backend server only
run-backend:
    @cd packages/backend && cargo run

# Run frontend server only
run-frontend:
    @cd packages/frontend && pnpm run dev

# Run both frontend and backend (requires DB setup)
run:
    @echo "Starting CatColab application..."
    @echo "→ Backend: http://localhost:8000"
    @echo "→ Frontend: http://localhost:3000"
    @cd packages/backend && cargo run & BACKEND_PID=$!
    @cd packages/frontend && pnpm run dev || kill $BACKEND_PID

# Setup local database (optional)
db-setup:
    @echo "DATABASE_URL=postgres://postgres:postgres@localhost:5432/catcolab" > .env
    @createdb -h localhost -p 5432 -U postgres catcolab 2>/dev/null || echo "Database exists"
    @cd packages/backend && cargo sqlx migrate run || echo "Run 'cargo install sqlx-cli' if this fails"
    @echo "✅ Database ready! You can now run 'just run'"
