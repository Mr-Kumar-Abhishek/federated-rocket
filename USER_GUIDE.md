# Federated Rocket User Guide

## Overview

Federated Rocket is a model rocket simulation and design tool written in Rust. It provides accurate 6-DOF flight simulation using the Barrowman method for aerodynamics, with support for:

- OpenRocket (.ork) and RockSim (.rkt) file formats
- 22 built-in rocket motors (Estes, Aerotech, Quest)
- SQLite motor database with ThrustCurve.org API integration
- 2D trajectory plotting and real-time telemetry dashboard
- 1D and multi-variate design optimization
- CLI and GUI interfaces

## Quick Start

### Installation

Download the latest binary for your platform from the [Releases](https://github.com/Mr-Kumar-Abhishek/federated-rocket/releases) page.

**Windows**: Extract the `.zip` file and run `federated-rocket.exe` (CLI) or `federated-rocket-gui.exe` (GUI).

**Linux/macOS**: Extract the `.tar.gz` file and run `./federated-rocket` or `./federated-rocket-gui`.

### Building from Source

```bash
git clone https://github.com/Mr-Kumar-Abhishek/federated-rocket.git
cd federated-rocket
cargo build --release --workspace
```

## CLI Usage

### Simulate a rocket flight

```bash
# Basic simulation
federated-rocket simulate --file my_rocket.ork

# With motor override, wind, and CSV output
federated-rocket simulate \
    --file my_rocket.ork \
    --motor "Estes C6-5" \
    --wind-speed 5.0 \
    --wind-direction 270 \
    --output trajectory.csv \
    --json

# Configure simulation parameters
federated-rocket simulate \
    --file my_rocket.ork \
    --max-time 60.0 \
    --time-step 0.0005 \
    --rod-clear 3.0 \
    --verbose
```

### View rocket design info

```bash
# Basic info
federated-rocket info my_rocket.ork

# Detailed component tree
federated-rocket info my_rocket.ork --detailed

# JSON output
federated-rocket info my_rocket.ork --json
```

### Browse motors

```bash
# List all motors
federated-rocket motors --list

# Search by manufacturer
federated-rocket motors --manufacturer Aerotech

# Search by designation
federated-rocket motors --designation C6

# Detailed view
federated-rocket motors --list --detailed
```

### Optimize a parameter

```bash
# Optimize fin span for max altitude
federated-rocket optimize \
    --file my_rocket.ork \
    --parameter fin_span \
    --goal altitude \
    --min 2.0 \
    --max 6.0 \
    --iterations 100
```

### Convert file formats

```bash
# Convert .rkt to .ork
federated-rocket convert my_rocket.rkt my_rocket.ork

# Convert .ork to .rkt
federated-rocket convert my_rocket.ork my_rocket.rkt
```

## GUI Usage

### Starting the GUI

```bash
federated-rocket-gui
```

### Main Window Layout

The GUI is organized into three columns:

**Left Panel — Design:**
- Load rocket designs via File > Open or type the path inline
- Browse the component tree with type-specific icons
- Select components to view properties (type, position, parent, children)

**Right Panel — Controls:**
- **Motor Selection**: Browse 22 built-in motors by manufacturer
- **Simulation**: Configure max time, time step, wind speed/direction, rod clear altitude, then click Simulate
- **Optimization** (enabled via View menu): Select parameter, goal, range, and run optimizer

**Center — Results:**
- **Plot**: 2D charts for altitude, velocity, Mach number, and flight path
- **Dashboard**: Real-time telemetry gauges and flight summary
- **Data**: Events timeline and trajectory data table

### Workflow

1. **Open Design**: File > Open and select a `.ork` or `.rkt` file
2. **Select Motor**: Browse and select a motor from the Motor panel
3. **Configure**: Adjust simulation parameters (time step, wind, etc.)
4. **Simulate**: Click "▶ Simulate" and watch results appear
5. **Analyze**: Use Plot, Dashboard, and Data tabs to review results
6. **Optimize**: Enable the Optimization panel to find optimal design parameters

## File Formats

### OpenRocket (.ork)

The primary file format. Files are ZIP archives containing OpenRocketDocument XML. Federated Rocket supports reading and writing all standard component types:

- Body Tubes, Nose Cones, Transitions
- Fin Sets (trapezoidal and freeform)
- Parachutes, Streamers
- Mass Components, Bulkheads, Centering Rings
- Engine Blocks, Launch Lugs, Rail Buttons
- Inner Tubes, Tube Couplers, Sleeves
- Pods, Boosters, Payloads
- Engines, Component Assemblies

### RockSim (.rkt)

Legacy format support with pipe-delimited structured text. Federated Rocket can read and write common RockSim component types.

## Motor Database

### Embedded Motors

22 motors are built into the binary:

| Manufacturer | Motors |
|---|---|
| Estes | A8-3, B4-4, B6-4, C6-5, C6-7, D12-5, E12-4, F15-6 |
| Aerotech | E23-5, F24-7, G25-10, H13-15, I161-14, J250-15 |
| Quest | B4-4, C6-5, D16-6 |

### ThrustCurve.org Integration

```bash
# Search ThrustCurve API
federated-rocket motors --manufacturer "Cesaroni"
```

## Simulation Models

### Aerodynamics

- **Barrowman Method**: Standard subsonic aerodynamics for model rockets
- **7 Nose Cone Shapes**: Conical, Ogive, Elliptical, Parabolic, Power Series, Von Karman, Haack Series
- **Supersonic Corrections**: Prandtl-Glauert, Karman-Tsien, wave drag
- **Component Interference**: Fin-body, body-fin, staging gap effects
- **Enhanced Drag Model**: Base drag, skin friction (compressible), wave drag, induced drag, boat-tail drag

### Physics

- **Atmosphere**: 4-layer International Standard Atmosphere (ISA) model
- **Gravity**: Constant, Inverse Square, and WGS-84 ellipsoidal models
- **Wind**: Constant, Power Law, Logarithmic, and Gust models
- **6-DOF Integration**: RK4/AdaptiveRK4 with quaternion orientation

### Events Detected

- Launch, Launch Rod Clear
- Burntime Start, Burntime End
- Apogee (bisection-accurate)
- Recovery Device Deployment
- Ground Hit (bisection-accurate)
- Mach Transition, Max Velocity/Acceleration

## Validation

The validation suite ensures numerical accuracy:

```bash
# Run validation
cargo run -p federated-rocket-validation --bin validate

# Run all tests
cargo test --workspace
```

## Development

### Prerequisites

- Rust 1.80.0 or later
- Cargo

### Project Structure

```
federated-rocket/
├── crates/
│   ├── core/          # Component model, units, materials
│   ├── math/          # Vectors, matrices, quaternions, integrators
│   ├── physics/       # Atmosphere, gravity, wind models
│   ├── aero/          # Barrowman aerodynamics, drag models
│   ├── simulation/    # 6-DOF simulation engine
│   ├── motor-db/      # SQLite motor database, ThrustCurve API
│   ├── fileio/        # .ork, .rkt, CSV file formats
│   ├── optimization/  # Golden Section, Nelder-Mead optimization
│   ├── cli/           # Command-line interface
│   ├── gui/           # Desktop GUI (egui/eframe)
│   └── validation/    # Numerical validation framework
├── docs/              # Documentation
├── .github/           # CI/CD workflows
└── Cargo.toml         # Workspace manifest
```

### Building

```bash
cargo build --workspace          # Build all crates
cargo build --release            # Release build
cargo build -p federated-rocket-cli   # CLI only
cargo build -p federated-rocket-gui   # GUI only
```

### Testing

```bash
cargo test --workspace           # All tests
cargo test -p federated-rocket-simulation  # Specific crate
cargo test -- --test-threads=1   # Sequential (for accurate benchmarks)
```

## License

[Apache 2.0](LICENSE)
