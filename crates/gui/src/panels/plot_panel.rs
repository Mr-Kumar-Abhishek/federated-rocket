use crate::app::FederatedRocketApp;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, PlotUi};
use federated_rocket_simulation::state::FlightState;

/// Plot panel showing trajectory data
pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    if let Some(ref result) = app.simulation_result {
        if result.trajectory.is_empty() {
            ui.label("No trajectory data");
            return;
        }

        // Plot selector tabs
        ui.horizontal(|ui| {
            ui.selectable_value(&mut app.plot_type, 0, "Altitude");
            ui.selectable_value(&mut app.plot_type, 1, "Velocity");
            ui.selectable_value(&mut app.plot_type, 2, "Mach");
            ui.selectable_value(&mut app.plot_type, 3, "Flight Path");
        });

        ui.separator();

        // The actual plot
        Plot::new("trajectory_plot")
            .height(300.0)
            .width(ui.available_width() - 10.0)
            .data_aspect(1.0)
            .show(ui, |plot_ui| match app.plot_type {
                0 => plot_altitude(plot_ui, &result.trajectory),
                1 => plot_velocity(plot_ui, &result.trajectory),
                2 => plot_mach(plot_ui, &result.trajectory),
                3 => plot_flight_path(plot_ui, &result.trajectory),
                _ => {}
            });
    } else {
        ui.label("Run a simulation to see plots");
    }
}

fn plot_altitude(plot_ui: &mut PlotUi, trajectory: &[FlightState]) {
    let points: PlotPoints = trajectory.iter().map(|s| [s.time, s.altitude()]).collect();
    let line = Line::new(points)
        .name("Altitude (m)")
        .color(egui::Color32::BLUE)
        .width(2.0);
    plot_ui.line(line);
}

fn plot_velocity(plot_ui: &mut PlotUi, trajectory: &[FlightState]) {
    let points: PlotPoints = trajectory.iter().map(|s| [s.time, s.speed()]).collect();
    let line = Line::new(points)
        .name("Velocity (m/s)")
        .color(egui::Color32::RED)
        .width(2.0);
    plot_ui.line(line);
}

fn plot_mach(plot_ui: &mut PlotUi, trajectory: &[FlightState]) {
    let points: PlotPoints = trajectory.iter().map(|s| [s.time, s.mach]).collect();
    let line = Line::new(points)
        .name("Mach")
        .color(egui::Color32::GREEN)
        .width(2.0);
    plot_ui.line(line);
}

fn plot_flight_path(plot_ui: &mut PlotUi, trajectory: &[FlightState]) {
    // 2D side view: altitude vs downrange distance
    let points: PlotPoints = trajectory
        .iter()
        .map(|s| [s.downrange_distance(), s.altitude()])
        .collect();
    let line = Line::new(points)
        .name("Flight Path")
        .color(egui::Color32::GOLD)
        .width(2.0);
    plot_ui.line(line);

    // Mark apogee if detected
    if let Some(apogee_state) = trajectory
        .iter()
        .max_by(|a, b| a.altitude().partial_cmp(&b.altitude()).unwrap())
    {
        plot_ui.points(
            egui_plot::Points::new(vec![[
                apogee_state.downrange_distance(),
                apogee_state.altitude(),
            ]])
            .color(egui::Color32::RED)
            .radius(5.0),
        );
    }
}
