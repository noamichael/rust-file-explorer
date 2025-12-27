use eframe::egui;
use std::{
    fs::{self, canonicalize},
    path::{Path, PathBuf},
};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
        opened_file: opened_file.ok().unwrap(),
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
    // contents: String,
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
            // contents: String::from(""),
            is_dir: metadata.is_dir(),
        })
    }
}

enum Action {
    OpenFile(FileNode),
    GoBack(FileNode),
    None,
}

struct MyApp {
    opened_file: FileNode,
    files: Vec<FileNode>,
}

impl MyApp {
    fn post_update(&mut self, action: Action) -> Result<(), std::io::Error> {
        match action {
            Action::OpenFile(node) => {
                match self.open_file(node) {
                    Ok(_) => {
                        println!("Opened file")
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e)
                    }
                };
            }
            Action::GoBack(opened_file) => {
                println!("ACTION(back): to {:?}", opened_file);

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
            self.files = read_dir(&absolute_path)?;
        } else {
            // let contents = fs::read_to_string(&file.file_name)?;
            // TODO
        }

        self.opened_file = opened_file;

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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Explorer");

            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    let opened_file = &self.opened_file;

                    ui.add(egui::Label::new(format!(
                        "Open File: {}",
                        opened_file.file_name
                    )));
                    // Render back link for directory
                    let back_label = ui.add(egui::Label::new("../").sense(egui::Sense::click()));

                    if back_label.hovered() {
                        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }

                    if back_label.clicked() {
                        action = Action::GoBack(opened_file.clone());
                    }

                    // Build left side file tree
                    for node in &self.files {
                        let file_label = ui.add(
                            egui::Label::new(node.file_name.clone()).sense(egui::Sense::click()),
                        );

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

    println!("ALL NODES: {:?}", nodes);

    Ok(nodes)
}
