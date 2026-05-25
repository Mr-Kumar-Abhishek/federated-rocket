/// CLI binary for running validation tests
/// Usage: cargo run -p federated-rocket-validation --bin validate
use federated_rocket_aero::compute::AeroCalculator;
use federated_rocket_physics::atmosphere::StandardAtmosphere;
use federated_rocket_physics::gravity::ConstantGravity;
use federated_rocket_physics::wind::NoWind;
use federated_rocket_simulation::engine::{AdaptiveSimulationEngine, SimulationConfig};
use federated_rocket_simulation::events::EventConfig;
use federated_rocket_simulation::state::FlightState;
use federated_rocket_validation::test_cases;

fn main() {
    println!("=== Federated Rocket Validation Suite ===\n");

    // Run all standard test cases
    let test_cases = vec![
        test_cases::test_case_simple_rocket(),
        test_cases::test_case_hpr_rocket(),
        test_cases::test_case_min_diameter(),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for case in &test_cases {
        println!("Test: {} - {}", case.name, case.description);
        println!("  Building rocket...");
        let tree = (case.build_tree)();

        println!("  Running simulation...");
        // Run the simulation
        let config = SimulationConfig {
            time_step: 0.001,
            reference_area: std::f64::consts::PI * 0.01, // placeholder
            reference_diameter: 0.02,                     // placeholder
            max_time: 120.0,
            min_time_step: Some(1e-6),
            max_time_step: Some(0.01),
            adaptive_tolerance: Some(1e-7),
            use_adaptive_stepping: true,
        };
        let event_config = EventConfig::default();
        let aero_calc = AeroCalculator::new();
        let engine = AdaptiveSimulationEngine::new(config, event_config, aero_calc);

        let atmosphere = StandardAtmosphere;
        let gravity = ConstantGravity;
        let wind = NoWind;
        let initial = FlightState::new();

        let result = engine.simulate(initial, None, &tree, &atmosphere, &gravity, &wind);

        println!("  Max altitude: {:.1}m", result.max_altitude);
        println!("  Max velocity: {:.1}m/s", result.max_velocity);
        println!("  Flight time: {:.1}s", result.flight_time);
        println!("  Events: {}", result.events.len());

        // Check basic criteria
        if result.max_altitude > 10.0 && result.flight_time > 2.0 {
            println!("  ✅ PASSED\n");
            passed += 1;
        } else {
            println!("  ❌ FAILED (insufficient flight)\n");
            failed += 1;
        }
    }

    println!(
        "=== Results: {}/{} passed, {} failed ===",
        passed,
        test_cases.len(),
        failed
    );
}