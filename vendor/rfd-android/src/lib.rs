//! The slice of `rfd`'s async file-dialog API that PdfCraft calls, for Android, where `rfd` has no
//! backend. A pick does not open a system dialog: it queues a [`android::PickRequest`] that the
//! Android shell (`apps/pdfcraft-android`) shows as its own in-app file browser and completes. The
//! calls keep `rfd`'s names and signatures so the call sites stay unchanged.

use std::collections::VecDeque;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

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

/// Builder for a file dialog.
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
        let pick = self.queue(android::Kind::File);
        async move { pick.await.and_then(|v| v.into_iter().next()) }
    }

    pub fn pick_files(self) -> impl Future<Output = Option<Vec<FileHandle>>> {
        self.queue(android::Kind::Files)
    }

    pub fn pick_folder(self) -> impl Future<Output = Option<FileHandle>> {
        let pick = self.queue(android::Kind::Folder);
        async move { pick.await.and_then(|v| v.into_iter().next()) }
    }

    pub fn save_file(self) -> impl Future<Output = Option<FileHandle>> {
        let pick = self.queue(android::Kind::Save);
        async move { pick.await.and_then(|v| v.into_iter().next()) }
    }

    fn queue(self, kind: android::Kind) -> Pick {
        let shared = Arc::new(Mutex::new(Shared { result: None, waker: None }));
        lock(&android::QUEUE).push_back(android::PickRequest {
            kind,
            title: self.title,
            file_name: self.file_name,
            directory: self.directory,
            filters: self.filters,
            shared: shared.clone(),
        });
        Pick { shared }
    }
}

/// What the Android shell sees of the dialogs (not part of `rfd`).
pub mod android {
    use super::*;

    pub(super) static QUEUE: Mutex<VecDeque<PickRequest>> = Mutex::new(VecDeque::new());

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Kind {
        File,
        Files,
        Folder,
        Save,
    }

    /// A pick waiting for the user. Complete it with [`PickRequest::finish`] (or drop it to cancel).
    pub struct PickRequest {
        pub kind: Kind,
        pub title: Option<String>,
        /// The suggested name when saving.
        pub file_name: Option<String>,
        pub directory: Option<PathBuf>,
        /// `(name, extensions)`; no filters means any file.
        pub filters: Vec<(String, Vec<String>)>,
        pub(super) shared: Arc<Mutex<Shared>>,
    }

    impl PickRequest {
        /// Answer the pick: `None` cancels it.
        pub fn finish(self, paths: Option<Vec<PathBuf>>) {
            self.complete(paths);
        }

        fn complete(&self, paths: Option<Vec<PathBuf>>) {
            let mut shared = lock(&self.shared);
            if shared.result.is_none() {
                shared.result = Some(paths.map(|p| p.into_iter().map(FileHandle::from).collect()));
            }
            if let Some(waker) = shared.waker.take() {
                waker.wake();
            }
        }
    }

    impl Drop for PickRequest {
        fn drop(&mut self) {
            self.complete(None);
        }
    }

    /// The oldest waiting pick, if any.
    pub fn take_request() -> Option<PickRequest> {
        lock(&QUEUE).pop_front()
    }
}

pub(crate) struct Shared {
    result: Option<Option<Vec<FileHandle>>>,
    waker: Option<Waker>,
}

struct Pick {
    shared: Arc<Mutex<Shared>>,
}

impl Future for Pick {
    type Output = Option<Vec<FileHandle>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = lock(&self.shared);
        match shared.result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                shared.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// A poisoned lock still holds a usable queue.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
