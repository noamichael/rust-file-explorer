use iced::{
    Task,
    widget::pane_grid::{self},
};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

use crate::fs_utils::{FileNode, determine_file_type, read_dir};
use std::{
    fs::{self, canonicalize},
    path::Path,
    process::exit,
    time::Duration,
};

/// The application state
#[derive(Debug)]
pub struct FileExplorerApp {
    /// The directory currently opened
    pub opened_dir: FileNode,
    /// The file currently opened for viewing (if present)
    pub opened_file: Option<FileNode>,
    /// The contents of the `opened_file`
    pub opened_file_contents: Result<String, std::io::Error>,
    /// The type of the `opened_file` (if present)
    pub opened_file_type: Option<String>,
    /// The children of the `opened_dir`
    pub files: Vec<FileNode>,
    /// The search filter for the file tree
    pub filters: Filters,
    /// Whether the application is in dark mode
    pub system_color_mode: dark_light::Mode,
    /// The state of the pane grid
    pub panes: pane_grid::State<PaneContent>,
    /// Syntax highlighting data
    pub highlighting: Highlighting,
    // The file node for the file info modal (if open)
    pub file_info_modal_node: Option<FileNode>,
    /// A boolean to track if the file info modal is open
    pub file_info_modal_open: bool,
}

/// The actions that can occur for the application. During the `update` function,
/// no app state mutations should occur. Instead, the `update` function returns
/// the action (if any) that took place during that frame and the `post_update`
/// function will apply the state changes.
#[derive(Debug, Clone)]
pub enum Action {
    // An action for when a file was clicked in the menu
    OpenFile(usize),
    // An action for when the "close file" button was click
    CloseFile,
    // An action for when the user attempts to navigate up a directory
    GoBack(),
    // Schedules a debounced search for a file by name. Calls SearchByFilename after delay
    DebouncedSearch(String),
    // Search for a file by name
    SearchByFilename(String),
    // An action for when the panes are resized
    PanesResized(pane_grid::ResizeEvent),
    // An action for when the context menu is opened on a file
    OpenContextMenu(ContextMenuAction),
    // An action for when the file info modal is closed
    CloseFileInfoModal,
}
#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    OpenFileInfoModal(usize),
}


/// The Filters used to search the opened file tree
#[derive(Debug)]
pub struct Filters {
    /// The text contents of the search
    pub file_name_search: String,
    /// The abort handler for the current operation
    pub file_filter_handle: Option<iced::task::Handle>,
}

#[derive(Debug)]
pub enum PaneContent {
    Sidebar,
    Content,
}

#[derive(Debug)]
pub struct Highlighting {
    pub syntax_set: syntect::parsing::SyntaxSet,
    pub theme_set: syntect::highlighting::ThemeSet,
}

/// The default methods
impl Default for FileExplorerApp {
    fn default() -> Self {
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

        let system_color_mode = match dark_light::detect() {
            Ok(mode) => mode,
            Err(_) => dark_light::Mode::Light,
        };

        println!("Detected system color mode: {:?}", system_color_mode);

        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.2,
            a: Box::new(pane_grid::Configuration::Pane(PaneContent::Sidebar)),
            b: Box::new(pane_grid::Configuration::Pane(PaneContent::Content)),
        });

        FileExplorerApp {
            files: nodes,
            opened_dir: opened_dir.ok().unwrap(),
            opened_file: None,
            opened_file_contents: Ok(String::from("")),
            opened_file_type: None,
            filters: Filters {
                file_name_search: String::from(""),
                file_filter_handle: None,
            },
            system_color_mode,
            panes,
            highlighting: Highlighting {
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme_set: ThemeSet::load_defaults(),
            },
            file_info_modal_node: None,
            file_info_modal_open: false,
        }
    }
}

