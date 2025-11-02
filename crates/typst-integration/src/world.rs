//! Typst World implementation for file system access

use crate::Result;
use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use std::sync::{ Arc, Mutex };
use typst::diag::{ FileError, FileResult };
use typst::foundations::Bytes;
use typst::syntax::{ FileId, Source };
use typst::text::{ Font, FontBook };
use typst::Library;
use chrono::{ Datelike, Local };

// Import LazyHash from typst-utils
use typst_utils::LazyHash;

/// System world for Typst compilation
pub struct SystemWorld {
    /// Project root directory
    root: PathBuf,
    /// Main entry file
    main: PathBuf,
    /// Standard library
    library: LazyHash<Library>,
    /// Font book
    book: LazyHash<FontBook>,
    /// Loaded fonts
    fonts: Vec<Font>,
    /// Source file cache
    sources: Arc<Mutex<HashMap<FileId, FileResult<Source>>>>,
    /// Binary file cache
    files: Arc<Mutex<HashMap<FileId, FileResult<Bytes>>>>,
}

impl SystemWorld {
    pub fn new(root: PathBuf, main: PathBuf) -> Result<Self> {
        let library = LazyHash::new(Library::default());

        let book = FontBook::new();
        let fonts = Vec::new();

        // TODO: Load system fonts properly
        // For now, we'll have an empty font list
        // In a real implementation, we'd use fontdb to find and load system fonts

        Ok(Self {
            root,
            main,
            library,
            book: LazyHash::new(book),
            fonts,
            sources: Arc::new(Mutex::new(HashMap::new())),
            files: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the main source file
    pub fn main_file(&self) -> &Path {
        &self.main
    }

    /// Resolve a file ID to a path
    fn id_to_path(&self, id: FileId) -> FileResult<PathBuf> {
        // Simplified path resolution
        // In real impl, would handle package imports, etc.
        let path = self.root.join(id.vpath().as_rootless_path());
        if path.exists() {
            Ok(path)
        } else {
            Err(FileError::NotFound(path))
        }
    }

    /// Load a source file
    fn load_source(&self, id: FileId) -> FileResult<Source> {
        let path = self.id_to_path(id)?;
        let content = std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Source::new(id, content))
    }

    /// Load a binary file
    fn load_file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.id_to_path(id)?;
        let content = std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(content.into())
    }
}

impl typst::World for SystemWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        // Get main file ID
        FileId::new(None, typst::syntax::VirtualPath::new(&self.main))
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let mut sources = self.sources.lock().unwrap();

        if let Some(result) = sources.get(&id) {
            return result.clone();
        }

        let result = self.load_source(id);
        sources.insert(id, result.clone());
        result
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let mut files = self.files.lock().unwrap();

        if let Some(result) = files.get(&id) {
            return result.clone();
        }

        let result = self.load_file(id);
        files.insert(id, result.clone());
        result
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<typst::foundations::Datetime> {
        let now = Local::now();
        let datetime = if let Some(offset) = offset {
            now.checked_add_signed(chrono::Duration::hours(offset))?
        } else {
            now
        };

        // Create time::Date from chrono DateTime
        use time::{ Date, Month };
        let date = Date::from_calendar_date(
            datetime.year(),
            Month::try_from(datetime.month() as u8).ok()?,
            datetime.day() as u8
        ).ok()?;
        Some(typst::foundations::Datetime::Date(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let root = PathBuf::from(".");
        let main = PathBuf::from("main.typ");
        let world = SystemWorld::new(root, main);
        assert!(world.is_ok());
    }
}
