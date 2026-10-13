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
    /// A pick the system document picker is answering.
    native_pick: Option<rfd::android::PickRequest>,
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

impl Shell {
    /// Opening goes to the system document picker; saving and choosing folders use the in-app browser.
    fn route_picks(&mut self) {
        if let Some(request) = self.native_pick.take() {
            match platform::take_picked() {
                Some(paths) => request.finish(Some(paths).filter(|p| !p.is_empty())),
                None => self.native_pick = Some(request),
            }
        }
        if self.native_pick.is_some() || self.browser.busy() {
            return;
        }
        let Some(request) = rfd::android::take_request() else { return };
        let multiple = match request.kind {
            rfd::android::Kind::File => false,
            rfd::android::Kind::Files => true,
            _ => return self.browser.begin(request, &self.activity),
        };
        if platform::request_documents(&self.activity, multiple) {
            self.native_pick = Some(request);
        } else {
            self.browser.begin(request, &self.activity);
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
        self.route_picks();
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
            platform::set_wake(cc.egui_ctx.clone());
            let mut app = PdfCraftApp::new();
            // A finger drag pans the page; text selection is the Select tool's job, one tap away.
            app.quick_tool = pdfcraft_ui_egui::QuickTool::Hand;
            app.compact_forced = true;
            if let Some(json) = cc.storage.and_then(|s| s.get_string("pdfcraft")) {
                app.restore(&json);
            }
            Ok(Box::new(Shell { app, activity, browser: browser::Browser::new(), native_pick: None, checked_intent: false }))
        }),
    );
    if let Err(e) = result {
        log::error!("PdfCraft stopped: {e}");
    }
}
