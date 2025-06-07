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

# Recreate `.topos` helper utilities
topos:
    @mkdir -p .topos
    @printf '%s\n' \
        '# CatColab Fork Guide' \
        '' \
        'This directory documents how this repository diverges from the upstream [ToposInstitute/CatColab](https://github.com/ToposInstitute/CatColab) project.' \
        'It collects auxiliary scripts and notes so that updates from upstream can be managed with minimal friction.' \
        '' \
        '## Differences from upstream' \
        '' \
        'The main branch of this repository tracks `ToposInstitute/CatColab:main` with a' \
        'few additional developer conveniences:' \
        '' \
        '- Flox environment files under `.flox/` for deterministic tooling.' \
        '- A `justfile` and `justfile-tests.yml` workflow for quick project setup and CI checks.' \
        '- Local notes in `CLAUDE.md`.' \
        '' \
        'Keeping these files separate helps reduce merge conflicts when pulling in changes from upstream.' \
        '' \
        '## Updating from upstream' \
        '' \
        'Run `./.topos/update-from-upstream.sh` to fetch and merge the latest commits from' \
        '`ToposInstitute/CatColab`. The script will add the `topos` remote if it does not already exist.' \
        '' \
        '```' \
        '# update local main branch' \
        './.topos/update-from-upstream.sh' \
        '```' \
        '' \
        'After merging, resolve any conflicts, run the project checks, and commit as usual.' \
        > .topos/README.md
    @printf '%s\n' \
        '#!/usr/bin/env bash' \
        'set -euo pipefail' \
        '' \
        'REMOTE="topos"' \
        'BRANCH="main"' \
        '' \
        'if ! git remote | grep -q "^${REMOTE}$"; then' \
        '  git remote add ${REMOTE} https://github.com/ToposInstitute/CatColab.git' \
        'fi' \
        '' \
        'git fetch ${REMOTE}' \
        'git merge --ff-only ${REMOTE}/${BRANCH}' \
        > .topos/update-from-upstream.sh
    @chmod +x .topos/update-from-upstream.sh
    @echo "[OK] .topos utilities ready"

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
    @echo "Using system username for PostgreSQL connection"
    @echo "DATABASE_URL=postgres:///${USER}" > .env
    @createdb catcolab 2>/dev/null || echo "Database exists"
    @cd packages/backend && cargo sqlx migrate run || echo "Run 'cargo install sqlx-cli' if this fails"
    @echo "[OK] Database ready! You can now run 'just run'"

