use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    Import,   // Typst import
    Include,  // Typst include
    Asset,    // Image, data file, etc.
    Package,  // Typst package
}

#[derive(Debug, Clone)]
pub struct FileDependency {
    pub path: PathBuf,
    pub dependency_type: DependencyType,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectSettings {
    pub format_on_save: bool,
    pub auto_compile: bool,
    pub compile_on_save: bool,
    pub output_directory: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub main_file: Option<PathBuf>,
    pub dependencies: HashMap<PathBuf, FileDependency>,
    pub settings: ProjectSettings,
    pub compiler_args: Vec<String>,
}

impl Project {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            main_file: None,
            dependencies: HashMap::new(),
            settings: ProjectSettings::default(),
            compiler_args: Vec::new(),
        }
    }

    pub fn discover(root: PathBuf) -> Result<Self> {
        // TODO: Scan directory for .typ files
        // TODO: Look for project configuration
        // TODO: Detect main file
        Ok(Self::new(root))
    }

    pub fn add_dependency(&mut self, path: PathBuf, dep_type: DependencyType) {
        let dep = FileDependency {
            path: path.clone(),
            dependency_type: dep_type,
            last_modified: SystemTime::now(),
        };
        self.dependencies.insert(path, dep);
    }

    pub fn is_file_in_project(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
    }
}

