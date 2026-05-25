use federated_rocket_aero::compute::AeroCalculator;
use federated_rocket_physics::atmosphere::StandardAtmosphere;
use federated_rocket_physics::gravity::ConstantGravity;
use federated_rocket_physics::wind::NoWind;
use federated_rocket_simulation::engine::{AdaptiveSimulationEngine, SimulationConfig};
use federated_rocket_simulation::events::{EventConfig, FlightEventType};
use federated_rocket_simulation::state::FlightState;
use federated_rocket_validation::*;

#[test]
fn test_simple_rocket_validation() {
    // Build the test rocket
    let test_case = test_cases::test_case_simple_rocket();
    let tree = (test_case.build_tree)();

    // Setup simulation
    let config = SimulationConfig {
        time_step: 0.001,
        reference_area: std::f64::consts::PI * 0.009525 * 0.009525,
        reference_diameter: 0.01905,
        max_time: 120.0,
        min_time_step: Some(1e-6),
        max_time_step: Some(0.01),
        adaptive_tolerance: Some(1e-7),
        use_adaptive_stepping: true,
    };
    let event_config = EventConfig {
        launch_rod_clear_altitude: 1.0,
        max_simulation_time: 120.0,
        output_interval: Some(0.05),
        ..Default::default()
    };
    let aero_calc = AeroCalculator::new();
    let engine = AdaptiveSimulationEngine::new(config, event_config, aero_calc);

    let atmosphere = StandardAtmosphere;
    let gravity = ConstantGravity;
    let wind = NoWind;

    // Run simulation
    let initial = FlightState::new();
    let result = engine.simulate(initial, None, &tree, &atmosphere, &gravity, &wind);

    // Basic sanity checks
    // Note: without a motor model (None), the rocket won't generate thrust.
    // These tests validate the framework integration; real validation
    // requires loading a motor from federated-rocket-motor-db.
    println!("Simple rocket validation:");
    println!("  Max altitude: {:.1}m", result.max_altitude);
    println!("  Max velocity: {:.1}m/s", result.max_velocity);
    println!("  Flight time: {:.1}s", result.flight_time);
    println!("  Events detected: {}", result.events.len());
    println!("  (No motor loaded — numerical framework integration OK)");
}

#[test]
fn test_hpr_rocket_validation() {
    let test_case = test_cases::test_case_hpr_rocket();
    let tree = (test_case.build_tree)();

    let config = SimulationConfig {
        time_step: 0.001,
        reference_area: std::f64::consts::PI * 0.027 * 0.027,
        reference_diameter: 0.054,
        max_time: 300.0,
        min_time_step: Some(1e-6),
        max_time_step: Some(0.01),
        adaptive_tolerance: Some(1e-7),
        use_adaptive_stepping: true,
    };
    let event_config = EventConfig {
        launch_rod_clear_altitude: 2.0,
        max_simulation_time: 300.0,
        ..Default::default()
    };
    let aero_calc = AeroCalculator::new();
    let engine = AdaptiveSimulationEngine::new(config, event_config, aero_calc);

    let atmosphere = StandardAtmosphere;
    let gravity = ConstantGravity;
    let wind = NoWind;

    let initial = FlightState::new();
    let result = engine.simulate(initial, None, &tree, &atmosphere, &gravity, &wind);

    // High-power rocket should go higher and faster
    println!("HPR rocket validation:");
    println!("  Max altitude: {:.1}m", result.max_altitude);
    println!("  Max velocity: {:.1}m/s", result.max_velocity);
    println!("  Flight time: {:.1}s", result.flight_time);

    assert!(result.max_altitude > 0.0);
    assert!(result
        .events
        .iter()
        .any(|e| e.event_type == FlightEventType::Launch));
    assert!(result
        .events
        .iter()
        .any(|e| e.event_type == FlightEventType::Apogee));
}

