use eframe::egui;
use egui::Color32;
use std::{
    fs::{self, canonicalize},
    path::Path,
    process::exit,
};

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

/// Represents a node in the file menu
#[derive(Clone, Debug)]
struct FileNode {
    /// The name of the file (excluding the path)
    file_name: String,
    /// The absolute path to this file, including the file name
    absolute_path: String,
    /// The parent folder of this file (Empty for the root folder)
    parent_folder: Option<String>,
    /// A flag to indicate if this node is a directory
    is_dir: bool,
}

/// File Node methods
impl FileNode {
    /// Constructs a file node from a relaltive path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read
    fn from_relative_path(path: &String) -> Result<FileNode, std::io::Error> {
        let current_path = Path::new(path);
        let absolute_path = canonicalize(current_path)?;
        let metadata = fs::metadata(path)?;
        let file_name = match current_path.file_name() {
            Some(p) => String::from(p.to_str().unwrap()),
            None => String::from(path),
        };
        let parent_folder = match absolute_path.parent() {
            Some(p) => Some(String::from(p.to_str().unwrap())),
            None => None,
        };

        Ok(FileNode {
            file_name,
            absolute_path: String::from(absolute_path.to_str().unwrap()),
            parent_folder,
            is_dir: metadata.is_dir(),
        })
    }
}

// The actions that can occur for the application. During the `update` function,
// no app state mutations should occur. Instead, the `update` function returns
// the action (if any) that took place during that frame and the `post_update`
// function will apply the state changes.
enum Action {
    // An action for when a file was clicked in the menu
    OpenFile(FileNode),
    // An action for when the "close file" button was click
    CloseFile,
    // An action for when the user attempts to navigate up a directory
    GoBack(FileNode),
    // An action for if no user interaction happened for this frame
    None,
}

// The application state
struct FileExplorerApp {
    // The directory currently opened
    opened_dir: FileNode,
    // The file currently opened for viewing (if present)
    opened_file: Option<FileNode>,
    // The contents of the `opened_file`
    opened_file_contents: Result<String, std::io::Error>,
    // The children of the `opened_dir`
    files: Vec<FileNode>,
}

/// The methods of the FileExplorerApp
impl FileExplorerApp {
    /// Processes the action that took place during the [`FileExplorerApp::update`] function
    ///
    /// # Arguments
    ///
    /// * `self` - the application instance
    /// * `action` - the [`Action`] that occurred during the last frame
    fn post_update(&mut self, action: Action) -> Result<(), std::io::Error> {
        match action {
            // Runs when a file node in the tree is clicked
            Action::OpenFile(node) => {
                match self.open_file(node) {
                    Ok(_) => {
                        println!("Successfully opened file")
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e)
                    }
                };
            }
            // Runs when the close file button is clicked
            Action::CloseFile => {
                self.opened_file = None;
                self.opened_file_contents = Ok(String::from(""));
            }
            // Runs when the top level `../` button is clicked
            Action::GoBack(opened_file) => {
                match opened_file.parent_folder {
                    Some(parent) => {
                        let parent_node = FileNode::from_relative_path(&parent);
                        let _ = self.open_file(parent_node.expect("Could not read parent file"));
                    }
                    None => {
                        // Do nothing
                        println!("Could not find parent folder...")
                    }
                }
            }
            // The action that is omitted if the user did nothing during the last frame
            Action::None => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Opens a file or directory. This will set `opened_file` or `opened_dir` based on the file type.
    ///
    /// # Arguments
    ///
    /// * `self` - The application instancee
    /// * `file` - The file that should be opened from the file tree. Can be a `File` or `Directory` node.
    fn open_file(&mut self, file: FileNode) -> Result<(), std::io::Error> {
        println!("Attempting to open, {:?}", file);
        let opened_file = file.clone();
        let absolute_path = opened_file.absolute_path.clone();

        if opened_file.is_dir {
            match read_dir(&absolute_path) {
                Err(e) => {
                    eprintln!("Could not open file: {}", e);
                }
                Ok(v) => {
                    self.opened_dir = opened_file;
                    self.files = v;
                }
            }
        } else {
            self.opened_file = Some(opened_file);
            self.opened_file_contents = fs::read_to_string(&file.absolute_path);
        }

        Ok(())
    }
}

impl eframe::App for FileExplorerApp {
    /// Draws the UI for the given frame. This is called for each frame.
    /// This function should not mutate any state so as to avoid borrow issues.
    ///
    /// # Arguments
    ///
    /// * `self` - The application instance
    /// * `ctx` - The drawing context
    /// * `_frame` - The frame being drawn (unused)
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The action performed during this frame.
        let mut action = Action::None;

        // Set Styles
        ctx.style_mut(|style| {
            style
                .text_styles
                .get_mut(&egui::TextStyle::Heading)
                .unwrap()
                .size = 32.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size = 24.0;
        });