# Ironclad database setup with resilience mechanisms, health checks, and recovery paths
db-setup-ironclad: check-flox
    @echo "\033[1;36mSetting up database with enhanced reliability...\033[0m"
    @echo "[1/6] Checking PostgreSQL status..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      if ! pg_isready -h localhost -p 6969 >/dev/null 2>&1; then \
        echo "\033[1;33m[RECOVERY] Starting PostgreSQL service...\033[0m"; \
        mkdir -p .postgres-data; \
        if [ ! -f ".postgres-data/postgresql.conf" ]; then \
          initdb -D .postgres-data; \
          echo "port = 6969" >> .postgres-data/postgresql.conf; \
          echo "max_connections = 100" >> .postgres-data/postgresql.conf; \
          echo "shared_buffers = 128MB" >> .postgres-data/postgresql.conf; \
          echo "log_statement = 'all'" >> .postgres-data/postgresql.conf; \
        fi; \
        pg_ctl -D .postgres-data -l .postgres-data/logfile start || \
          (echo "\033[1;31m[ERROR] Failed to start PostgreSQL\033[0m" && exit 1); \
        echo "Waiting for PostgreSQL to initialize..."; \
        for i in {1..10}; do \
          if pg_isready -h localhost -p 6969 >/dev/null 2>&1; then \
            echo "\033[1;32m[OK] PostgreSQL is running\033[0m"; \
            break; \
          fi; \
          if [ "$i" -eq 10 ]; then \
            echo "\033[1;31m[ERROR] PostgreSQL failed to start after waiting\033[0m"; \
            exit 1; \
          fi; \
          echo -n "."; \
          sleep 1; \
        done; \
      else \
        echo "\033[1;32m[OK] PostgreSQL is already running\033[0m"; \
      fi; \
      POSTGRES_PORT=6969; \
    else \
      echo "\033[1;33m[INFO] Using system PostgreSQL\033[0m"; \
      if ! pg_isready >/dev/null 2>&1; then \
        echo "\033[1;31m[ERROR] System PostgreSQL is not running\033[0m"; \
        echo "Please start PostgreSQL with:"; \
        echo "  macOS: brew services start postgresql"; \
        echo "  Linux: sudo systemctl start postgresql"; \
        exit 1; \
      fi; \
      POSTGRES_PORT=5432; \
    fi
    
    @echo "[2/6] Verifying PostgreSQL permissions..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      if ! psql -h localhost -p 6969 -d postgres -c "" >/dev/null 2>&1; then \
        echo "\033[1;33m[RECOVERY] Setting up PostgreSQL access...\033[0m"; \
        if [ -f ".postgres-data/pg_hba.conf" ]; then \
          sed -i.bak '/^host/d' .postgres-data/pg_hba.conf; \
          echo "host all all 127.0.0.1/32 trust" >> .postgres-data/pg_hba.conf; \
          pg_ctl -D .postgres-data reload; \
        fi; \
      fi; \
    fi
    
    @echo "[3/6] Setting up database connection configuration..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      echo "DATABASE_URL=postgres://localhost:6969/catcolab" > .env; \
      echo "DATABASE_URL=postgres://localhost:6969/catcolab" > packages/backend/.env; \
    else \
      echo "DATABASE_URL=postgres:///catcolab" > .env; \
      echo "DATABASE_URL=postgres:///catcolab" > packages/backend/.env; \
    fi
    
    @echo "[4/6] Creating database (with retry mechanism)..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      createdb -h localhost -p 6969 catcolab 2>/dev/null || \
        echo "\033[1;32m[OK] Database already exists\033[0m" || \
        (echo "\033[1;33m[RECOVERY] Retrying database creation...\033[0m" && \
         sleep 2 && createdb -h localhost -p 6969 catcolab); \
    else \
      createdb catcolab 2>/dev/null || \
        echo "\033[1;32m[OK] Database already exists\033[0m" || \
        (echo "\033[1;33m[RECOVERY] Retrying database creation...\033[0m" && \
         sleep 2 && createdb catcolab); \
    fi
    
    @echo "[5/6] Running migrations with verification..."
    @cd packages/backend && \
      command -v sqlx >/dev/null 2>&1 || \
        (echo "\033[1;33m[RECOVERY] Installing sqlx-cli...\033[0m" && \
         cargo install sqlx-cli --no-default-features --features postgres); \
      (cargo sqlx migrate run && echo "\033[1;32m[OK] Migrations completed successfully\033[0m") || \
        (echo "\033[1;31m[ERROR] Migration failed\033[0m" && exit 1)
    
    @echo "[6/6] Verifying database schema integrity..."
    @if [ -n "${FLOX_ENV:-}" ]; then \
      tables_count=$(psql -h localhost -p 6969 -d catcolab -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" | tr -d '[:space:]'); \
      if [ "$tables_count" -gt 0 ]; then \
        echo "\033[1;32m[OK] Database schema verified with $tables_count tables\033[0m"; \
      else \
        echo "\033[1;31m[ERROR] Database schema verification failed - no tables found\033[0m"; \
        exit 1; \
      fi; \
    else \
      tables_count=$(psql -d catcolab -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" | tr -d '[:space:]'); \
      if [ "$tables_count" -gt 0 ]; then \
        echo "\033[1;32m[OK] Database schema verified with $tables_count tables\033[0m"; \
      else \
        echo "\033[1;31m[ERROR] Database schema verification failed - no tables found\033[0m"; \
        exit 1; \
      fi; \
    fi
    
    @echo "\033[1;36m✅ Ironclad database setup complete!\033[0m"
    @echo "Run 'just run' to start the application"

# Display database setup decision tree with causal dependencies
explain-db:
    @echo "u001b[1;36m# CatColab Database Setup Decision Treeu001b[0m"
    @echo "# Shows causal dependencies and path-dependent decision points"
    @echo "u001b[0;37m"
    @echo "START"
    @echo "└── Is Flox environment active? ────────────────────┐"
    @echo "    ├── YES ──► PostgreSQL available? ─────────────┤"
    @echo "    │           ├── YES ──► User has DB permissions? │"
    @echo "    │           │           ├── YES ──► ┌─────────┴───────────────────────┐"
    @echo "    │           │           │           │ 1. Set DATABASE_URL in .env    │"
    @echo "    │           │           │           │ 2. createdb catcolab           │"
    @echo "    │           │           │           │ 3. Run migrations              │"
    @echo "    │           │           │           └───────────────────────────────┘"
    @echo "    │           │           │           │"
    @echo "    │           │           │           v"
    @echo "    │           │           │           SUCCESS: Database ready"
    @echo "    │           │           │"
    @echo "    │           │           └── NO ───► Action: Grant user permissions"
    @echo "    │           │                       └── Try again"
    @echo "    │           │"
    @echo "    │           └── NO ───► Action: Start PostgreSQL service"
    @echo "    │                       ├── Started successfully ──► Continue"
    @echo "    │                       └── Failed to start ────► ERROR"
    @echo "    │"
    @echo "    └── NO ───► System PostgreSQL available? ──────┐"
    @echo "                ├── YES ──► ┌───────────────────────┘"
    @echo "                │           │ 1. Set DATABASE_URL=postgres:///catcolab"
    @echo "                │           │ 2. createdb catcolab"
    @echo "                │           │ 3. Run migrations with cargo sqlx"
    @echo "                │           └────────────────────────────"
    @echo "                │           │"
    @echo "                │           v"
    @echo "                │           Is sqlx-cli installed?"
    @echo "                │           ├── YES ──► Run migrations"
    @echo "                │           └── NO ───► Install sqlx-cli"
    @echo "                │                       └── Try migrations again"
    @echo "                │"
    @echo "                └── NO ───► Options:"
    @echo "                            ├── Install PostgreSQL"
    @echo "                            ├── Use Docker container"
    @echo "                            └── Use staging mode (no DB)"
    @echo ""
    @echo "u001b[1;33m# Causal Dependencies in just db-setupu001b[0m"
    @echo ""
    @echo "┌─────────────────────────────────────────────────────────────┐"
    @echo "│ COMMAND: just db-setup                                      │"
    @echo "├─────────────────────────────────────────────────────────────┤"
    @echo "│ 1. check-flox ────┬─► Environment active    ──► Use Flox DB │"
    @echo "│                    └─► Environment inactive ──► Use system DB│"
    @echo "│                                                             │"
    @echo "│ 2. Set DATABASE_URL in .env files                           │"
    @echo "│    CAUSES: Backend can locate and connect to database        │"
    @echo "│                                                             │"
    @echo "│ 3. Create database with createdb                            │"
    @echo "│    CAUSES: Empty database exists for migrations              │"
    @echo "│    REQUIRES: PostgreSQL running, user permissions            │"
    @echo "│                                                             │"
    @echo "│ 4. Run migrations with cargo sqlx                           │"
    @echo "│    CAUSES: Schema creation (tables, constraints)             │"
    @echo "│    REQUIRES: Database exists, sqlx-cli installed             │"
    @echo "└─────────────────────────────────────────────────────────────┘"
    @echo ""
    @echo "u001b[1;35m# Potential Failure Pointsu001b[0m"
    @echo ""
    @echo "1. PostgreSQL not running"
    @echo "   ├── Symptom: createdb fails with connection refused"
    @echo "   └── Fix: Start PostgreSQL service"
    @echo ""
    @echo "2. Missing user permissions"
    @echo "   ├── Symptom: 'permission denied to create database'"
    @echo "   └── Fix: Grant CREATEDB permission to user"
    @echo ""
    @echo "3. sqlx-cli not installed"
    @echo "   ├── Symptom: 'command not found: sqlx'"
    @echo "   └── Fix: cargo install sqlx-cli"
    @echo ""
    @echo "4. Migration conflicts"
    @echo "   ├── Symptom: 'error applying migration'"
    @echo "   └── Fix: dropdb catcolab && just db-setup"
    @echo ""
    @echo "u001b[1;32m# Execution Pathsu001b[0m"
    @echo ""
    @echo "┌─ HAPPY PATH ──────────────────────────────────────────────┐"
    @echo "│ PostgreSQL running → Database created → Migrations run    │"
    @echo "│ \_ All commands succeed, database ready for use           │"
    @echo "└────────────────────────────────────────────────────────────┘"
    @echo ""
    @echo "┌─ RECOVERY PATHS ───────────────────────────────────────────┐"
    @echo "│ Path 1: PostgreSQL not running → Start service → Try again │"
    @echo "│ Path 2: Database exists → Skip creation → Run migrations   │"
    @echo "│ Path 3: Sqlx missing → Install → Run migrations            │"
    @echo "└────────────────────────────────────────────────────────────┘"
    @echo ""
    @echo "u001b[1;34m# Dynamic System Effectsu001b[0m"
    @echo ""
    @echo "├── R1: Reinforcing Loop: Better tooling → Faster setup → More development"
    @echo "├── B1: Balancing Loop: Database issues → Fixes applied → Issues resolved"
    @echo "└── Delay Effect: PostgreSQL startup time affects setup success"
    @echo ""
    @echo "u001b[0;37m"
    @echo "Run 'just db-setup' to execute the actual database setup process."

# Full deploy from scratch with local components (with database)
deploy-local: check-deps install-wasm build-wasm setup db-setup run

# Full deploy from scratch with local components (without database, using staging mode)
deploy-local-staging: check-deps install-wasm build-wasm setup run-staging

# Check for required dependencies
check-deps:
    @echo "Checking dependencies..."
    @command -v node >/dev/null 2>&1 || { echo "[ERROR] Node.js not found"; exit 1; }
    @command -v cargo >/dev/null 2>&1 || { echo "[ERROR] Rust not found. Install from https://rustup.rs"; exit 1; }
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

# Configure Claude with Exa MCP server
mcp:
    @echo "🔍 Adding Exa MCP server to Claude Code..."
    @echo "[1/4] 📦 Creating project-relative .topos directory"
    @mkdir -p .topos
    
    @echo "[2/4] 📥 Cloning Exa MCP Server repository"
    @if [ -d ".topos/exa-mcp-server" ]; then \
      echo "     ↳ Exa MCP repository already exists, pulling latest changes"; \
      cd .topos/exa-mcp-server && git pull; \
    else \
      echo "     ↳ Cloning fresh repository from GitHub"; \
      git clone https://github.com/exa-labs/exa-mcp-server.git .topos/exa-mcp-server; \
    fi
    
    @echo "[3/4] 🛠️ Installing Exa MCP Server dependencies"
    @cd .topos/exa-mcp-server && npm install && npm run build
    @echo "     ↳ Server build complete"
    
    @echo "[4/4] ⚙️ Adding Exa MCP Server to Claude Code"
    @echo "     ↳ Please enter your Exa API key (from https://dashboard.exa.ai/api-keys):"
    @read -p "       API Key: " EXA_API_KEY && \
      echo "     ↳ Running Claude CLI add command..." && \
      claude mcp add exa npx $(pwd)/.topos/exa-mcp-server/build/index.js --env EXA_API_KEY="$EXA_API_KEY"
    
    @echo "✅ Exa MCP Server added to Claude Code!"
    @echo "🔄 Please restart Claude Desktop for changes to take effect."
    @echo "🌐 You can now use web search in Claude with commands like:"
    @echo "   'Search the web for infinity-topos framework'"
    @echo "   'Find recent information about collaborative editing'"
    @echo "🔗 For more information, visit: https://github.com/exa-labs/exa-mcp-server"
    @echo ""
    @echo "To verify the installation, run: claude mcp list"