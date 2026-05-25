use crate::app::FederatedRocketApp;
use eframe::egui;

pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    ui.heading("Simulation");

    // Simulation parameters (local state for the panel)
    let mut max_time = 120.0_f64;
    let mut time_step = 0.001_f64;
    let mut wind_speed = 0.0_f64;
    let mut wind_dir = 0.0_f64;
    let mut rod_clear = 2.0_f64;

    ui.add(egui::Slider::new(&mut max_time, 10.0..=300.0).text("Max Time (s)"));
    ui.add(egui::Slider::new(&mut time_step, 0.0001..=0.01).text("Time Step (s)"));
    ui.add(egui::Slider::new(&mut wind_speed, 0.0..=30.0).text("Wind Speed (m/s)"));
    ui.add(egui::Slider::new(&mut wind_dir, 0.0..=360.0).text("Wind Direction (°)"));
    ui.add(egui::Slider::new(&mut rod_clear, 0.5..=10.0).text("Rod Clear (m)"));

    ui.separator();

    // Simulate button
    let can_simulate = app.component_tree.is_some() && !app.is_simulating;

    if ui
        .add_enabled(can_simulate, egui::Button::new("\u{25B6} Simulate"))
        .clicked()
    {
        if let Some(ref tree) = app.component_tree {
            app.is_simulating = true;
            app.status_message = "Running simulation...".to_string();

            // Setup models
            let atmosphere = federated_rocket_physics::atmosphere::StandardAtmosphere;
            let gravity = federated_rocket_physics::gravity::ConstantGravity;

            let config = federated_rocket_simulation::engine::SimulationConfig {
                time_step,
                reference_area: std::f64::consts::PI * 0.0254 * 0.0254,
                reference_diameter: 0.0508,
            };

            let event_config = federated_rocket_simulation::events::EventConfig {
                launch_rod_clear_altitude: rod_clear,
                max_simulation_time: max_time,
                ground_altitude: 0.0,
                output_interval: 0.1,
                ..Default::default()
            };

            let engine =
                federated_rocket_simulation::engine::SimulationEngine::new(config, event_config);
            let initial_state = federated_rocket_simulation::state::FlightState::new();

            let result = if wind_speed > 0.0 {
                let wind = federated_rocket_physics::wind::ConstantWind::new(wind_speed, wind_dir);
                engine.simulate(
                    initial_state,
                    None, // motor
                    tree,
                    &atmosphere,
                    &gravity,
                    &wind,
                )
            } else {
                let wind = federated_rocket_physics::wind::NoWind;
                engine.simulate(
                    initial_state,
                    None, // motor
                    tree,
                    &atmosphere,
                    &gravity,
                    &wind,
                )
            };

            app.simulation_result = Some(result);
            app.is_simulating = false;
            app.status_message = "Simulation complete".to_string();
        }
    }

    // Progress indicator
    if app.is_simulating {
        ui.add(
            egui::ProgressBar::new(app.simulation_progress).text("Simulating..."),
        );
    }

    ui.separator();

    // Quick stats
    if let Some(ref result) = app.simulation_result {
        ui.heading("Quick Stats");
        ui.label(format!(
            "Flight time: {:.2}s",
            result.flight_time
        ));
        ui.label(format!(
            "Max altitude: {:.1}m ({:.1}ft)",
            result.max_altitude,
            result.max_altitude * 3.28084
        ));
        ui.label(format!(
            "Max velocity: {:.1}m/s ({:.1}mph)",
            result.max_velocity,
            result.max_velocity * 2.23694
        ));
        ui.label(format!(
            "Max acceleration: {:.1}m/s\u{00B2} ({:.1}G)",
            result.max_acceleration,
            result.max_acceleration / 9.80665
        ));
    }
}