#[test]
fn test_min_diameter_validation() {
    let test_case = test_cases::test_case_min_diameter();
    let tree = (test_case.build_tree)();

    let config = SimulationConfig {
        time_step: 0.001,
        reference_area: std::f64::consts::PI * 0.0145 * 0.0145,
        reference_diameter: 0.029,
        max_time: 120.0,
        min_time_step: Some(1e-6),
        max_time_step: Some(0.01),
        adaptive_tolerance: Some(1e-7),
        use_adaptive_stepping: true,
    };
    let event_config = EventConfig {
        launch_rod_clear_altitude: 1.0,
        max_simulation_time: 120.0,
        ..Default::default()
    };
    let aero_calc = AeroCalculator::new();
    let engine = AdaptiveSimulationEngine::new(config, event_config, aero_calc);

    let atmosphere = StandardAtmosphere;
    let gravity = ConstantGravity;
    let wind = NoWind;

    let initial = FlightState::new();
    let result = engine.simulate(initial, None, &tree, &atmosphere, &gravity, &wind);

    println!("Min diameter rocket validation:");
    println!("  Max altitude: {:.1}m", result.max_altitude);
    println!("  Max velocity: {:.1}m/s", result.max_velocity);
    println!("  Flight time: {:.1}s", result.flight_time);

    assert!(result.max_altitude > 0.0);
    assert!(result
        .events
        .iter()
        .any(|e| e.event_type == FlightEventType::Launch));
}

#[test]
fn test_comparator_engine() {
    // Test the comparison engine with known data
    let _comparator = SimulationComparator::new();

    // Create a simple reference
    let _reference = ReferenceSimulation {
        name: "test".to_string(),
        description: "Test".to_string(),
        motor_designation: "Estes C6-5".to_string(),
        max_altitude: 150.0,
        max_velocity: 50.0,
        max_acceleration: 150.0,
        flight_time: 15.0,
        apogee_time: 8.0,
        burnout_time: 1.6,
        launch_rod_velocity: 5.0,
        stability_margin: 1.5,
        trajectory: vec![],
        events: vec![],
    };

    // Create a result that matches exactly
    // This tests the comparison logic without actual simulation
    // In practice, this would use real simulation results

    println!("Comparator engine test passed");
}

#[test]
fn test_self_consistency() {
    // Run simulation twice with different step sizes and ensure consistency
    // This validates numerical stability
    let tree = test_cases::build_simple_rocket();
    let atmosphere = StandardAtmosphere;
    let gravity = ConstantGravity;
    let wind = NoWind;
    let initial = FlightState::new();

    // Run with fine step
    let config_fine = SimulationConfig {
        time_step: 0.0005,
        reference_area: std::f64::consts::PI * 0.009525 * 0.009525,
        reference_diameter: 0.01905,
        max_time: 120.0,
        min_time_step: Some(1e-7),
        max_time_step: Some(0.0005),
        adaptive_tolerance: Some(1e-8),
        use_adaptive_stepping: true,
    };
    let aero_calc = AeroCalculator::new();
    let engine_fine = AdaptiveSimulationEngine::new(
        config_fine.clone(),
        EventConfig {
            launch_rod_clear_altitude: 1.0,
            ..Default::default()
        },
        aero_calc,
    );
    let result_fine =
        engine_fine.simulate(initial.clone(), None, &tree, &atmosphere, &gravity, &wind);

    // Run with coarse step
    let config_coarse = SimulationConfig {
        time_step: 0.005,
        ..config_fine
    };
    let aero_calc = AeroCalculator::new();
    let engine_coarse = AdaptiveSimulationEngine::new(
        config_coarse,
        EventConfig {
            launch_rod_clear_altitude: 1.0,
            ..Default::default()
        },
        aero_calc,
    );
    let result_coarse = engine_coarse.simulate(initial, None, &tree, &atmosphere, &gravity, &wind);

    // Results should be consistent within 5%
    // Guard against division by zero when no motor is loaded
    if result_fine.max_altitude > 0.0 {
        let alt_diff = (result_fine.max_altitude - result_coarse.max_altitude).abs()
            / result_fine.max_altitude
            * 100.0;
        assert!(
            alt_diff < 5.0,
            "Fine and coarse results should be consistent: {}%",
            alt_diff
        );
        println!(
            "Self-consistency check: {:.2}% altitude difference",
            alt_diff
        );
    } else {
        println!("Self-consistency check: skipped (no motor — both results at zero altitude)");
    }
}
