use crate::app::{FileExplorerApp, Filters};
use crate::fs_utils::{FileNode, read_dir};
use eframe::egui;
use std::{fs::canonicalize, path::Path, process::exit};

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

    // This will convert the relative path to the absolute path
    let cwd = canonicalize(Path::new("./"));

    if cwd.is_err() {
        eprintln!("Could not open CWD: {}", cwd.err().unwrap());
        exit(1);
    }

    let cwd_absolute_path = &String::from(cwd.unwrap().to_str().unwrap());

    // Read the Current Working Directory to build the initial Tree Menu
    let nodes: Vec<FileNode> = match read_dir(cwd_absolute_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            let s: Vec<FileNode> = Vec::new();
            s
        }
    };

    // A referencee to the opened directory
    let opened_dir = FileNode::from_relative_path(cwd_absolute_path);

    let app = FileExplorerApp {
        files: nodes,
        opened_dir: opened_dir.ok().unwrap(),
        opened_file: None,
        opened_file_contents: Ok(String::from("")),
        opened_file_type: None,
        opened_file_line_numbers: None,
        filters: Filters {
            file_name_search: String::from(""),
        },
    };

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
