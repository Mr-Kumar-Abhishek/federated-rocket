// Place in crates/validation/tests/benchmarks.rs
// Run with: cargo bench

#[cfg(test)]
mod benchmarks {
    // These are manual timing benchmarks that can be run
    // For proper criterion benchmarks, add criterion dependency
    
    #[test]
    fn benchmark_simulation_speed() {
        // Build a standard rocket
        let tree = federated_rocket_validation::test_cases::build_simple_rocket();
        let atmosphere = federated_rocket_physics::atmosphere::StandardAtmosphere;
        let gravity = federated_rocket_physics::gravity::ConstantGravity;
        let wind = federated_rocket_physics::wind::NoWind;
        let aero_calc = federated_rocket_aero::compute::AeroCalculator::new();
        
        let config = federated_rocket_simulation::engine::SimulationConfig {
            time_step: 0.001,
            output_interval: 0.1,
            reference_area: std::f64::consts::PI * 0.009525 * 0.009525,
            reference_diameter: 0.01905,
            max_time: 120.0,
            min_time_step: Some(1e-6),
            max_time_step: Some(0.01),
            adaptive_tolerance: Some(1e-7),
            use_adaptive_stepping: true,
        };
        let event_config = federated_rocket_simulation::events::EventConfig {
            launch_rod_clear_altitude: 1.0,
            max_simulation_time: 120.0,
            output_interval: Some(0.1),
            ..Default::default()
        };
        let engine = federated_rocket_simulation::engine::AdaptiveSimulationEngine::new(
            config, event_config, aero_calc
        );
        
        // Time the simulation
        use std::time::Instant;
        let start = Instant::now();
        
        const NUM_RUNS: u32 = 5;
        for _ in 0..NUM_RUNS {
            let initial = federated_rocket_simulation::state::FlightState::new();
            let result = engine.simulate(initial.clone(), None, &tree, &atmosphere, &gravity, &wind);
            assert!(result.max_altitude > 0.0);
        }
        
        let elapsed = start.elapsed();
        let avg = elapsed / NUM_RUNS;
        
        println!("Average simulation time: {:?}", avg);
        println!("Real-time ratio: {:.2}x", avg.as_secs_f64() / 10.0);
        
        // Performance requirement: simulation should be faster than real-time
        // For a 10-second flight, simulation should take less than 10 seconds
        assert!(avg.as_secs_f64() < 10.0, 
            "Simulation must run faster than real-time: {:?}", avg);
    }
    
    #[test]
    fn benchmark_aero_calculation() {
        use std::time::Instant;
        
        let tree = federated_rocket_validation::test_cases::build_simple_rocket();
        let aero_calc = federated_rocket_aero::compute::AeroCalculator::new();
        let velocity = federated_rocket_math::vector::Vector3D::new(100.0, 0.0, 0.0);
        let ang_vel = federated_rocket_math::vector::Vector3D::zero();
        let atmosphere = federated_rocket_physics::atmosphere::AtmosphericConditions {
            altitude: 100.0,
            temperature: 288.15,
            pressure: 101325.0,
            density: 1.225,
            speed_of_sound: 340.294,
            viscosity: 1.7894e-5,
        };
        
        const NUM_CALCS: u32 = 10000;
        let start = Instant::now();
        
        for _ in 0..NUM_CALCS {
            let _ = aero_calc.compute_forces(
                &tree, velocity, ang_vel, &atmosphere,
                std::f64::consts::PI * 0.009525 * 0.009525,
                0.01905,
            );
        }
        
        let elapsed = start.elapsed();
        let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / NUM_CALCS as f64;
        
        println!("Average aero calculation: {:.1} µs", avg_us);
        
        // Aero calculations should each take < 100µs
        assert!(avg_us < 100.0, 
            "Aero calculation too slow: {:.1} µs", avg_us);
    }
    
    #[test]
    fn benchmark_file_io() {
        use std::time::Instant;
        use std::io::Write;
        
        // Generate a minimal .ork file
        let ork_content = r#"<?xml version='1.0' encoding='utf-8'?>
<OpenRocketDocument>
  <Version>1.6</Version>
  <Name>Benchmark Rocket</Name>
  <Subcomponents>
    <BodyTube>
      <Name>Body</Name>
      <Length>30.0</Length>
      <OuterRadius>0.5</OuterRadius>
      <InnerRadius>0.475</InnerRadius>
      <Material type="bulk" name="Cardboard">0.5</Material>
    </BodyTube>
  </Subcomponents>
</OpenRocketDocument>"#;
        
        let dir = std::env::temp_dir().join("federated_rocket_bench");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bench.ork");
        
        let start = Instant::now();
        
        const NUM_IO: u32 = 100;
        for _ in 0..NUM_IO {
            // Write
            std::fs::write(&path, ork_content).unwrap();
            // Read back
            let _content = std::fs::read_to_string(&path).unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_secs_f64() * 1000.0 / NUM_IO as f64;
        
        println!("Average file I/O: {:.2} ms", avg_ms);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}