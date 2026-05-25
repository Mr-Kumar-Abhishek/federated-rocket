# Federated Rocket 🚀

A modern, high-performance model rocket simulation and design tool written in Rust.

[![CI](https://github.com/Mr-Kumar-Abhishek/federated-rocket/actions/workflows/ci.yml/badge.svg)](https://github.com/Mr-Kumar-Abhishek/federated-rocket/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/Mr-Kumar-Abhishek/federated-rocket/branch/master/graph/badge.svg)](https://codecov.io/gh/Mr-Kumar-Abhishek/federated-rocket)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80.0%2B-orange.svg)](rust-toolchain.toml)

## Features

- **6-DOF Flight Simulation** RK4/AdaptiveRK4 integration with quaternion orientation
- **Barrowman Aerodynamics** with full supersonic corrections and component interference
- **22 Built-in Motors** from Estes, Aerotech, and Quest
- **SQLite Motor Database** with ThrustCurve.org API integration
- **File Format Support** OpenRocket (.ork), RockSim (.rkt), CSV export
- **Design Optimization** Golden Section and Nelder-Mead search
- **Dual Interface** CLI (clap) and Desktop GUI (egui/eframe)
- **Cross-Platform** Windows, macOS, Linux
- **Validation Framework** Numerical comparison against Java OpenRocket

## Quick Start

```bash
# CLI: Simulate a rocket
federated-rocket simulate --file design.ork --motor "Estes C6-5"

# GUI: Launch the desktop application
federated-rocket-gui
```

## Architecture

**10 crates** across 3 layers:

| Layer | Crates |
|-------|--------|
| **Foundation** | `core`, `math`, `validation` |
| **Domain** | `physics`, `aero`, `simulation`, `fileio`, `motor-db`, `optimization` |
| **Application** | `cli`, `gui` |

```
┌─────────────────────────────────────────────┐
│  Application: cli, gui                      │
├─────────────────────────────────────────────┤
│  Domain: physics, aero, simulation,         │
│          fileio, motor-db, optimization      │
├─────────────────────────────────────────────┤
│  Foundation: core, math, validation          │
└─────────────────────────────────────────────┘
```

## Documentation

- [User Guide](USER_GUIDE.md) — Complete usage documentation
- [Changelog](CHANGELOG.md) — Release history
- [Contributing](CONTRIBUTING.md) — Development guidelines
- [docs/](docs/) — Project specifications (SRS, SDD, Implementation Plan)

## Building

```bash
git clone https://github.com/Mr-Kumar-Abhishek/federated-rocket.git
cd federated-rocket
cargo build --release --workspace
```

## Testing

```bash
cargo test --workspace        # 449+ tests
cargo bench                   # Performance benchmarks
cargo run -p federated-rocket-validation --bin validate  # Numerical validation
```

## License

Apache 2.0. See [LICENSE](LICENSE).

## Acknowledgments

- [OpenRocket](https://openrocket.info/) — The reference Java implementation
- [Barrowman Method](https://en.wikipedia.org/wiki/Barrowman_method) — Subsonic aerodynamics
- [ThrustCurve.org](https://www.thrustcurve.org/) — Motor database
