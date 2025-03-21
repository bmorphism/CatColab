# CatColab minimal justfile - Optimized for Flox integration

# Show available commands
default:
    @just --list

# Check if the Flox environment is active
check-flox:
    @echo "Checking Flox environment..."
    @if [ -z "${FLOX_ENV:-}" ]; then \
        echo "[WARNING] Not in Flox environment!" && \
        echo "Options to setup Flox environment:" && \
        echo "  1. flox pull bmorphism/CatColab      # Get a local copy connected to FloxHub" && \
        echo "  2. flox pull --copy bmorphism/CatColab  # Get disconnected local copy" && \
        echo "  3. cd .. && flox init CatColab && cd CatColab && flox install pnpm node wasm-pack  # Create new env" && \
        echo "[NOTE] Continuing with system-installed tools. You'll need npm/pnpm, node and wasm-pack installed."; \
    else \
        echo "[OK] Using Flox environment: ${FLOX_ENV}"; \
    fi

# Prepare the project
setup: check-flox
    @echo "Installing frontend dependencies..."
    @cd packages/frontend && pnpm install || npm install
    @echo "[OK] Setup complete! Try 'just run-staging' to run without a database"

# Run in staging mode (no database required)
run-staging: check-flox
    @echo "Running CatColab with staging backend (no local DB)..."
    @cd packages/frontend && pnpm run dev --mode staging || npm run dev -- --mode staging

# Run backend server only
run-backend: check-flox
    @cd packages/backend && cargo run

# Run frontend server only
run-frontend: check-flox
    @cd packages/frontend && pnpm run dev || npm run dev

# Run both frontend and backend (requires DB setup)
run: check-flox
    @echo "Starting CatColab application..."
    @echo "-> Backend: http://localhost:8000"
    @echo "-> Frontend: http://localhost:3000"
    @cd packages/backend && cargo run & BACKEND_PID=$!
    @cd packages/frontend && pnpm run dev || npm run dev || kill $BACKEND_PID

# Setup local database (optional)
db-setup: check-flox
    @echo "Setting up database for macOS..."
    @echo "DATABASE_URL=postgres:///catcolab" > .env
    @createdb catcolab 2>/dev/null || echo "Database exists"
    @cd packages/backend && cargo sqlx migrate run || echo "Run 'cargo install sqlx-cli' if this fails"
    @echo "[OK] Database ready! You can now run 'just run'"

# Full deploy from scratch with local components
deploy-local: check-deps install-wasm build-wasm setup db-setup run

# Check for required dependencies
check-deps:
    @echo "Checking dependencies..."
    @command -v node >/dev/null 2>&1 || { echo "[ERROR] Node.js not found"; exit 1; }
    @command -v cargo >/dev/null 2>&1 || { echo "[ERROR] Rust not found. Install from https://rustup.rs"; exit 1; }
    @command -v psql >/dev/null 2>&1 || { echo "[WARNING] PostgreSQL not found. Required for database features."; }
    @echo "[OK] All core dependencies found"

# Install wasm-pack if not installed
install-wasm:
    @echo "Checking for wasm-pack..."
    @command -v wasm-pack >/dev/null 2>&1 || { echo "Installing wasm-pack..."; curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh; }
    @echo "[OK] wasm-pack is installed"

# Build WebAssembly components
build-wasm:
    @echo "Building WebAssembly components..."
    @cd packages/catlog-wasm && wasm-pack build
    @echo "[OK] WebAssembly components built"

# Build the project for production
build: check-flox build-wasm
    @echo "Building project for production..."
    @cd packages/frontend && pnpm run build || npm run build
    @echo "[OK] Build complete!"