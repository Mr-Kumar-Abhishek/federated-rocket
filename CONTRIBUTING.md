# Contributing to Federated Rocket

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful, inclusive, and constructive. Focus on technical merit and collaboration.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR-USERNAME/federated-rocket.git`
3. Set up upstream: `git remote add upstream https://github.com/Mr-Kumar-Abhishek/federated-rocket.git`
4. Create a feature branch: `git checkout -b feature/my-feature`

## Development Workflow

### 1. Code Style

- Run `cargo fmt` before committing
- Follow Rust API guidelines and naming conventions
- Use `clippy` and address all warnings: `cargo clippy --workspace`
- Prefer explicit type annotations over `_` inference in public APIs

### 2. Architecture

The project follows a strict 3-layer architecture:
- **Foundation**: No dependencies on other workspace crates
- **Domain**: Depend only on foundation crates
- **Application**: Depend on domain crates

A dependency should never point upward (application → domain → foundation).

### 3. Adding a New Crate

1. Create the crate directory under `crates/`
2. Add to workspace `Cargo.toml` members
3. Create `Cargo.toml` with `version = "0.1.0"` and `edition = "2021"`
4. Create `src/lib.rs` with module declarations
5. Add tests: minimum 80% line coverage for new code

### 4. Adding a New Component Type

1. Add variant to `RocketComponent` enum in [`core/src/component.rs`](crates/core/src/component.rs)
2. Create data struct with all geometric and physical properties
3. Add serialization support (Serialize/Deserialize)
4. Update `component_type()` and `bounding_box()` methods
5. Add Barrowman calculations in [`aero/src/barrowman.rs`](crates/aero/src/barrowman.rs)
6. Add .ork serialization in [`fileio/src/ork.rs`](crates/fileio/src/ork.rs)
7. Add tests for all new code

### 5. Testing Requirements

- All new code must have tests
- `cargo test --workspace` must pass before submitting PR
- For simulation changes, run validation: `cargo run -p federated-rocket-validation --bin validate`
- For performance-critical changes, run benchmarks

### 6. Commit Messages

Use descriptive multiline commit messages:

```
feat(scope): brief description

Detailed explanation of what was changed and why.

Key points:
- Point 1
- Point 2

Testing: tests added, all pass
```

### 7. Pull Request Process

1. Ensure all CI checks pass (format, clippy, test, build)
2. Update documentation if needed
3. Add changelog entry
4. Submit PR with clear description of changes
5. Request review from maintainers

## Crate Dependency Map

```
core ──┬── math ──┬── physics ──┬── aero ──┬── simulation ──┬── optimization
       │          │              │           │                │
       │          │              │           └── motor-db ────┤
       │          │              │                            │
       │          │              └── fileio ──────────────────┤
       │          │                                           │
       └──────────┴───────────────────────────────────────────┴── cli
                                                                gui
```

## Performance Guidelines

- Simulations must run faster than real-time
- Aero calculations: <100µs per evaluation
- File I/O: <2s for typical .ork files
- GUI rendering: >30 FPS
