use crate::app::FileExplorerApp;
use eframe::egui;

// Import modules for this application

// The application struct itself
mod app;
// The filesystem utilities and structures
mod fs_utils;
// The UI rendering code which gets attached to the FileExplorerApp
mod ui;

/// The Entrypoint of the application. Reads the CWD for files and
/// constructs a GUI Window with the Application state.
fn main() -> eframe::Result {
    // Initialize logging
    env_logger::init();

    // Launch options for the app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    let app = FileExplorerApp::default();

    eframe::run_native(
        "Rust File Explorer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<FileExplorerApp>::new(app))
        }),
    )
}
