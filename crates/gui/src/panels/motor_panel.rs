use crate::app::FederatedRocketApp;
use eframe::egui;

pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    ui.heading("Motor Selection");

    let motors = federated_rocket_motor_db::embedded::embedded_motors();

    // Filter by manufacturer
    let manufacturers: Vec<String> = {
        let mut mfrs: Vec<String> = motors.iter().map(|m| m.manufacturer.clone()).collect();
        mfrs.sort();
        mfrs.dedup();
        mfrs
    };

    // Manufacturer selector
    let mut selected_mfr = "All".to_string();
    egui::ComboBox::from_label("Manufacturer")
        .selected_text(&selected_mfr)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected_mfr, "All".to_string(), "All");
            for mfr in &manufacturers {
                ui.selectable_value(&mut selected_mfr, mfr.clone(), mfr);
            }
        });

    // Motor list
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 100.0)
        .show(ui, |ui| {
            for motor in &motors {
                // Apply manufacturer filter
                if selected_mfr != "All" && motor.manufacturer != selected_mfr {
                    continue;
                }

                let class = motor.impulse_class().display_name();
                let label = format!(
                    "{} {} [{}]",
                    motor.manufacturer_abbrev, motor.designation, class
                );

                if ui.selectable_label(false, &label).clicked() {
                    app.status_message = format!(
                        "Selected: {} {} - {}N\u{00B7}s, {:.1}s burn",
                        motor.manufacturer,
                        motor.designation,
                        motor.total_impulse,
                        motor.burn_time
                    );
                }
            }
        });

    ui.separator();

    // Motor details (shown when selected)
    ui.label("Click a motor to view details");
}