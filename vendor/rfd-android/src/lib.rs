//! The slice of `rfd`'s async file-dialog API that PdfCraft calls, for Android, where `rfd` has no
//! backend. The dialogs choose nothing yet (every pick resolves to `None`): picking a document on
//! Android goes through the Storage Access Framework, which needs a Java activity result and is
//! done outside this shim. Names and signatures follow `rfd` so the call sites stay unchanged.

use std::path::{Path, PathBuf};

/// A chosen file or folder.
#[derive(Clone, Debug)]
pub struct FileHandle {
    path: PathBuf,
}

impl FileHandle {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    pub async fn read(&self) -> Vec<u8> {
        std::fs::read(&self.path).unwrap_or_default()
    }
}

impl From<PathBuf> for FileHandle {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

/// Builder for a file dialog; the options are kept for when a picker backs it.
#[derive(Clone, Debug, Default)]
pub struct AsyncFileDialog {
    title: Option<String>,
    file_name: Option<String>,
    directory: Option<PathBuf>,
    filters: Vec<(String, Vec<String>)>,
}

impl AsyncFileDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[impl ToString]) -> Self {
        self.filters.push((name.into(), extensions.iter().map(ToString::to_string).collect()));
        self
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn set_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }

    pub fn set_directory(mut self, path: impl AsRef<Path>) -> Self {
        self.directory = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn pick_file(self) -> impl Future<Output = Option<FileHandle>> {
        std::future::ready(None)
    }

    pub fn pick_files(self) -> impl Future<Output = Option<Vec<FileHandle>>> {
        std::future::ready(None)
    }

    pub fn pick_folder(self) -> impl Future<Output = Option<FileHandle>> {
        std::future::ready(None)
    }

    pub fn save_file(self) -> impl Future<Output = Option<FileHandle>> {
        std::future::ready(None)
    }
}
