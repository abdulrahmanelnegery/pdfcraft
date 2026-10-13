//! PdfCraft Enhanced on Android: the same egui shell as the desktop app, started from a
//! NativeActivity, with an in-app file browser for its dialogs and PDFs opened from other apps.
//!
//! Experimental. Not yet wired: the system clipboard and sharing/exporting through Android's share
//! sheet.
#![cfg(target_os = "android")]

mod browser;
mod platform;

use android_activity::AndroidApp;
use eframe::App;
use pdfcraft_ui_egui::PdfCraftApp;

/// The desktop app plus what only Android has.
struct Shell {
    app: PdfCraftApp,
    activity: AndroidApp,
    browser: browser::Browser,
    checked_intent: bool,
}

impl Shell {
    /// A PDF another app asked us to view (`ACTION_VIEW`), opened once at start.
    fn open_incoming(&mut self) {
        if std::mem::replace(&mut self.checked_intent, true) {
            return;
        }
        if let Some(platform::Incoming { name, bytes }) = platform::incoming_document(&self.activity)
            && let Err(e) = self.app.open_bytes(&name, None, bytes)
        {
            log::warn!("could not open {name}: {e}");
        }
    }
}

impl App for Shell {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, raw);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.app.save(storage);
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.open_incoming();
        self.app.logic(ctx, frame);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        self.browser.show(ui.ctx(), &self.activity);
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let activity = app.clone();
    let options = eframe::NativeOptions { android_app: Some(app), renderer: eframe::Renderer::Wgpu, ..Default::default() };
    let result = eframe::run_native(
        "PdfCraft Enhanced",
        options,
        Box::new(move |cc| {
            let mut app = PdfCraftApp::new();
            if let Some(json) = cc.storage.and_then(|s| s.get_string("pdfcraft")) {
                app.restore(&json);
            }
            Ok(Box::new(Shell { app, activity, browser: browser::Browser::new(), checked_intent: false }))
        }),
    );
    if let Err(e) = result {
        log::error!("PdfCraft stopped: {e}");
    }
}
