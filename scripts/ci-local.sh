#!/bin/bash

# Local CI simulation script for AI Manager
# This script runs the same checks as CI locally for faster development

set -e

echo "🚀 Running local CI simulation for AI Manager..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Store start time
start_time=$(date +%s)

# Step 1: Check formatting
print_status "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt --all' to fix"
    exit 1
fi

# Step 2: Run clippy
print_status "Running clippy lints..."
if cargo clippy --workspace --all-targets -- -D warnings; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues"
    exit 1
fi

# Step 3: Build workspace
print_status "Building workspace..."
if cargo build --workspace; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Step 4: Run tests
print_status "Running test suite..."
if cargo test --workspace; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    exit 1
fi

# Step 5: Check documentation
print_status "Checking documentation..."
if cargo doc --workspace --no-deps; then
    print_success "Documentation builds successfully"
else
    print_warning "Documentation has issues"
fi

# Step 6: Security audit (if cargo-audit is installed)
print_status "Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues"
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Step 7: Check for unused dependencies (if cargo-machete is installed)
print_status "Checking for unused dependencies..."
if command -v cargo-machete >/dev/null 2>&1; then
    if cargo machete; then
        print_success "No unused dependencies found"
    else
        print_warning "Unused dependencies found"
    fi
else
    print_warning "cargo-machete not installed. Install with: cargo install cargo-machete"
fi

# Step 8: Build release version
print_status "Building release version..."
if cargo build --workspace --release; then
    print_success "Release build completed successfully"
else
    print_error "Release build failed"
    exit 1
fi

# Step 9: Check if pre-commit hooks would pass
print_status "Checking pre-commit hooks..."
if command -v pre-commit >/dev/null 2>&1; then
    if pre-commit run --all-files; then
        print_success "Pre-commit hooks passed"
    else
        print_warning "Pre-commit hooks found issues"
    fi
else
    print_warning "pre-commit not installed. Install with: pip install pre-commit"
fi

# Calculate and display total time
end_time=$(date +%s)
duration=$((end_time - start_time))
minutes=$((duration / 60))
seconds=$((duration % 60))

echo ""
echo "============================================"
print_success "All CI checks completed successfully!"
echo -e "${BLUE}[INFO]${NC} Total time: ${minutes}m ${seconds}s"
echo "============================================"

# Additional information
echo ""
echo "🔍 To install missing tools:"
echo "  cargo install cargo-audit cargo-machete"
echo "  pip install pre-commit"
echo "  pre-commit install"
echo ""
echo "📝 To run individual checks:"
echo "  cargo fmt --all"
echo "  cargo clippy --workspace --all-targets"
echo "  cargo test --workspace"
echo "  cargo build --workspace --release"
echo ""
echo "🚀 Ready for CI! Your code should pass all GitHub Actions checks."
