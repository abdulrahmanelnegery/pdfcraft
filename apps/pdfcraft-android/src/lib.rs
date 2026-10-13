//! PdfCraft on Android: the same egui shell as the desktop app, started from a NativeActivity.
//!
//! Experimental. Not yet wired: the file picker (rfd has no Android backend, so documents need
//! Storage Access Framework calls through JNI), the share/open-with intents, and the soft keyboard.
#![cfg(target_os = "android")]

use android_activity::AndroidApp;
use pdfcraft_ui_egui::PdfCraftApp;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let options = eframe::NativeOptions { android_app: Some(app), renderer: eframe::Renderer::Wgpu, ..Default::default() };
    let result = eframe::run_native(
        "PdfCraft Enhanced",
        options,
        Box::new(|cc| {
            let mut app = PdfCraftApp::new();
            if let Some(json) = cc.storage.and_then(|s| s.get_string("pdfcraft")) {
                app.restore(&json);
            }
            Ok(Box::new(app))
        }),
    );
    if let Err(e) = result {
        log::error!("PdfCraft stopped: {e}");
    }
}
