use crate::fs_utils::{FileNode, determine_file_type, read_dir};
use std::{
    fs::{self, canonicalize},
    path::Path,
    process::exit,
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
    /// The lines of the `opened_file`
    pub opened_file_lines: Result<Vec<String>, std::io::Error>,
    /// The children of the `opened_dir`
    pub files: Vec<FileNode>,
    /// The search filter for the file tree
    pub filters: Filters,
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
    // Search for a file by name
    SearchByFilename(String),
    // An action for if no user interaction happened for this frame
    None,
}

/// The Filters used to search the opened file tree
#[derive(Debug)]
pub struct Filters {
    /// The text contents of the search
    pub file_name_search: String,
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

        FileExplorerApp {
            files: nodes,
            opened_dir: opened_dir.ok().unwrap(),
            opened_file: None,
            opened_file_contents: Ok(String::from("")),
            opened_file_type: None,
            opened_file_lines: Ok(Vec::new()),
            filters: Filters {
                file_name_search: String::from(""),
            },
        }
    }
}

/// The methods of the FileExplorerApp
impl FileExplorerApp {
    /// Processes the action that took place during the [`FileExplorerApp::update`] function
    ///
    /// # Arguments
    ///
    /// * `self` - the application instance
    /// * `action` - the [`Action`] that occurred during the last frame
    pub fn post_update(&mut self, action: Action) -> Result<(), std::io::Error> {
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
                };
            }
            // Runs when the close file button is clicked
            Action::CloseFile => {
                self.opened_file = None;
                self.opened_file_contents = Ok(String::from(""));
                self.opened_file_lines = Ok(Vec::new());
                self.opened_file_type = None;
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
            }
            // Runs when we search for a file by name
            Action::SearchByFilename(search_file_name) => {
                println!("Searching for [{}]", search_file_name);

                for file in &mut self.files {
                    file.matches_filters = file
                        .file_name
                        .to_lowercase()
                        .contains(&search_file_name.trim().to_lowercase());
                }
            }
            // The action that is omitted if the user did nothing during the last frame
            Action::None => (),
        }

        Ok(())
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
                Ok(file_contents) => {
                    self.opened_file_lines =
                        Ok(file_contents.lines().map(|s| s.to_string()).collect());
                    self.opened_file_type = determine_file_type(&file.absolute_path);
                }
            }
        }

        Ok(())
    }
}