/// The methods of the FileExplorerApp
impl FileExplorerApp {
    /// Processes the action that took place during the [`FileExplorerApp::view`] function
    ///
    /// # Arguments
    ///
    /// * `self` - the application instance
    /// * `action` - the [`Action`] that occurred during the last frame
    pub fn post_update(&mut self, action: Action) -> Task<Action> {
        let opened_dir = &self.opened_dir;
        match action {
            // Runs when a file node in the tree is clicked
            Action::OpenFile(node) => {
                match self.open_child_file(node) {
                    Ok(_) => {
                        println!("Successfully opened file")
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e)
                    }
                }
                Task::none()
            }
            // Runs when the close file button is clicked
            Action::CloseFile => {
                self.opened_file = None;
                self.opened_file_contents = Ok(String::from(""));
                self.opened_file_type = None;
                Task::none()
            }
            // Runs when the top level `../` button is clicked
            Action::GoBack() => {
                match &opened_dir.parent_folder {
                    Some(parent) => {
                        let parent_node = FileNode::from_relative_path(parent);
                        let _ = self.open_file(parent_node.expect("Could not read parent file"));
                    }
                    None => {
                        // Do nothing
                        println!("Could not find parent folder...")
                    }
                }
                Task::none()
            }
            // Runs when we search for a file by name
            Action::DebouncedSearch(search_file_name) => {
                // Store the search in the state
                self.filters.file_name_search = search_file_name.clone();

                // Abort any existing filter operation
                let _ = self
                    .filters
                    .file_filter_handle
                    .as_ref()
                    .map(|abort_handler| abort_handler.abort());

                // Create a task that performs the search after a delay
                let handler =
                    Task::perform(tokio::time::sleep(Duration::from_millis(500)), move |_| {
                        Action::SearchByFilename(search_file_name)
                    });

                // Split handler into task_handler and abort_handler
                let (task_handler, abort_handler) = handler.abortable();

                // store the abort_handler
                self.filters.file_filter_handle = Some(abort_handler);

                // Return the task_handler for Iced to execute later
                task_handler
            }
            Action::SearchByFilename(search_file_name) => {
                println!("Searching for [{}]", search_file_name);

                for file in &mut self.files {
                    file.matches_filters = file
                        .file_name
                        .to_lowercase()
                        .contains(&search_file_name.trim().to_lowercase());
                }

                Task::none()
            }
            // Runs when the panes are resized
            Action::PanesResized(event) => {
                self.panes.resize(event.split, event.ratio);
                Task::none()
            }
            Action::OpenContextMenu(context_menu_action) => {
                match context_menu_action {
                    ContextMenuAction::OpenFileInfoModal(index) => {
                        println!("Opening File Info Model for file at index: {}", index);
                        let file_node = self.files.get(index).cloned();
                        self.file_info_modal_node = file_node;
                        self.file_info_modal_open = true;
                    }
                }
                Task::none()
            }
            Action::CloseFileInfoModal => {
                self.file_info_modal_open = false;
                self.file_info_modal_node = None;
                Task::none()
            }
        }
    }

    fn open_child_file(&mut self, index: usize) -> Result<(), std::io::Error> {
        let file = &self.files[index];
        self.open_file(file.clone())
    }

    /// Opens a file or directory. This will set `opened_file` or `opened_dir` based on the file type.
    ///
    /// # Arguments
    ///
    /// * `self` - The application instancee
    /// * `file` - The file that should be opened from the file tree. Can be a `File` or `Directory` node.
    fn open_file(&mut self, file: FileNode) -> Result<(), std::io::Error> {
        println!("Attempting to open, {:?}", file);

        if let Some(f) = &self.opened_file
            && f.absolute_path == file.absolute_path
        {
            println!("File is already opened - skipping");
            return Ok(());
        }

        let opened_file = file.clone();
        let absolute_path = opened_file.absolute_path.clone();

        if opened_file.is_dir {
            self.filters.file_name_search.clear();
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

            match &self.opened_file_contents {
                // Ignore errors when reading file contents
                Err(_) => {}
                Ok(_) => {
                    self.opened_file_type = determine_file_type(&file.absolute_path);
                }
            }
        }

        Ok(())
    }
}
