//! Headless rendering of the app.
//!
//! `cargo test` renders each lesson with `egui_kittest` and compares it against
//! the PNG in `tests/snapshots/`. Nothing here needs a window or a GPU driver
//! beyond what wgpu picks by default.
//!
//! To accept the current look as the new baseline:
//!
//! ```sh
//! UPDATE_SNAPSHOTS=1 cargo test
//! ```

use egui_docs::app::DocsApp;
use egui_kittest::Harness;

/// Render one lesson at a realistic window size.
fn render_lesson(lesson: &'static str, name: &str) {
    let mut app = DocsApp::default();
    app.select_lesson(lesson);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 820.0))
        .wgpu()
        .build_ui(move |ui| app.show(ui));

    // A few passes, so panel sizes and scroll areas settle.
    harness.run();
    harness.snapshot(name);
}

#[test]
fn lesson_response() {
    render_lesson("Response", "lesson_response");
}

#[test]
fn lesson_ui() {
    render_lesson("Ui", "lesson_ui");
}

#[test]
fn lesson_id() {
    render_lesson("Id", "lesson_id");
}

#[test]
fn lesson_atom() {
    render_lesson("Atom", "lesson_atom");
}

#[test]
fn lesson_button() {
    render_lesson("Button", "lesson_button");
}
