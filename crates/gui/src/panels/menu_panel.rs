use crate::app::FederatedRocketApp;
use eframe::egui;
use std::path::Path;

pub fn show_menu_bar(app: &mut FederatedRocketApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    app.show_file_open_dialog = true;
                    app.dialog_path = app.current_file.clone().unwrap_or_default();
                    ui.close_menu();
                }

                if ui.button("Save").clicked() {
                    if let Some(ref path) = app.current_file.clone() {
                        if let Some(ref tree) = app.component_tree {
                            let p = Path::new(path);
                            match federated_rocket_fileio::ork::OpenRocketFile::save(p, tree) {
                                Ok(()) => {
                                    app.status_message = format!("Saved: {}", path);
                                    app.has_unsaved_changes = false;
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("Failed to save: {}", e));
                                }
                            }
                        }
                    } else {
                        app.status_message = "No file to save. Use Save As...".to_string();
                    }
                    ui.close_menu();
                }

                if ui.button("Save As...").clicked() {
                    app.show_file_save_as_dialog = true;
                    app.dialog_path = app.current_file.clone().unwrap_or_default();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut app.show_design_panel, "Design Panel");
                ui.checkbox(&mut app.show_simulation_panel, "Simulation Panel");
                ui.checkbox(&mut app.show_results_panel, "Results Panel");
                ui.checkbox(&mut app.show_motor_panel, "Motor Panel");
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    app.status_message =
                        "Federated Rocket v0.1.0 - Model Rocket Simulation Software".to_string();
                    ui.close_menu();
                }
            });
        });
    });
}