# Changelog

All notable changes to the Federated Rocket project will be documented in this file.

## [0.1.0] - 2026-05-25

### Added

#### Foundation Layer
- **Core Crate** (`federated-rocket-core`): Units system with 47 variants across 12 categories (Length, Mass, Time, Velocity, Acceleration, Angle, Force, Pressure, Temperature, Area, Volume, Impulse). `Quantity<T>` generic for type-safe unit conversions. `Coordinate` struct with full 3D vector operations (distance, angle, rotation via Rodrigues' formula). `Material` system with 28 bulk materials and 8 surface materials with realistic density/thickness constants.
- **Core Components**: `RocketComponent` tagged enum with 22 variants replacing Java class hierarchy. `ComponentTree` using `slotmap::SlotMap` for O(1) key-based lookups without `Rc<RefCell<>>` overhead. `ComponentKey` newtype for type-safe handles.
- **Math Crate** (`federated-rocket-math`): `Vector3D` with full arithmetic and rotation operations. `Matrix3x3` with determinant, inverse, and rotation matrix generation. `Quaternion` with SLERP, Euler angle conversion, and axis-angle representation. Generic `Integrator<State>` trait with `EulerIntegrator`, `RK4Integrator`, `RK6Integrator`, and `AdaptiveRK4Integrator` (Richardson extrapolation). `Interpolator` with Linear, CubicSpline, Akima, and Polynomial methods.

#### Domain Layer
- **Physics Crate** (`federated-rocket-physics`): 14 physical constants. `AtmosphericModel` trait with `StandardAtmosphere` (4-layer ISA), `IsothermalAtmosphere`, and `ExtremeTemperatureAtmosphere`. `GravityModel` trait with `ConstantGravity`, `InverseSquareGravity`, and `Wgs84Gravity`. `WindModel` trait with `NoWind`, `ConstantWind`, `PowerLawWind`, `LogarithmicWind`, and `WindGust`.
- **Aero Crate** (`federated-rocket-aero`): `BarrowmanCalculator` for nose cone CNα/CP (7 shapes), transition aerodynamics, fin set calculations (`4·N·(s/d)² / (1+√(1+(2Lc/(cr+ct))²))`), total rocket CP, drag components (base, skin friction, pressure, fin, interference), pitch damping. `SupersonicCorrections` with Prandtl-Glauert factor, Karman-Tsien factor, wave drag, transonic blend. `AeroCalculator` orchestrator.
- **Advanced Aero**: `InterferenceFactors` with fin-body (Barrowman), body-fin (Barnwell-Sewell), staging gap, pod interference. `RingFinAero` for tube fin aerodynamics. `BodyFlapAero` for control surfaces and canards. Enhanced Mach-dependent drag model with compressible skin friction, wave drag for 5 nose shapes, boat-tail drag, and staging gap drag.
- **Simulation Crate** (`federated-rocket-simulation`): `FlightState` with 6-DOF state (position, velocity, quaternion orientation, angular velocity, mass, inertia). 18 `FlightEventType` variants (Launch, RodClear, BurntimeStart/End, Apogee, Recovery, GroundHit, MachTransition, etc.). `MotorModel` with thrust curve interpolation and CSV loading.
- **Simulation Engine**: `SimulationEngine` with fixed-step RK4 integration. `AdaptiveSimulationEngine` with adaptive step sizing and Richardson error estimation. Event bisection search for exact apogee/burnout/ground-hit times. Self-consistency validation across step sizes.
- **Motor DB Crate** (`federated-rocket-motor-db`): `Motor` type with impulse classification. `MotorDatabase` using SQLite (rusqlite bundled) with full CRUD, search, and CSV import. `ThrustCurveApi` client for ThrustCurve.org integration. 22 embedded motors (Estes, Aerotech, Quest). `MotorCache` thread-safe in-memory cache.
- **File I/O Crate** (`federated-rocket-fileio`): `OpenRocketFile` reader/writer for .ork format (ZIP+XML). `RockSimFile` parser for .rkt format (pipe-delimited). `CsvExport` for trajectory, events, and motor curve export. Auto-format detection by file extension.

#### Application Layer
- **CLI Crate** (`federated-rocket-cli`): 5 subcommands: `simulate` (full 6-DOF simulation with configurable parameters, JSON/CSV output), `info` (rocket design introspection with tree view), `motors` (motor database query with manufacturer/class filtering), `optimize` (1D Golden Section parameter optimization), `convert` (inter-format file conversion).
- **GUI Crate** (`federated-rocket-gui`): egui/eframe desktop application with 8 panels: Menu (File/View/Help), Design (component tree with icons and properties), Motor (browser with manufacturer filter), Simulation (configurable sliders + Run button), Results (events timeline + trajectory data table), Plot (2D altitude/velocity/mach/flight-path charts), Dashboard (telemetry gauges with flight summary), Optimization (parameter/goal selection + convergence history).
- **Validation Crate** (`federated-rocket-validation`): `ReferenceSimulation` data structures for OpenRocket comparison. `SimulationComparator` with configurable tolerances (0.1% altitude/velocity, 0.01 Mach). 3 standard test cases (simple 3FNC, HPR 54mm, min-diameter 29mm). CLI validation runner.

#### Quality Assurance
- **CI/CD**: 6 GitHub Actions workflows: multi-platform CI (linux/windows/macos), code coverage (Codecov), performance benchmarks (criterion), release automation (tag-triggered with binary artifacts), numerical validation (weekly OpenRocket comparison), dependency updates (dependabot weekly/monthly).
- **Testing**: 449+ unit tests across all crates. 143 aero tests, 84 math tests, 82 core tests, 64 physics tests, 63 simulation tests, 45 optimization tests, 36 motor-db tests, 23 fileio tests, 6 CLI tests, 5 validation integration tests.
- **Performance**: Sub-100µs aero calculations. Real-time simulation capability. 7.1 MB CLI binary, 15.1 MB GUI binary.

### Architecture

```
10 crates across 3 layers:
┌─────────────────────────────────────────────┐
│  Application: cli, gui                      │
├─────────────────────────────────────────────┤
│  Domain: physics, aero, simulation,         │
│          fileio, motor-db, optimization      │
├─────────────────────────────────────────────┤
│  Foundation: core, math, validation          │
└─────────────────────────────────────────────┘
```
