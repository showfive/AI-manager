#!/bin/bash

# AI Manager Development Server Script
# Starts the core service in development mode with hot reloading

set -e

echo "🔥 Starting AI Manager development server..."

# Check if configuration exists
if [ ! -f "config/default.toml" ]; then
    echo "❌ Configuration not found. Run './scripts/setup.sh' first."
    exit 1
fi

# Load environment variables if .env exists
if [ -f ".env" ]; then
    echo "📂 Loading environment variables from .env"
    export $(cat .env | grep -v '^#' | xargs)
fi

# Set development environment
export RUST_ENV=development
export RUST_LOG=${RUST_LOG:-"ai_manager_core=debug,ai_manager_shared=info,ai_manager_llm_service=debug"}

# Create directories if they don't exist
mkdir -p data logs

echo "🚀 Starting core service..."
echo "📊 Log level: $RUST_LOG"
echo "🗂️  Database: data/ai_manager.db"
echo "📝 Logs: logs/ai_manager.log"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Run with cargo watch for hot reloading if available
if command -v cargo-watch &> /dev/null; then
    echo "🔄 Using cargo-watch for hot reloading"
    cargo watch -x "run -p ai-manager-core"
else
    echo "💡 Install cargo-watch for hot reloading: cargo install cargo-watch"
    cargo run -p ai-manager-core
fi