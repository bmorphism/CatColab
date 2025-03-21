# CatColab - Universal Cross-Platform justfile

# Detect platform
host := `uname -s`
is_linux := `[ "$(uname -s)" = "Linux" ] && echo true || echo false`
is_macos := `[ "$(uname -s)" = "Darwin" ] && echo true || echo false`

# Show available commands
default:
    @just --list

# Check if the Flox environment is active
check-flox:
    @if [ -z "${FLOX_ENV:-}" ]; then \
      echo "[WARNING] Not using Flox environment. Consider: flox activate"; \
    else \
      echo "[OK] Using Flox environment: ${FLOX_ENV}"; \
    fi

# Setup project dependencies and tools
setup: check-flox
    @echo "[1/3] Installing frontend dependencies..."
    @cd packages/frontend && (pnpm install || npm install)
    @echo "[2/3] Installing required tools..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      flox install postgresql netcat wasm-pack; \
    else \
      command -v wasm-pack >/dev/null || (echo "Installing wasm-pack..." && curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh); \
    fi
    @echo "[3/3] Building WebAssembly components..."
    @cd packages/catlog-wasm && wasm-pack build
    @echo "[OK] Setup complete! Run 'just db-setup' next."

# Setup local database
db-setup: check-flox
    @echo "Setting up database..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      echo "[INFO] Using Flox PostgreSQL on port 6969"; \
      mkdir -p .postgres-data; \
      if [ ! -f ".postgres-data/postgresql.conf" ]; then \
        initdb -D .postgres-data; \
        echo "port = 6969" >> .postgres-data/postgresql.conf; \
      fi; \
      pg_ctl -D .postgres-data -l .postgres-data/logfile start || echo "PostgreSQL already running"; \
      createdb -p 6969 catcolab 2>/dev/null || echo "Database ready"; \
      echo "DATABASE_URL=postgres://localhost:6969/catcolab" > .env; \
      echo "DATABASE_URL=postgres://localhost:6969/catcolab" > packages/backend/.env; \
    else \
      echo "[INFO] Using system PostgreSQL"; \
      createdb catcolab 2>/dev/null || echo "Database ready"; \
      echo "DATABASE_URL=postgres:///catcolab" > .env; \
      echo "DATABASE_URL=postgres:///catcolab" > packages/backend/.env; \
    fi
    @echo "[INFO] Running migrations..."
    @cd packages/backend && (cargo sqlx migrate run || echo "[WARNING] Install sqlx-cli with: cargo install sqlx-cli")
    @echo "[OK] Database ready! You can now run 'just run'"

# Run both frontend and backend
run: check-flox
    @echo "Starting CatColab application..."
    @echo "-> Backend: http://localhost:8000"
    @echo "-> Frontend: http://localhost:3000"
    @cd packages/backend && cargo run & BACKEND_PID=$!
    @cd packages/frontend && (pnpm run dev || npm run dev) || kill $BACKEND_PID

# No-database version for testing
run-staging: check-flox
    @echo "Running CatColab with staging backend (no local DB)..."
    @cd packages/frontend && (pnpm run dev --mode staging || npm run dev -- --mode staging)

# Run backend server only
run-backend: check-flox
    @cd packages/backend && cargo run

# Run frontend server only
run-frontend: check-flox
    @cd packages/frontend && (pnpm run dev || npm run dev)