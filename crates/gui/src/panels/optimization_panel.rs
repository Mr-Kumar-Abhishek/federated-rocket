use eframe::egui;
use crate::app::FederatedRocketApp;
use federated_rocket_optimization::golden_section::GoldenSectionSearch;

/// Optimization control panel
pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    ui.heading("Optimization");
    ui.separator();
    
    // Parameter selection
    ui.label("Parameter to optimize:");
    let params = ["Nose Length", "Fin Span", "Body Length", "Mass", "CG Position"];
    egui::ComboBox::from_label("")
        .selected_text(params[app.optimization_param_idx])
        .show_ui(ui, |ui| {
            for (i, param) in params.iter().enumerate() {
                ui.selectable_value(&mut app.optimization_param_idx, i, *param);
            }
        });
    
    // Goal selection
    ui.label("Optimization goal:");
    let goals = ["Maximize Altitude", "Maximize Velocity", "Minimize Drag"];
    egui::ComboBox::from_label("")
        .selected_text(goals[app.optimization_goal_idx])
        .show_ui(ui, |ui| {
            for (i, goal) in goals.iter().enumerate() {
                ui.selectable_value(&mut app.optimization_goal_idx, i, *goal);
            }
        });
    
    // Parameter range
    ui.label("Parameter range:");
    ui.add(egui::Slider::new(&mut app.optimization_min, 0.0..=50.0).text("Min"));
    ui.add(egui::Slider::new(&mut app.optimization_max, 0.0..=100.0).text("Max"));
    
    ui.separator();
    
    // Run optimization
    let can_optimize = app.component_tree.is_some() 
        && !app.is_optimizing 
        && app.optimization_max > app.optimization_min;
    
    if ui.add_enabled(can_optimize, egui::Button::new("🎯 Run Optimization"))
        .clicked()
    {
        app.is_optimizing = true;
        app.status_message = "Running optimization...".to_string();
        
        // Simple 1D optimization placeholder
        // In a real implementation, this would modify the component tree
        // and run the simulation for each evaluation
        let objective = |x: f64| -> f64 {
            // Target function: altitude (simplified)
            // Real implementation would clone tree, modify param, simulate
            -(x - 15.0).powi(2) + 200.0  // parabola with max at x=15
        };
        
        let gss = GoldenSectionSearch {
            max_iterations: 50,
            ..Default::default()
        };
        
        let result = gss.maximize(objective, app.optimization_min, app.optimization_max);
        
        app.optimization_result = Some(result);
        app.is_optimizing = false;
        app.status_message = "Optimization complete".to_string();
    }
    
    // Show progress
    if app.is_optimizing {
        ui.add(egui::ProgressBar::new(0.5).text("Optimizing..."));
    }
    
    // Show results
    ui.separator();
    if let Some(ref result) = app.optimization_result {
        ui.heading("Optimization Results");
        ui.label(format!("Optimal value: {:.4}", result.final_value));
        if let Some(param) = result.parameters.first() {
            ui.label(format!("Parameter: {:.2} ({:.2} - {:.2})", param.value, param.min, param.max));
        }
        ui.label(format!("Improvement: {:.1}%", result.improvement));
        ui.label(format!("Iterations: {}", result.iterations));
        ui.label(format!("Converged: {}", result.converged));
        
        // Plot convergence history
        if !result.history.is_empty() {
            ui.separator();
            ui.label("Convergence History");
            
            let points: egui_plot::PlotPoints = result.history.iter()
                .map(|(x, y)| [*x, *y])
                .collect();
            
            egui_plot::Plot::new("optimization_history")
                .height(150.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(egui_plot::Line::new(points).color(egui::Color32::GREEN));
                });
        }
    }
}