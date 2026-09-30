//! Lesson: `egui::Id`.
//!
//! The lesson is built around the failure it teaches: three identical headers
//! that share one open/closed state until you give them distinct `Id`s.

use crate::i18n::Tr;
use crate::lesson::{Lesson, Section};

/// The two idiomatic ways out of an id collision.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fix {
    /// No fix — identical labels produce identical `Id`s.
    None,
    /// Override the salt of this one widget.
    IdSalt,
    /// Change the *parent* `Id`, so everything inside becomes distinct.
    PushId,
}

pub struct IdLesson {
    item_count: usize,
    fix: Fix,
    show_ids: bool,
    warn_on_id_clash: bool,
}

impl Default for IdLesson {
    fn default() -> Self {
        Self {
            item_count: 3,
            fix: Fix::None,
            show_ids: true,
            warn_on_id_clash: true,
        }
    }
}

impl IdLesson {
    /// One collapsing header. Returns the `Id` egui actually gave it.
    ///
    /// `salt` mirrors what the demo passes to `.id_salt(..)`; `None` means the
    /// widget falls back to its label, which is the whole point of the lesson.
    fn item(&self, ui: &mut egui::Ui, tr: Tr<'_>, index: usize, salt: Option<usize>) -> egui::Id {
        let mut header = egui::CollapsingHeader::new(tr.get("id-header"));
        if let Some(salt) = salt {
            header = header.id_salt(salt);
        }

        let response = header.show(ui, |ui| {
            ui.label(tr.fmt("id-body", &[("index", index.into())]));
        });

        // Read off the Response. Recomputing it with `ui.make_persistent_id`
        // gives a *different* value: a container derives its widget Id inside
        // its own child Ui, not the one you hand it.
        response.header_response.id
    }
}

impl Lesson for IdLesson {
    fn id(&self) -> &'static str {
        "id"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn demo(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        // Scoped to this demo: the knob must not leak into other lessons.
        let previous_warn = ui.ctx().options(|options| options.warn_on_id_clash);
        ui.ctx()
            .options_mut(|options| options.warn_on_id_clash = self.warn_on_id_clash);

        let mut ids = Vec::with_capacity(self.item_count);
        for index in 0..self.item_count {
            let id = match self.fix {
                Fix::None => self.item(ui, tr, index, None),
                Fix::IdSalt => self.item(ui, tr, index, Some(index)),
                Fix::PushId => {
                    // A new parent Id for this iteration: every Id derived inside
                    // is now distinct, without touching the widgets themselves.
                    ui.push_id(index, |ui| self.item(ui, tr, index, None)).inner
                }
            };
            ids.push(id);

            // egui paints its id-clash warning just below the offending widget,
            // where it would otherwise cover the next header. Constant spacing
            // (not only when clashing) keeps the layout from jumping when the
            // fix is switched on and off.
            ui.add_space(18.0);
        }

        ui.ctx()
            .options_mut(|options| options.warn_on_id_clash = previous_warn);

        if self.show_ids {
            ui.add_space(8.0);
            // `Id` is Hash + Eq but not Ord, so a HashSet is the way to count distinct ones.
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            let listed = ids
                .iter()
                .map(|id| id.short_debug_format())
                .collect::<Vec<_>>()
                .join("   ");
            ui.label(
                egui::RichText::new(tr.fmt("id-header-ids", &[("ids", listed.into())]))
                    .monospace()
                    .color(if unique.len() == ids.len() {
                        ui.visuals().strong_text_color()
                    } else {
                        ui.visuals().error_fg_color
                    }),
            );
            ui.label(
                egui::RichText::new(tr.fmt(
                    "id-distinct",
                    &[("ids", unique.len().into()), ("widgets", ids.len().into())],
                ))
                .weak(),
            );
        }

        ui.add_space(8.0);
        if self.fix == Fix::None {
            ui.label(egui::RichText::new(tr.get("id-dead")).italics());
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        ui.label(tr.get("id-items"));
        ui.add(egui::Slider::new(&mut self.item_count, 1..=5));
        ui.add_space(8.0);

        ui.label(tr.get("id-fix"));
        ui.selectable_value(&mut self.fix, Fix::None, tr.get("id-fix-none"));
        ui.selectable_value(&mut self.fix, Fix::IdSalt, ".id_salt(index)");
        ui.selectable_value(&mut self.fix, Fix::PushId, "ui.push_id(index, ..)");
        ui.add_space(8.0);

        ui.checkbox(&mut self.show_ids, tr.get("id-show-ids"));
        ui.checkbox(&mut self.warn_on_id_clash, "warn_on_id_clash");

        ui.add_space(12.0);
        ui.label(egui::RichText::new(tr.get("id-try")).italics());
    }

    fn code(&self, tr: Tr<'_>) -> String {
        let count = self.item_count;
        let header = format!("{:?}", tr.get("id-header"));
        // The demo's label with Rust's own `{index}` in place of the number,
        // so the snippet formats it the same way the demo does.
        let body = format!("{:?}", tr.fmt("id-body", &[("index", "{index}".into())]));

        match self.fix {
            Fix::None => {
                let warning = tr.fmt("id-code-clash", &[("count", count.into())]);
                let comment = warning
                    .lines()
                    .enumerate()
                    .map(|(i, line)| format!("//{} {line}\n", if i == 0 { " ⚠" } else { "  " }))
                    .collect::<String>();
                format!(
                    "{comment}\
                     for index in 0..{count} {{\n    \
                         egui::CollapsingHeader::new({header}).show(ui, |ui| {{\n        \
                             ui.label(format!({body}));\n    \
                         }});\n\
                     }}"
                )
            }
            Fix::IdSalt => format!(
                "for index in 0..{count} {{\n    \
                     egui::CollapsingHeader::new({header})\n        \
                         .id_salt(index) // {salt}\n        \
                         .show(ui, |ui| {{\n            \
                             ui.label(format!({body}));\n        \
                         }});\n\
                 }}",
                salt = tr.get("id-code-salt"),
            ),
            Fix::PushId => format!(
                "for index in 0..{count} {{\n    \
                     // {push}\n    \
                     ui.push_id(index, |ui| {{\n        \
                         egui::CollapsingHeader::new({header}).show(ui, |ui| {{\n            \
                             ui.label(format!({body}));\n        \
                         }});\n    \
                     }});\n\
                 }}",
                push = tr.get("id-code-push"),
            ),
        }
    }

    fn notes(&self) -> &'static [&'static str] {
        &[
            "address",
            "derived",
            "read-it",
            "one-widget",
            "salt-vs-push",
            "debug-warning",
        ]
    }

    fn references(&self) -> &'static [&'static str] {
        &[
            "egui::Id",
            "egui::Ui::push_id",
            "egui::Options::warn_on_id_clash",
            "crates/egui/src/widget_rect.rs",
        ]
    }
}
