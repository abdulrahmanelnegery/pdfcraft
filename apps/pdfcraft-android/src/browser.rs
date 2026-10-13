//! The file browser PdfCraft's open, save and folder dialogs use on Android: the system document
//! picker needs a Java activity result, so the app shows its own over shared storage.

use std::path::{Path, PathBuf};

use android_activity::AndroidApp;
use rfd::android::{Kind, PickRequest};

use crate::platform;

/// Where browsing starts when the dialog names no folder.
const ROOTS: [&str; 2] = ["/storage/emulated/0", "/sdcard"];

struct Entry {
    path: PathBuf,
    name: String,
    folder: bool,
}

pub struct Browser {
    request: Option<PickRequest>,
    dir: PathBuf,
    entries: Vec<Entry>,
    chosen: Vec<PathBuf>,
    file_name: String,
    access: bool,
}

impl Browser {
    pub fn new() -> Self {
        Self { request: None, dir: PathBuf::new(), entries: Vec::new(), chosen: Vec::new(), file_name: String::new(), access: false }
    }

    /// Whether a dialog is showing.
    pub fn busy(&self) -> bool {
        self.request.is_some()
    }

    /// Start answering `request` with the browser.
    pub fn begin(&mut self, request: PickRequest, app: &AndroidApp) {
        self.access = platform::has_storage_access(app);
        self.dir = request.directory.clone().filter(|d| d.is_dir()).unwrap_or_else(|| default_root());
        self.file_name = request.file_name.clone().unwrap_or_default();
        self.chosen.clear();
        self.request = Some(request);
        self.refresh();
    }

    /// Draw the browser over the app while a dialog waits.
    pub fn show(&mut self, ctx: &egui::Context, app: &AndroidApp) {
        let mut done: Option<Option<Vec<PathBuf>>> = None;
        let mut go: Option<PathBuf> = None;
        let mut open_settings = false;
        let mut recheck = false;
        let (kind, title, filters) = match &self.request {
            Some(r) => (r.kind, r.title.clone(), r.filters.clone()),
            None => return,
        };
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("android-file-browser")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            egui::Frame::NONE.fill(ui.visuals().panel_fill).show(ui, |ui| {
                ui.set_min_size(screen.size());
                ui.set_max_size(screen.size());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.add_sized([48.0, 40.0], egui::Button::new("✕")).clicked() {
                        done = Some(None);
                    }
                    ui.heading(title.clone().unwrap_or_else(|| default_title(kind).to_string()));
                });
                ui.separator();
                if !self.access {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label("PdfCraft Enhanced needs permission to read your files.");
                        ui.label("Turn on \"Allow access to manage all files\" for this app, then come back.");
                        ui.add_space(12.0);
                        if ui.add_sized([240.0, 48.0], egui::Button::new("Allow file access")).clicked() {
                            open_settings = true;
                        }
                        if ui.add_sized([240.0, 48.0], egui::Button::new("I have allowed it")).clicked() {
                            recheck = true;
                        }
                    });
                    return;
                }
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.add_sized([48.0, 40.0], egui::Button::new("↑")).clicked() {
                        go = self.dir.parent().map(Path::to_path_buf).filter(|p| p.starts_with("/storage") || p.starts_with("/sdcard"));
                    }
                    ui.label(egui::RichText::new(self.dir.display().to_string()).small());
                });
                let footer = 64.0;
                egui::ScrollArea::vertical().max_height(ui.available_height() - footer).auto_shrink([false, false]).show(ui, |ui| {
                    for entry in &self.entries {
                        if !entry.folder && matches!(kind, Kind::Folder) {
                            continue;
                        }
                        let selected = self.chosen.contains(&entry.path);
                        let icon = if entry.folder { "📁" } else { "📄" };
                        let row = ui.add_sized(
                            [ui.available_width(), 48.0],
                            egui::Button::new(format!("{icon}  {}", entry.name)).selected(selected).frame(selected),
                        );
                        if row.clicked() {
                            if entry.folder {
                                go = Some(entry.path.clone());
                            } else if matches!(kind, Kind::File) {
                                done = Some(Some(vec![entry.path.clone()]));
                            } else if matches!(kind, Kind::Files) {
                                match self.chosen.iter().position(|p| *p == entry.path) {
                                    Some(i) => {
                                        self.chosen.remove(i);
                                    }
                                    None => self.chosen.push(entry.path.clone()),
                                }
                            } else {
                                self.file_name = entry.name.clone();
                            }
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    match kind {
                        Kind::Folder => {
                            if ui.add_sized([200.0, 48.0], egui::Button::new("Choose this folder")).clicked() {
                                done = Some(Some(vec![self.dir.clone()]));
                            }
                        }
                        Kind::Files => {
                            let label = format!("Open {} file(s)", self.chosen.len());
                            if ui.add_enabled(!self.chosen.is_empty(), egui::Button::new(label).min_size(egui::vec2(200.0, 48.0))).clicked() {
                                done = Some(Some(self.chosen.clone()));
                            }
                        }
                        Kind::Save => {
                            ui.add_sized([ui.available_width() - 120.0, 48.0], egui::TextEdit::singleline(&mut self.file_name));
                            if ui
                                .add_enabled(!self.file_name.trim().is_empty(), egui::Button::new("Save").min_size(egui::vec2(100.0, 48.0)))
                                .clicked()
                            {
                                done = Some(Some(vec![self.dir.join(self.file_name.trim())]));
                            }
                        }
                        Kind::File => {
                            ui.label("Tap a file to open it");
                        }
                    }
                });
            });
        });
        if open_settings {
            platform::request_storage_access(app);
        }
        if recheck {
            self.access = platform::has_storage_access(app);
            self.refresh();
        }
        if let Some(dir) = go {
            self.dir = dir;
            self.refresh();
        }
        if let Some(result) = done {
            if let Some(request) = self.request.take() {
                request.finish(result);
            }
        }
        let _ = filters;
    }

    fn refresh(&mut self) {
        let filters = self.request.as_ref().map(|r| r.filters.clone()).unwrap_or_default();
        let wanted: Vec<String> = filters.iter().flat_map(|(_, e)| e.iter().map(|e| e.to_lowercase())).collect();
        let mut entries: Vec<Entry> = std::fs::read_dir(&self.dir)
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().into_owned();
                        if name.starts_with('.') {
                            return None;
                        }
                        let path = e.path();
                        let folder = path.is_dir();
                        let extension = path.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
                        let shown = folder || wanted.is_empty() || wanted.iter().any(|w| w == "*" || *w == extension);
                        shown.then_some(Entry { path, name, folder })
                    })
                    .collect()
            })
            .unwrap_or_default();
        entries.sort_by(|a, b| b.folder.cmp(&a.folder).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        self.entries = entries;
    }
}

fn default_root() -> PathBuf {
    ROOTS.iter().map(PathBuf::from).find(|p| p.is_dir()).unwrap_or_else(|| PathBuf::from("/"))
}

fn default_title(kind: Kind) -> &'static str {
    match kind {
        Kind::File => "Open a file",
        Kind::Files => "Choose files",
        Kind::Folder => "Choose a folder",
        Kind::Save => "Save as",
    }
}
