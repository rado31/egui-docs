#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use egui_docs::app::DocsApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // egui and eframe report problems through `log`, and without a logger those
    // messages go nowhere: a missing `links` feature made every hyperlink do
    // nothing while warning about it to an audience of no one. `warn` by default
    // keeps startup quiet and still surfaces that class of bug; `RUST_LOG=debug`
    // (or `RUST_LOG=egui=debug`) when more is wanted.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-docs",
        options,
        Box::new(|cc| Ok(Box::new(DocsApp::new(cc)))),
    )
}

/// Web entry point.
///
/// The same `DocsApp` runs here — eframe swaps the backend, not the app. The
/// canvas is created by `index.html`; we only hand it to the runner.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Panics go to the browser console instead of silently aborting the module.
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let canvas = eframe::web_sys::window()
            .expect("no window")
            .document()
            .expect("no document")
            .get_element_by_id("egui_docs_canvas")
            .expect("no element with id 'egui_docs_canvas'")
            .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
            .expect("'egui_docs_canvas' is not a <canvas>");

        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(DocsApp::new(cc)))),
            )
            .await;

        // Replace the loading text with whatever happened.
        if let Some(loading) = eframe::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"))
        {
            match result {
                Ok(()) => loading.remove(),
                Err(error) => loading.set_inner_html(&format!(
                    "<p style='color:#b00'>Failed to start: {error:?}</p>"
                )),
            }
        }
    });
}