        // Left navigation tree
        egui::SidePanel::left("file_explorer").show(ctx, |ui| {
            ui.heading(&self.opened_dir.file_name);
            ui.add(egui::Separator::default().horizontal());

            // Draw the file tree
            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    // Render back link for directory
                    if self.opened_dir.absolute_path != "/" {
                        let back_label =
                            ui.add(egui::Label::new("../").sense(egui::Sense::click()));

                        ui.add(egui::Separator::default().horizontal());

                        if back_label.hovered() {
                            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }

                        if back_label.clicked() {
                            action = Action::GoBack(self.opened_dir.clone());
                        }
                    }

                    // Build left side file tree
                    for node in &self.files {
                        let gui_file_name = if node.is_dir {
                            format!("{}/", node.file_name)
                        } else {
                            String::from(&node.file_name)
                        };

                        let mut file_name_text = egui::RichText::new(gui_file_name);

                        // Draw selected file
                        match &self.opened_file {
                            Some(opened_file) => {
                                if opened_file.absolute_path == node.absolute_path {
                                    file_name_text = file_name_text
                                        .underline()
                                        .background_color(Color32::LIGHT_BLUE)
                                        .color(Color32::BLACK);
                                }
                            }
                            None => {
                                // do nothing
                            }
                        }

                        let file_label =
                            ui.add(egui::Label::new(file_name_text).sense(egui::Sense::click()));

                        ui.add(egui::Separator::default().horizontal());

                        if file_label.hovered() {
                            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }

                        if file_label.clicked() {
                            println!("CLICKED {}", node.file_name);
                            action = Action::OpenFile(node.clone());
                        }
                    }
                });
        });

        // Main window panel
        egui::CentralPanel::default().show(ctx, |ui| {
            //Content that DOES NOT SCROLL

            match &self.opened_file {
                Some(file) => {
                    ui.horizontal(|ui| {
                        ui.heading(format!("{}", file.file_name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_button =
                                ui.add(egui::Button::new("Close").fill(Color32::DARK_RED));
                            if close_button.clicked() {
                                action = Action::CloseFile;
                            }
                        });
                    });
                }
                None => {
                    ui.heading(String::from("No File Opened"));
                }
            };

            ui.add(egui::Separator::default().horizontal());

            // Scrolling text content
            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    match &self.opened_file {
                        Some(opened_file) => {
                            match &self.opened_file_contents {
                                Ok(contents) => {
                                    // Determine the file type for syntax highlighting
                                    let file_type = determine_file_type(&opened_file.absolute_path);
                                    egui_extras::syntax_highlighting::code_view_ui(
                                        ui,
                                        &egui_extras::syntax_highlighting::CodeTheme::default(),
                                        contents,
                                        &String::from(file_type.unwrap_or(String::from("text"))),
                                    );
                                    // ui.code(contents);
                                    // ui.add(egui::Label::new(contents));
                                }
                                Err(e) => {
                                    let error = egui::RichText::new(format!("Error: {}", e))
                                        .color(Color32::RED);
                                    ui.add(egui::Label::new(error));
                                }
                            }
                        }
                        None => {
                            ui.add(egui::Label::new(String::from(
                                "Please select a file from the menu",
                            )));
                        }
                    };
                });
        });

        let _ = self.post_update(action);
    }
}

/// Returns a list of all the FileNodes for the given path
///
/// # Arguments
///
/// * `path` - The path to read
fn read_dir(path: &String) -> Result<Vec<FileNode>, std::io::Error> {
    let mut nodes: Vec<FileNode> = Vec::new();

    let entries = match fs::read_dir(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            return Ok(nodes);
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e.path(),
            Err(_) => return Ok(nodes),
        };

        match FileNode::from_relative_path(&String::from(entry.to_str().unwrap())) {
            Ok(node) => nodes.push(node),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    // Skip files that cannot be accessed due to permission issues
                    eprintln!("read_dir: permission denied for file: {}", entry.display());
                    continue;
                }
                eprintln!("read_dir: could not read file: {}, {}", e, entry.display());
                continue;
            }
        }
    }

    Ok(nodes)
}

fn determine_file_type(path: &String) -> Option<String> {
    let extension = Path::new(path).extension()?;

    let returned = match extension.to_str()? {
        "rs" => Some(String::from("rust")),
        "js" => Some(String::from("javascript")),
        "ts" => Some(String::from("typescript")),
        "html" | "htm" => Some(String::from("html")),
        "css" => Some(String::from("css")),
        "xml" => Some(String::from("xml")),
        "py" => Some(String::from("python")),
        "txt" => Some(String::from("text")),
        "md" => Some(String::from("markdown")),
        "json" => Some(String::from("json")),
        "toml" => Some(String::from("toml")),
        "yaml" | "yml" => Some(String::from("yml")),
        _ => extension.to_str().map(|s| s.to_string()),
    };

    if returned.is_none() {
        println!("Could not determine file type for extension: {:?}", extension);
    }

    returned
}
