//! The slice of `arboard`'s API that PdfCraft calls, for Android, where `arboard` has no backend.
//! There is no system clipboard here yet: every call reports an error, which the callers already
//! handle (copying an image or reading one from the clipboard says it is not available).

use std::borrow::Cow;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the system clipboard is not available on Android yet")
    }
}

impl std::error::Error for Error {}

pub struct ImageData<'a> {
    pub width: usize,
    pub height: usize,
    pub bytes: Cow<'a, [u8]>,
}

pub struct Clipboard;

impl Clipboard {
    pub fn new() -> Result<Self, Error> {
        Err(Error)
    }

    pub fn set_image(&mut self, _image: ImageData<'_>) -> Result<(), Error> {
        Err(Error)
    }

    pub fn get_image(&mut self) -> Result<ImageData<'static>, Error> {
        Err(Error)
    }

    pub fn get_text(&mut self) -> Result<String, Error> {
        Err(Error)
    }

    pub fn set(&mut self) -> Set {
        Set
    }
}

pub struct Set;

impl Set {
    pub fn file_list<P: AsRef<Path>>(self, _paths: &[P]) -> Result<(), Error> {
        Err(Error)
    }
}
