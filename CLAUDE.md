# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

AI Manager is a background resident application that leverages advanced LLM
reasoning capabilities to assist with schedule management, administrative tasks,
and automation of simple repetitive operations. The project adopts a
microarchitecture approach to completely separate LLM, database, the main
system, and external services for improved maintainability.

## Development Commands

### Setup

```bash
# Initial project setup (recommended first step)
./scripts/setup.sh

# Development server with hot reloading
./scripts/dev.sh
```

### Build & Test

```bash
# Comprehensive testing suite
./scripts/test.sh

# Quick workspace tests
cargo test --workspace

# Production build with distribution
./scripts/build.sh

# Development build
cargo build --workspace

# Release build
cargo build --workspace --release
```

### Development

```bash
# Run core service directly
cargo run -p ai-manager-core

# Run specific service tests
cargo test -p ai-manager-core
cargo test -p ai-manager-llm-service
cargo test -p ai-manager-shared

# Check service compilation
cargo check -p ai-manager-core

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace --all-targets
```

### CI/CD & Quality Assurance

```bash
# Run local CI simulation (all checks)
./scripts/ci-local.sh

# Pre-commit hooks setup
pre-commit install
pre-commit run --all-files

# Docker build and test
docker build -t ai-manager .

# Security audit
cargo install cargo-audit
cargo audit

# Dependency analysis
cargo install cargo-machete
cargo machete
```

## Project Status

### ✅ **Phase 1 - COMPLETED**

- ✅ **Cargo Workspace Setup** - Complete microservice architecture
- ✅ **Event Bus System** - tokio::mpsc based message routing with broadcasting
- ✅ **Core Service** - Service orchestration with health monitoring and auto-restart
- ✅ **LLM Service** - Multi-provider abstraction (OpenAI, Claude) with usage tracking
- ✅ **Shared Types** - Comprehensive message types and error handling
- ✅ **Development Tooling** - Production-ready scripts for setup, dev, test, build
- ✅ **Configuration System** - TOML-based config with environment variable overrides
- ✅ **CI/CD Pipeline** - GitHub Actions with comprehensive testing and releases
- ✅ **Quality Assurance** - Pre-commit hooks, security scanning, Docker support

### 🔄 **Phase 2 - Ready to Implement**

- 🔄 **Data Service** - Complete database abstraction with SQLite/PostgreSQL support
- 🔄 **External Services** - Google Calendar and Email processing integration
- 🔄 **UI Foundation** - Tauri + React chat interface

### 📋 **Phase 3 - Future**

- 📋 **Voice Interface** - Speech recognition and synthesis
- 📋 **PC Automation** - Advanced automation capabilities

## Architecture

- **Language**: Rust with tokio async runtime
- **Architecture**: Event-driven microservices with complete separation of:
  - **Core Service** (orchestration, service management, health monitoring)
  - **LLM Service** (multi-provider AI communication with usage tracking)
  - **Data Service** (database abstraction layer - SQLite/PostgreSQL/External DB)
  - **External Service** (Google Calendar, Email, notifications)
  - **UI Service** (Tauri + React desktop application)

## Project Structure

```text
ai-manager/
├── Cargo.toml              # Workspace configuration
├── CLAUDE.md               # This file - development guidance
├── README.md               # Project overview
├── .gitignore              # Git ignore rules
├── .env.example            # Environment variables template
│
├── crates/                 # Rust microservices
│   ├── core/               # ✅ Main orchestration service
│   ├── llm-service/        # ✅ LLM API integrations (OpenAI, Claude)
│   ├── data-service/       # 🔄 Database abstraction layer
│   ├── external-service/   # 🔄 Google Calendar, Email integration
│   └── shared/             # ✅ Common types, messages, errors
│
├── ui/                     # 🔄 Tauri desktop application
│   ├── src-tauri/          # Rust backend for UI
│   └── src/                # React frontend
│
├── docs/                   # ✅ Complete design documentation
│   ├── requirements.md     # Requirements specification
│   ├── tech-stack.md       # Technology choices and rationale
│   ├── architecture.md     # Microservice architecture design
│   ├── development-plan.md # Phase-based development strategy
│   └── project-structure.md# Project organization
│
├── config/                 # ✅ Configuration files
│   ├── default.toml        # Default configuration (created by setup)
│   └── user.toml           # User overrides (gitignored)
│
├── scripts/                # ✅ Development automation
│   ├── setup.sh            # Project initialization
│   ├── dev.sh              # Development server
│   ├── test.sh             # Testing suite
│   └── build.sh            # Production build
│
├── data/                   # Application data (gitignored)
├── logs/                   # Log files (gitignored)
└── credentials/            # API credentials (gitignored)
```

## Key Technologies & Implementation Details

