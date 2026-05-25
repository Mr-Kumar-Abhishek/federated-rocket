use crate::app::FederatedRocketApp;
use eframe::egui;

pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    if let Some(ref result) = app.simulation_result {
        // Events timeline
        ui.heading("Flight Events");
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for event in &result.events {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:.2}s", event.time));
                        ui.label(&event.description);
                        if event.mach > 0.0 {
                            ui.label(format!("M{:.2}", event.mach));
                        }
                    });
                }
            });

        ui.separator();

        // Trajectory data table
        ui.heading("Trajectory Data");
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                egui::Grid::new("trajectory_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Time");
                        ui.strong("Alt (m)");
                        ui.strong("Vel (m/s)");
                        ui.strong("Mach");
                        ui.strong("AoA (\u{00B0})");
                        ui.strong("Q (kPa)");
                        ui.end_row();

                        for state in &result.trajectory {
                            ui.label(format!("{:.2}", state.time));
                            ui.label(format!("{:.1}", state.altitude()));
                            ui.label(format!("{:.1}", state.speed()));
                            ui.label(format!("{:.2}", state.mach));
                            ui.label(format!("{:.1}", state.angle_of_attack.to_degrees()));
                            ui.label(format!("{:.1}", state.dynamic_pressure / 1000.0));
                            ui.end_row();
                        }
                    });
            });
    } else {
        ui.heading("No Results");
        ui.label("Run a simulation to see results here.");
    }
}