#!/bin/bash

# AI Manager Setup Script
# This script sets up the development environment for AI Manager

set -e  # Exit on any error

echo "🚀 Setting up AI Manager development environment..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Rust found: $(rustc --version)"

# Check if Node.js is installed (for future UI development)
if ! command -v node &> /dev/null; then
    echo "⚠️  Node.js not found. You'll need it for UI development."
    echo "   Install from https://nodejs.org/"
else
    echo "✓ Node.js found: $(node --version)"
fi

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p data
mkdir -p logs
mkdir -p config
mkdir -p credentials

# Create default configuration if it doesn't exist
if [ ! -f "config/default.toml" ]; then
    echo "📝 Creating default configuration..."
    cat > config/default.toml << 'EOF'
[llm]
default_provider = "openai"

[llm.providers.openai]
api_key = "your-openai-api-key-here"
model = "gpt-3.5-turbo"
max_tokens = 2000
temperature = 0.7

[database]
database_type = "SQLite"
connection_string = "sqlite:data/ai_manager.db"
max_connections = 10
enable_logging = false

[external_services.notifications]
enable_desktop = true
enable_sound = true

[ui]
theme = "dark"
enable_system_tray = true

[ui.window_size]
width = 1200
height = 800

[logging]
level = "info"
file_logging = true
log_file_path = "logs/ai_manager.log"
EOF
    echo "✓ Default configuration created at config/default.toml"
    echo "🔧 Please edit config/default.toml to add your API keys"
else
    echo "✓ Configuration already exists"
fi

# Create .env template
if [ ! -f ".env.example" ]; then
    cat > .env.example << 'EOF'
# AI Manager Environment Variables
# Copy this file to .env and fill in your values

# OpenAI API Key
OPENAI_API_KEY=your-openai-api-key-here

# Claude API Key (optional)
CLAUDE_API_KEY=your-claude-api-key-here

# Database URL (optional, overrides config)
DATABASE_URL=sqlite:data/ai_manager.db

# Log level (optional)
RUST_LOG=ai_manager_core=debug,ai_manager_shared=info
EOF
    echo "✓ Environment template created (.env.example)"
fi

# Build all workspace crates
echo "🔨 Building workspace..."
cargo build --workspace

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
cargo test --workspace

if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
else
    echo "⚠️  Some tests failed, but setup can continue"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit config/default.toml to add your API keys"
echo "2. Copy .env.example to .env and customize as needed"
echo "3. Run './scripts/dev.sh' to start the development server"
echo "4. Run './scripts/test.sh' to run tests"
echo "5. Run './scripts/build.sh' for production builds"
echo ""
echo "📚 See docs/ for more information"