### Core Infrastructure

- **Event Bus**: tokio::mpsc channels with message routing and broadcasting
- **Service Management**: Health monitoring, auto-restart, graceful shutdown
- **Configuration**: TOML files with environment variable overrides
- **Error Handling**: Comprehensive error types with recovery strategies
- **Logging**: Structured logging with configurable levels

### LLM Integration

- **Provider Abstraction**: Pluggable LLM providers (OpenAI, Claude implemented)
- **Prompt Management**: Template system with variable substitution
- **Usage Tracking**: Token usage monitoring and cost estimation
- **Error Recovery**: Retry logic and circuit breaker patterns

### Database Abstraction

- **Multi-Database Support**: SQLite (default), PostgreSQL, External DB
- **Connection Management**: Connection pooling and health checks
- **Migration System**: Database schema versioning
- **Repository Pattern**: Clean data access abstraction

## Development Notes

### Working with Services

- All services communicate via async message passing (tokio::mpsc)
- Each service is independently testable and deployable
- Services register with the event bus for message routing
- Health monitoring ensures service availability

### Message Patterns

```rust
// User input flows: UI → Core → LLM → Core → UI
ServiceMessage::UserInput { content, timestamp, user_id }
ServiceMessage::LLMRequest { prompt, context, provider, request_id }
ServiceMessage::LLMResponse { content, usage, request_id }
ServiceMessage::SystemResponse { content, message_type, timestamp }

// System management flows
ServiceMessage::ServiceHealthCheck { service_id }
ServiceMessage::ServiceHealthResponse { service_id, status }
ServiceMessage::ShutdownService { service_id }
```

### Configuration Management

- Default configuration in `config/default.toml`
- User overrides in `config/user.toml`
- Environment variables with `AI_MANAGER_` prefix
- Validation on startup with helpful error messages

### Testing Strategy

- Unit tests for individual components
- Integration tests for service communication
- Health check verification
- Configuration validation tests

## ⚠️ IMPORTANT: Development Guidelines

### Before Making Changes

1. **Understand the Current State**:
   - Phase 1 is complete and functional
   - Core services are production-ready
   - Build and test systems are established

2. **Read Design Documents** (if implementing new features):
   - `docs/requirements.md` - What needs to be built
   - `docs/architecture.md` - How services interact
   - `docs/development-plan.md` - Implementation phases

3. **Follow Architectural Principles**:
   - Event-driven microservice architecture
   - Database abstraction (never SQLite-only)
   - Async message passing for all communication
   - Service independence and testability

### Development Workflow

1. **Setup**: Run `./scripts/setup.sh` for initial environment
2. **Development**: Use `./scripts/dev.sh` for hot-reloading development
3. **Testing**: Run `./scripts/test.sh` for comprehensive testing
4. **Building**: Use `./scripts/build.sh` for production builds

### Code Quality Standards

- All services must compile without warnings
- Tests must pass before committing
- Follow Rust idioms and best practices
- Maintain comprehensive error handling
- Document public APIs
- Pre-commit hooks must pass (formatting, linting, security)
- CI/CD pipeline verification required for all changes

### When Adding New Services

1. Create new crate in `crates/` directory
2. Add to workspace in root `Cargo.toml`
3. Implement message handling patterns
4. Register with event bus
5. Add health check capabilities
6. Include comprehensive tests

## Current Implementation Status

### Completed & Production Ready ✅

- **Workspace Architecture**: Complete separation of concerns
- **Event Bus System**: Full message routing with broadcasting
- **Core Service**: Service orchestration and management
- **LLM Service**: Multi-provider support with usage tracking
- **Shared Infrastructure**: Types, errors, constants
- **Development Tooling**: Complete automation scripts
- **Documentation**: Comprehensive design documents
- **CI/CD Infrastructure**: GitHub Actions, pre-commit hooks, Docker support
- **Quality Assurance**: Automated testing, security scanning, code formatting

### Ready for Implementation 🔄

- **Data Service**: Database layer (structure exists, needs completion)
- **External Services**: Calendar and email integration
- **UI Layer**: Tauri + React interface

### Key Achievements

- ✅ **Zero-downtime service management** with health monitoring
- ✅ **Multi-LLM provider support** with cost tracking
- ✅ **Production-ready build system** with distribution packaging
- ✅ **Comprehensive testing framework** with multiple test types
- ✅ **Database abstraction ready** for multiple backends
- ✅ **Event-driven architecture** with proper service isolation
- ✅ **Enterprise-grade CI/CD** with automated quality checks
- ✅ **Docker containerization** with multi-stage builds
- ✅ **Security-first development** with secret detection and audit tools

The codebase is now ready for Phase 2 development with a solid, scalable
foundation that follows all architectural requirements from the design
documents.
