#!/bin/bash

# AI Manager Test Script
# Runs all tests with various configurations

set -e

echo "🧪 Running AI Manager tests..."

# Set test environment
export RUST_ENV=test
export RUST_LOG=${RUST_LOG:-"warn"}

# Function to run tests with error handling
run_tests() {
    local test_type="$1"
    local test_args="$2"
    
    echo "📋 Running $test_type tests..."
    
    if cargo test $test_args; then
        echo "✅ $test_type tests passed"
    else
        echo "❌ $test_type tests failed"
        return 1
    fi
}

# Run unit tests
run_tests "unit" "--workspace --lib"

# Run integration tests
run_tests "integration" "--workspace --test '*'"

# Run doc tests
run_tests "documentation" "--workspace --doc"

# Run tests with coverage if tarpaulin is available
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📊 Generating test coverage report..."
    cargo tarpaulin --workspace --out Html --output-dir target/coverage
    echo "📈 Coverage report generated at target/coverage/tarpaulin-report.html"
else
    echo "💡 Install cargo-tarpaulin for coverage reports: cargo install cargo-tarpaulin"
fi

# Run clippy for linting
echo "🔍 Running clippy lints..."
if cargo clippy --workspace --all-targets -- -D warnings; then
    echo "✅ Clippy lints passed"
else
    echo "⚠️  Clippy found issues"
fi

# Check formatting
echo "📏 Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "⚠️  Code formatting issues found. Run 'cargo fmt' to fix."
fi

# Security audit
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running security audit..."
    if cargo audit; then
        echo "✅ Security audit passed"
    else
        echo "⚠️  Security vulnerabilities found"
    fi
else
    echo "💡 Install cargo-audit for security audits: cargo install cargo-audit"
fi

echo ""
echo "🎉 Test suite complete!"
echo ""
echo "💡 Tips:"
echo "  - Run specific tests: cargo test test_name"
echo "  - Run tests for specific crate: cargo test -p ai-manager-core"
echo "  - Run tests with output: cargo test -- --nocapture"
echo "  - Install recommended tools:"
echo "    cargo install cargo-watch cargo-tarpaulin cargo-audit"