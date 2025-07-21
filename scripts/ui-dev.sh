#!/bin/bash

# AI Manager UI Development Script
# Starts the Tauri + React development environment

set -e

echo "🚀 Starting AI Manager UI Development..."

# Check if we're in the right directory
if [ ! -f "CLAUDE.md" ]; then
    echo "❌ Please run this script from the AI Manager root directory"
    exit 1
fi

# Navigate to UI directory
cd ui

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing UI dependencies..."
    npm install
fi

# Start development server
echo "🎯 Starting Tauri development server..."
echo "   Frontend will be available at http://localhost:1420"
echo "   Desktop app will open automatically"
echo ""
echo "Press Ctrl+C to stop the development server"

npm run tauri:dev