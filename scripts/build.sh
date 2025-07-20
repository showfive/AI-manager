#!/bin/bash

# AI Manager Build Script
# Builds the project for production deployment

set -e

echo "🔨 Building AI Manager for production..."

# Set production environment
export RUST_ENV=production

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build workspace in release mode
echo "⚙️  Building workspace in release mode..."
cargo build --workspace --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

# Build individual binaries
echo "📦 Building core service binary..."
cargo build -p ai-manager-core --release

# Copy binaries to a distribution directory
echo "📁 Creating distribution directory..."
mkdir -p dist/bin
mkdir -p dist/config
mkdir -p dist/docs

# Copy binaries
cp target/release/ai-manager-core dist/bin/
echo "✅ Core service binary copied to dist/bin/"

# Copy configuration templates
if [ -f "config/default.toml" ]; then
    cp config/default.toml dist/config/default.toml.example
    echo "✅ Configuration template copied"
fi

# Copy documentation
if [ -d "docs" ]; then
    cp -r docs/* dist/docs/
    echo "✅ Documentation copied"
fi

# Copy scripts
mkdir -p dist/scripts
cp scripts/*.sh dist/scripts/
chmod +x dist/scripts/*.sh
echo "✅ Scripts copied"

# Create production README
cat > dist/README.md << 'EOF'
# AI Manager - Production Distribution

## Quick Start

1. Copy `config/default.toml.example` to `config/default.toml`
2. Edit `config/default.toml` with your API keys and settings
3. Run `./bin/ai-manager-core`

## Files

- `bin/ai-manager-core` - Main application binary
- `config/default.toml.example` - Configuration template
- `scripts/` - Utility scripts
- `docs/` - Documentation

## Configuration

Edit `config/default.toml` to configure:
- LLM provider API keys
- Database settings
- UI preferences
- Logging configuration

## Environment Variables

Set these environment variables to override configuration:
- `OPENAI_API_KEY` - OpenAI API key
- `CLAUDE_API_KEY` - Claude API key
- `DATABASE_URL` - Database connection string
- `RUST_LOG` - Logging level

## Running

```bash
# Set your API key
export OPENAI_API_KEY="your-key-here"  # pragma: allowlist secret

# Run the application
./bin/ai-manager-core
```

For more information, see the documentation in the `docs/` directory.
EOF

# Calculate binary sizes
echo ""
echo "📊 Build Summary:"
echo "  Core service: $(ls -lh dist/bin/ai-manager-core | awk '{print $5}')"
echo "  Total distribution: $(du -sh dist | cut -f1)"

# Verify the binary works
echo ""
echo "🔍 Testing binary..."
if timeout 5s dist/bin/ai-manager-core --help > /dev/null 2>&1; then
    echo "✅ Binary verification passed"
else
    echo "⚠️  Binary verification failed or timed out"
fi

echo ""
echo "🎉 Production build complete!"
echo ""
echo "📦 Distribution ready in ./dist/"
echo "🚀 To deploy:"
echo "  1. Copy the ./dist/ directory to your production server"
echo "  2. Configure config/default.toml with your settings"
echo "  3. Run ./bin/ai-manager-core"
echo ""
echo "💡 Consider creating a systemd service for production deployment"
