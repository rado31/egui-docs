//! Checks on the translations in `locales/`. They catch what a reader would
//! otherwise find first: a broken file, a message one language forgot, or a
//! message the code asks for that no language has.

use std::collections::BTreeSet;
use std::path::Path;

use egui_docs::i18n::{I18n, SOURCE_LANGUAGE};

#[test]
fn every_file_parses() {
    let i18n = I18n::new();
    assert!(
        i18n.problems().is_empty(),
        "broken translation files:\n{}",
        i18n.problems().join("\n")
    );
}

/// Every language has exactly the messages and attributes English has. A
/// missing one would silently fall back to English; an extra one is usually a
/// typo in an id, which is the same bug seen from the other side.
#[test]
fn every_language_matches_the_source_language() {
    let i18n = I18n::new();
    let source = i18n.ids(SOURCE_LANGUAGE);

    let mut report = String::new();
    for code in i18n.codes().filter(|code| *code != SOURCE_LANGUAGE) {
        let ids = i18n.ids(code);
        for id in source.difference(&ids) {
            report.push_str(&format!("locales/{code}: missing {id}\n"));
        }
        for id in ids.difference(&source) {
            report.push_str(&format!(
                "locales/{code}: {id} is not in locales/{SOURCE_LANGUAGE}\n"
            ));
        }
    }
    assert!(report.is_empty(), "{report}");
}

/// Every id written literally in the code — `tr.get("...")`, `tr.fmt("...")`
/// — exists in the source language. The snapshot tests only see the messages
/// the default settings happen to show; this sees all the literal ones.
#[test]
fn every_id_used_in_the_code_exists() {
    let source = I18n::new().ids(SOURCE_LANGUAGE);

    let mut used = BTreeSet::new();
    let mut files = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(path) = files.pop() {
        if path.is_dir() {
            files.extend(std::fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let text = std::fs::read_to_string(&path).unwrap();
            for call in ["tr.get(", "tr.fmt("] {
                for (start, _) in text.match_indices(call) {
                    let rest = text[start + call.len()..].trim_start();
                    if let Some(rest) = rest.strip_prefix('"') {
                        let id = &rest[..rest.find('"').unwrap()];
                        used.insert((id.to_owned(), path.display().to_string()));
                    }
                }
            }
        }
    }

    let missing: Vec<String> = used
        .into_iter()
        .filter(|(id, _)| !source.contains(id))
        .map(|(id, file)| format!("{file}: {id}"))
        .collect();
    assert!(
        missing.is_empty(),
        "not in locales/{SOURCE_LANGUAGE}:\n{}",
        missing.join("\n")
    );
}

#[test]
fn best_match_prefers_the_exact_tag() {
    let i18n = I18n::new();
    assert_eq!(i18n.best_match("ru-RU"), Some("ru"));
    assert_eq!(i18n.best_match("ru_RU"), Some("ru"));
    assert_eq!(i18n.best_match("en"), Some("en"));
    assert_eq!(i18n.best_match("xx-YY"), None);
}

/// Russian has three plural forms for whole numbers; English has two.
#[test]
fn plurals_follow_the_language() {
    let i18n = I18n::new();
    let distinct = |code: &str, ids: usize, widgets: usize| {
        i18n.tr_in(code).unwrap().fmt(
            "id-distinct",
            &[("ids", ids.into()), ("widgets", widgets.into())],
        )
    };
    assert_eq!(distinct("en", 1, 3), "1 distinct Id for 3 widgets");
    assert_eq!(distinct("ru", 1, 1), "1 уникальный Id на 1 виджет");
    assert_eq!(distinct("ru", 3, 3), "3 уникальных Id на 3 виджета");
    assert_eq!(distinct("ru", 5, 5), "5 уникальных Id на 5 виджетов");
}

/// Switching language mid-session swaps a lesson's default text — here the
/// demo button's label — and the page's own text with it.
#[test]
fn switching_language_retranslates_default_text() {
    use egui_docs::app::DocsApp;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    let mut app = DocsApp::default();
    app.select_lesson("button");
    let mut harness = Harness::new_ui_state(|ui, app: &mut DocsApp| app.show(ui), app);
    harness.run();
    harness.get_by_label("Click me");

    assert!(harness.state_mut().set_language("ru"));
    harness.run();
    harness.get_by_label("Нажми меня");
    harness.get_by_label("Копировать");
}
