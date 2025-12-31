use crate::app::FileExplorerApp;

// Import modules for this application

// The application struct itself
mod app;
// The filesystem utilities and structures
mod fs_utils;
// The UI rendering code which gets attached to the FileExplorerApp
mod ui;

/// The Entrypoint of the application. Reads the CWD for files and
/// constructs a GUI Window with the Application state.
fn main() {
    let _ = iced::application(
        FileExplorerApp::default,
        FileExplorerApp::update,
        FileExplorerApp::view,
    )
    .run();
}
