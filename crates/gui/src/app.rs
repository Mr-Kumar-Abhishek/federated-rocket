use eframe::egui;
use federated_rocket_core::component_tree::ComponentKey;
use federated_rocket_core::component_tree::ComponentTree;
use federated_rocket_simulation::engine::SimulationResult;

use crate::panels::{
    dashboard_panel, design_panel, menu_panel, motor_panel, optimization_panel, plot_panel,
    results_panel, simulation_panel,
};

/// Main application state
pub struct FederatedRocketApp {
    // File state
    pub current_file: Option<String>,
    pub component_tree: Option<ComponentTree>,
    pub has_unsaved_changes: bool,

    // Panel visibility
    pub show_design_panel: bool,
    pub show_simulation_panel: bool,
    pub show_results_panel: bool,
    pub show_motor_panel: bool,

    // Simulation state
    pub simulation_result: Option<SimulationResult>,
    pub is_simulating: bool,
    pub simulation_progress: f32,

    // Messages
    pub status_message: String,
    pub error_message: Option<String>,

    // Selected items
    pub selected_component: Option<ComponentKey>,

    // File dialog state
    pub show_file_open_dialog: bool,
    pub show_file_save_as_dialog: bool,
    pub dialog_path: String,

    // New: Plot state
    pub plot_type: usize, // 0=altitude, 1=velocity, 2=mach, 3=flight_path

    // New: Optimization state
    pub optimization_param_idx: usize,
    pub optimization_goal_idx: usize,
    pub optimization_min: f64,
    pub optimization_max: f64,
    pub is_optimizing: bool,
    pub optimization_result: Option<federated_rocket_optimization::types::OptimizationResult>,

    // New: Panel visibility
    pub show_plot_panel: bool,
    pub show_dashboard_panel: bool,
    pub show_optimization_panel: bool,
}

impl FederatedRocketApp {
    pub fn new() -> Self {
        Self {
            current_file: None,
            component_tree: None,
            has_unsaved_changes: false,
            show_design_panel: true,
            show_simulation_panel: true,
            show_results_panel: true,
            show_motor_panel: false,
            simulation_result: None,
            is_simulating: false,
            simulation_progress: 0.0,
            status_message: "Welcome to Federated Rocket".to_string(),
            error_message: None,
            selected_component: None,
            show_file_open_dialog: false,
            show_file_save_as_dialog: false,
            dialog_path: String::new(),

            // Plot
            plot_type: 0,

            // Optimization
            optimization_param_idx: 0,
            optimization_goal_idx: 0,
            optimization_min: 0.0,
            optimization_max: 50.0,
            is_optimizing: false,
            optimization_result: None,

            // Panel visibility
            show_plot_panel: true,
            show_dashboard_panel: true,
            show_optimization_panel: false,
        }
    }
}

impl eframe::App for FederatedRocketApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        menu_panel::show_menu_bar(self, ctx);

        // Main layout: left sidebar + center + right sidebar
        egui::SidePanel::left("design_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Design");
                ui.separator();
                design_panel::show(self, ui);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(350.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Controls");
                    ui.separator();
                    motor_panel::show(self, ui);
                    ui.separator();
                    simulation_panel::show(self, ui);

                    if self.show_optimization_panel {
                        ui.separator();
                        optimization_panel::show(self, ui);
                    }
                });
            });

        // Center: Results with tabs for Plots, Dashboard, Data
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Results");
                ui.separator();
                ui.selectable_value(&mut self.show_plot_panel, true, "📈 Plot");
                ui.selectable_value(&mut self.show_dashboard_panel, true, "📊 Dashboard");
                ui.selectable_value(&mut self.show_results_panel, true, "📋 Data");
            });
            ui.separator();

            if self.show_plot_panel {
                plot_panel::show(self, ui);
            } else if self.show_dashboard_panel {
                dashboard_panel::show(self, ui);
            } else if self.show_results_panel {
                results_panel::show(self, ui);
            }
        });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(20.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);
                    if let Some(ref err) = self.error_message {
                        ui.colored_label(egui::Color32::RED, err);
                    }
                });
            });

        // Error popup
        if let Some(ref err) = self.error_message.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, err);
                    if ui.button("Close").clicked() {
                        self.error_message = None;
                    }
                });
        }

        // File open dialog
        if self.show_file_open_dialog {
            egui::Window::new("Open Rocket File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter file path:");
                    ui.text_edit_singleline(&mut self.dialog_path);
                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() {
                            let path = self.dialog_path.trim().to_string();
                            if !path.is_empty() {
                                let p = std::path::Path::new(&path);
                                if p.exists() {
                                    match federated_rocket_fileio::format_detect::load_rocket_file(
                                        p,
                                    ) {
                                        Ok(tree) => {
                                            self.component_tree = Some(tree);
                                            self.current_file = Some(path.clone());
                                            self.status_message = format!("Loaded: {}", path);
                                            self.has_unsaved_changes = false;
                                            self.error_message = None;
                                        }
                                        Err(e) => {
                                            self.error_message =
                                                Some(format!("Failed to load: {}", e));
                                        }
                                    }
                                } else {
                                    self.error_message = Some(format!("File not found: {}", path));
                                }
                            }
                            self.show_file_open_dialog = false;
                            self.dialog_path.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_file_open_dialog = false;
                            self.dialog_path.clear();
                        }
                    });
                });
        }

        // File save-as dialog
        if self.show_file_save_as_dialog {
            egui::Window::new("Save Rocket File As")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter file path:");
                    ui.text_edit_singleline(&mut self.dialog_path);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let path = self.dialog_path.trim().to_string();
                            if !path.is_empty() {
                                let p = std::path::Path::new(&path);
                                if let Some(ref tree) = self.component_tree {
                                    match federated_rocket_fileio::ork::OpenRocketFile::save(
                                        p, tree,
                                    ) {
                                        Ok(()) => {
                                            self.current_file = Some(path.clone());
                                            self.status_message = format!("Saved: {}", path);
                                            self.has_unsaved_changes = false;
                                            self.error_message = None;
                                        }
                                        Err(e) => {
                                            self.error_message =
                                                Some(format!("Failed to save: {}", e));
                                        }
                                    }
                                }
                            }
                            self.show_file_save_as_dialog = false;
                            self.dialog_path.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_file_save_as_dialog = false;
                            self.dialog_path.clear();
                        }
                    });
                });
        }
    }
}
