use eframe::egui;
use egui::Color32;
use std::{
    fs::{self, canonicalize},
    path::Path,
};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    let nodes: Vec<FileNode> = match read_dir(&String::from("./")) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            let s: Vec<FileNode> = Vec::new();
            s
        }
    };

    let opened_file = FileNode::from_relative_path(&String::from("./"));

    let app = MyApp {
        files: nodes,
        opened_dir: opened_file.ok().unwrap(),
        opened_file: None,
        opened_file_contents: String::from("No File Selected"),
    };

    eframe::run_native(
        "Application",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::new(app))
        }),
    )
}

#[derive(Clone, Debug)]
struct FileNode {
    file_name: String,
    parent_folder: Option<String>,
    absolute_path: String,
    is_dir: bool,
}

impl FileNode {
    fn from_relative_path(path: &String) -> Result<FileNode, std::io::Error> {
        let current_path = Path::new(path);
        let absolute_path = canonicalize(current_path).expect("Could not find path");
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

enum Action {
    OpenFile(FileNode),
    CloseFile,
    GoBack(FileNode),
    None,
}

struct MyApp {
    opened_dir: FileNode,
    opened_file: Option<FileNode>,
    opened_file_contents: String,
    files: Vec<FileNode>,
}

impl MyApp {
    fn post_update(&mut self, action: Action) -> Result<(), std::io::Error> {
        match action {
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
            Action::CloseFile => {
                self.opened_file = None;
                self.opened_file_contents.clear();
            }
            Action::GoBack(opened_file) => {
                match opened_file.parent_folder {
                    Some(parent) => {
                        println!("ACTION(back): parent {:?}", parent);
                        let parent_node = FileNode::from_relative_path(&parent);
                        let _ = self.open_file(parent_node.expect("Could not read parent file"));
                    }
                    None => {
                        // Do nothing
                        println!("Could not find parent folder...")
                    }
                }
            }
            Action::None => {
                // Do nothing
            }
        }

        Ok(())
    }

    fn open_file(&mut self, file: FileNode) -> Result<(), std::io::Error> {
        println!("Attempting to open, {:?}", file);
        let opened_file = file.clone();
        let absolute_path = opened_file.absolute_path.clone();

        if opened_file.is_dir {
            self.opened_dir = opened_file;
            self.files = read_dir(&absolute_path)?;
        } else {
            self.opened_file = Some(opened_file);
            let contents = match fs::read_to_string(&file.absolute_path) {
                Ok(contents) => contents,
                Err(e) => e.to_string(),
            };

            println!("Read File: {}", contents);

            self.opened_file_contents = contents;
        }

        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            ui.heading("File Explorer");

            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    // Render back link for directory
                    let back_label = ui.add(egui::Label::new("../").sense(egui::Sense::click()));

                    if back_label.hovered() {
                        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }

                    if back_label.clicked() {
                        action = Action::GoBack(self.opened_dir.clone());
                    }

                    // Build left side file tree
                    for node in &self.files {
                        let gui_file_name = if node.is_dir {
                            format!("{}/", node.file_name)
                        } else {
                            String::from(&node.file_name)
                        };

                        let file_label =
                            ui.add(egui::Label::new(gui_file_name).sense(egui::Sense::click()));

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
                        ui.add(egui::Label::new(format!("Open File: {}", file.file_name)));
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
                    // Draw no header
                }
            };

            // Scrolling text content
            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    match &self.opened_file {
                        Some(_) => {
                            ui.add(egui::Label::new(&self.opened_file_contents));
                        }
                        None => {
                            ui.add(egui::Label::new(String::from("No File Opened")));
                        }
                    };
                });
        });

        let _ = self.post_update(action);
    }
}

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
                eprintln!("error: {}", e);
                return Ok(nodes);
            }
        }
    }

    Ok(nodes)
}
