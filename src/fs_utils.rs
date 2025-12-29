use std::{
    fs::{self, canonicalize},
    path::Path,
};

/// Represents a node in the file menu
#[derive(Clone, Debug)]
pub struct FileNode {
    /// The name of the file (excluding the path)
    pub file_name: String,
    /// The absolute path to this file, including the file name
    pub absolute_path: String,
    /// The parent folder of this file (Empty for the root folder)
    pub parent_folder: Option<String>,
    /// A flag to indicate if this node is a directory
    pub is_dir: bool,
}

/// File Node methods
impl FileNode {
    /// Constructs a file node from a relaltive path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read
    pub fn from_relative_path(path: &String) -> Result<FileNode, std::io::Error> {
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

    /// Returns a display-friendly name for the file node
    ///
    /// # Arguments
    /// * `self` - The file node instance
    pub fn display_name(&self) -> String {
        if self.is_dir {
            format!("📂 {}/", self.file_name)
        } else {
            format!("📄 {}", self.file_name)
        }
    }
}

/// Returns a list of all the FileNodes for the given path
///
/// # Arguments
///
/// * `path` - The path to read
pub fn read_dir(path: &String) -> Result<Vec<FileNode>, std::io::Error> {
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

    // Sort directories first, then files, both alphabetically
    nodes.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
        }
    });

    Ok(nodes)
}

/// Determines the file type based on the file extension
///
/// # Arguments
///
/// * `path` - The path to the file
pub fn determine_file_type(path: &String) -> Option<String> {
    let extension = Path::new(path).extension()?;

    let returned = match extension.to_str()? {
        _ => extension.to_str().map(|s| s.to_string()),
    };

    if returned.is_none() {
        println!(
            "Could not determine file type for extension: {:?}",
            extension
        );
    }

    returned
}
