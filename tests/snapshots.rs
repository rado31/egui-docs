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
use egui_docs::i18n::SOURCE_LANGUAGE;
use egui_kittest::{Harness, SnapshotResults};

/// Render one lesson, by its id, at a realistic window size — once per
/// language. English goes to `lesson_<id>.png`, every other language to
/// `lesson_<id>_<code>.png`: a translation can overflow where English did not.
///
/// Also fails if the lesson asked for a message its language does not have.
fn render_lesson(lesson: &'static str) {
    let codes: Vec<&str> = DocsApp::default().i18n().codes().collect();
    // One harness per language, one result for the whole test: every language
    // is compared (and updated) even when an earlier one differs. Fails when
    // dropped, if anything did.
    let mut results = SnapshotResults::new();

    for code in codes {
        let mut app = DocsApp::default();
        assert!(app.set_language(code));
        app.select_lesson(lesson);

        let mut harness = Harness::builder()
            .with_size(egui::vec2(1280.0, 820.0))
            .wgpu()
            .build_ui_state(|ui, app: &mut DocsApp| app.show(ui), app);

        // A few passes, so panel sizes and scroll areas settle.
        harness.run();

        let missing = harness.state().i18n().take_missing();
        assert!(
            missing.is_empty(),
            "lesson {lesson:?} in {code:?} looked up messages that are not there:\n{}",
            missing.into_iter().collect::<Vec<_>>().join("\n")
        );

        if code == SOURCE_LANGUAGE {
            harness.snapshot(format!("lesson_{lesson}"));
        } else {
            harness.snapshot(format!("lesson_{lesson}_{code}"));
        }
        results.extend_harness(&mut harness);
    }
}

#[test]
fn lesson_response() {
    render_lesson("response");
}

#[test]
fn lesson_ui() {
    render_lesson("ui");
}

#[test]
fn lesson_id() {
    render_lesson("id");
}

#[test]
fn lesson_atom() {
    render_lesson("atom");
}

#[test]
fn lesson_button() {
    render_lesson("button");
}
