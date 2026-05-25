mod app;
mod panels;

use app::FederatedRocketApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Federated Rocket"),
        ..Default::default()
    };

    eframe::run_native(
        "Federated Rocket",
        options,
        Box::new(|_cc| Ok(Box::new(FederatedRocketApp::new()))),
    )
}
