use crate::app::FederatedRocketApp;
use eframe::egui;
use federated_rocket_core::component_tree::{ComponentKey, ComponentTree};

pub fn show(app: &mut FederatedRocketApp, ui: &mut egui::Ui) {
    if app.component_tree.is_some() {
        // File path display
        if let Some(ref path) = app.current_file {
            ui.monospace(path);
            ui.separator();
        }

        // Component tree display
        ui.label(format!(
            "Components: {}",
            app.component_tree
                .as_ref()
                .map_or(0, |t| t.component_count())
        ));

        // Extract current selection before closure
        let current_selection = app.selected_component;
        let mut new_selection = current_selection;

        // Use a raw pointer to split the borrow between tree (shared) and selection (mutable)
        let tree_ptr: *const ComponentTree = match app.component_tree.as_ref() {
            Some(t) => t,
            None => &ComponentTree::new(),
        };

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 40.0)
            .show(ui, |ui| {
                // Safety: the tree is not mutated during rendering
                let tree = unsafe { &*tree_ptr };
                if let Some(root) = tree.root() {
                    render_component_node(ui, &mut new_selection, tree, root, 0, current_selection);
                } else {
                    ui.label("Empty rocket design");
                }
            });

        // Write back the selection after the closure
        app.selected_component = new_selection;

        // Add component button (placeholder)
        if ui.button("+ Add Component").clicked() {
            app.status_message = "Component editing not yet implemented".to_string();
        }
    } else {
        ui.label("No design loaded");
        ui.label("Use File > Open to load a .ork or .rkt file");

        // Simple file path input
        ui.horizontal(|ui| {
            ui.label("Path:");
            let mut path = String::new();
            if ui.text_edit_singleline(&mut path).changed() && !path.is_empty() {
                let p = std::path::Path::new(&path);
                if p.exists() {
                    match federated_rocket_fileio::format_detect::load_rocket_file(p) {
                        Ok(tree) => {
                            app.component_tree = Some(tree);
                            app.current_file = Some(path.clone());
                            app.status_message = format!("Loaded: {}", path);
                        }
                        Err(e) => {
                            app.error_message = Some(format!("Failed to load: {}", e));
                        }
                    }
                }
            }
            if ui.button("Browse").clicked() {
                app.show_file_open_dialog = true;
                app.dialog_path = path;
            }
        });
    }
}

fn render_component_node(
    ui: &mut egui::Ui,
    selected_component: &mut Option<ComponentKey>,
    tree: &ComponentTree,
    key: ComponentKey,
    depth: usize,
    current_selected: Option<ComponentKey>,
) {
    if let Some(node) = tree.get(key) {
        let indent = "  ".repeat(depth);
        let is_selected = current_selected == Some(key);

        ui.horizontal(|ui| {
            ui.label(indent);

            // Component icon/type indicator
            let icon = match node.component.component_type() {
                "Body Tube" => "\u{1F4E6}",
                "Nose Cone" => "\u{1F53A}",
                "Transition" => "\u{1F53B}",
                "Fin Set" => "\u{25B6}",
                "Parachute" => "\u{1FA82}",
                "Engine" => "\u{1F525}",
                "Mass Component" => "\u{2696}",
                "Bulkhead" => "\u{25A0}",
                "Launch Lug" => "\u{25CB}",
                "Centering Ring" => "\u{25C9}",
                "Inner Tube" => "\u{25A1}",
                "Tube Coupler" => "\u{25D0}",
                _ => "\u{25A1}",
            };

            // Selectable label with component info
            let label = format!(
                "{} {} ({})",
                icon,
                node.component.name(),
                node.component.component_type()
            );
            let response = ui.selectable_label(is_selected, &label);
            if response.clicked() {
                *selected_component = Some(key);
            }
        });

        // Show properties if selected
        if is_selected {
            ui.indent("properties", |ui| {
                ui.label(format!("Type: {}", node.component.component_type()));
                let pos = node.component.position();
                ui.label(format!(
                    "Position: ({:.2}, {:.2}, {:.2})",
                    pos.x, pos.y, pos.z
                ));
                if let Some(parent) = tree.parent(key) {
                    if let Some(pn) = tree.get(parent) {
                        ui.label(format!("Parent: {}", pn.component.name()));
                    }
                }
                ui.label(format!("Children: {}", node.children.len()));
            });
        }

        // Render children
        for child_key in &node.children {
            render_component_node(
                ui,
                selected_component,
                tree,
                *child_key,
                depth + 1,
                current_selected,
            );
        }
    }
}
