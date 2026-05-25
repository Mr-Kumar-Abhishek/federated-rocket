use eframe::egui;
use crate::app::FederatedRocketApp;

/// Real-time telemetry dashboard with gauges
pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    if let Some(ref result) = app.simulation_result {
        // Get latest state
        if let Some(latest) = result.trajectory.last() {
            ui.heading("Telemetry");
            ui.separator();
            
            // Gauges in a 2x3 grid
            egui::Grid::new("telemetry_grid")
                .min_col_width(120.0)
                .max_col_width(200.0)
                .show(ui, |ui| {
                    // Row 1: Altitude & Velocity
                    gauge_widget(ui, "Altitude", latest.altitude(), 0.0, result.max_altitude.max(1.0), "m", egui::Color32::BLUE);
                    gauge_widget(ui, "Velocity", latest.speed(), 0.0, result.max_velocity.max(1.0), "m/s", egui::Color32::RED);
                    ui.end_row();
                    
                    // Row 2: Mach & AoA
                    gauge_widget(ui, "Mach", latest.mach, 0.0, 3.0, "", egui::Color32::GREEN);
                    gauge_widget(ui, "Angle of Attack", latest.angle_of_attack.to_degrees(), 0.0, 45.0, "°", egui::Color32::YELLOW);
                    ui.end_row();
                    
                    // Row 3: Dynamic Pressure & Time
                    gauge_widget(ui, "Dynamic Pressure", latest.dynamic_pressure / 1000.0, 0.0, 100.0, "kPa", egui::Color32::ORANGE);
                    gauge_widget(ui, "Flight Time", latest.time, 0.0, result.flight_time.max(1.0), "s", egui::Color32::WHITE);
                    ui.end_row();
                });
            
            ui.separator();
            
            // Critical values display
            ui.heading("Flight Summary");
            ui.label(format!("🏔 Max Altitude: {:.1}m ({:.1}ft)", result.max_altitude, result.max_altitude * 3.28084));
            ui.label(format!("🚀 Max Velocity: {:.1}m/s ({:.1}mph)", result.max_velocity, result.max_velocity * 2.23694));
            ui.label(format!("💥 Max Accel: {:.1}m/s² ({:.1}G)", result.max_acceleration, result.max_acceleration / 9.80665));
            ui.label(format!("⏱ Flight Time: {:.1}s", result.flight_time));
        }
    } else {
        ui.heading("Telemetry");
        ui.separator();
        ui.label("No data available");
        ui.label("Run a simulation to see telemetry");
    }
}

/// A simple gauge widget showing a value as a progress bar
fn gauge_widget(ui: &mut egui::Ui, label: &str, value: f64, min: f64, max: f64, unit: &str, color: egui::Color32) {
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);
    
    ui.vertical(|ui| {
        ui.label(format!("{}: {:.2}{}", label, value, unit));
        let progress = normalized as f32;
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(150.0)
                .fill(color)
        );
    });
}