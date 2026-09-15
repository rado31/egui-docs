//! Lesson: `egui::Id`.
//!
//! The lesson is built around the failure it teaches: three identical headers
//! that share one open/closed state until you give them distinct `Id`s.

use crate::lesson::{Lesson, Note, Section};

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
    fn item(&self, ui: &mut egui::Ui, index: usize, salt: Option<usize>) -> egui::Id {
        let mut header = egui::CollapsingHeader::new("Details");
        if let Some(salt) = salt {
            header = header.id_salt(salt);
        }

        let response = header.show(ui, |ui| {
            ui.label(format!("Body of item #{index}"));
        });

        // Read off the Response. Recomputing it with `ui.make_persistent_id`
        // gives a *different* value: a container derives its widget Id inside
        // its own child Ui, not the one you hand it.
        response.header_response.id
    }
}

impl Lesson for IdLesson {
    fn title(&self) -> &'static str {
        "Id"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn summary(&self) -> &'static str {
        "Where a widget's state lives between frames — and what happens when two widgets share it."
    }

    fn demo(&mut self, ui: &mut egui::Ui) {
        // Scoped to this demo: the knob must not leak into other lessons.
        let previous_warn = ui.ctx().options(|options| options.warn_on_id_clash);
        ui.ctx()
            .options_mut(|options| options.warn_on_id_clash = self.warn_on_id_clash);

        let mut ids = Vec::with_capacity(self.item_count);
        for index in 0..self.item_count {
            let id = match self.fix {
                Fix::None => self.item(ui, index, None),
                Fix::IdSalt => self.item(ui, index, Some(index)),
                Fix::PushId => {
                    // A new parent Id for this iteration: every Id derived inside
                    // is now distinct, without touching the widgets themselves.
                    ui.push_id(index, |ui| self.item(ui, index, None)).inner
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
                egui::RichText::new(format!("header Ids:  {listed}"))
                    .monospace()
                    .color(if unique.len() == ids.len() {
                        ui.visuals().strong_text_color()
                    } else {
                        ui.visuals().error_fg_color
                    }),
            );
            ui.label(
                egui::RichText::new(format!(
                    "{} distinct Id(s) for {} widget(s)",
                    unique.len(),
                    ids.len()
                ))
                .weak(),
            );
        }

        ui.add_space(8.0);
        if self.fix == Fix::None {
            ui.label(
                egui::RichText::new(
                    "Only the last header answers the mouse — the first two are dead. \
                     Click it: all three open anyway.",
                )
                .italics(),
            );
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Items");
        ui.add(egui::Slider::new(&mut self.item_count, 1..=5));
        ui.add_space(8.0);

        ui.label("Fix");
        ui.selectable_value(&mut self.fix, Fix::None, "none — collide");
        ui.selectable_value(&mut self.fix, Fix::IdSalt, ".id_salt(index)");
        ui.selectable_value(&mut self.fix, Fix::PushId, "ui.push_id(index, ..)");
        ui.add_space(8.0);

        ui.checkbox(&mut self.show_ids, "show resolved Ids");
        ui.checkbox(&mut self.warn_on_id_clash, "warn_on_id_clash");

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Open one header, then switch the fix on and off. \
                 The state does not move — the Id it is stored under does.",
            )
            .italics(),
        );
    }

    fn code(&self) -> String {
        let count = self.item_count;

        match self.fix {
            Fix::None => format!(
                "// ⚠ Every header gets the same label, and `CollapsingHeader`\n\
                 //   uses the label as its id salt — so all {count} share one Id,\n\
                 //   and therefore one open/closed state.\n\
                 for index in 0..{count} {{\n    \
                     egui::CollapsingHeader::new(\"Details\").show(ui, |ui| {{\n        \
                         ui.label(format!(\"Body of item #{{index}}\"));\n    \
                     }});\n\
                 }}"
            ),
            Fix::IdSalt => format!(
                "for index in 0..{count} {{\n    \
                     egui::CollapsingHeader::new(\"Details\")\n        \
                         .id_salt(index) // replaces the label as the Id source\n        \
                         .show(ui, |ui| {{\n            \
                             ui.label(format!(\"Body of item #{{index}}\"));\n        \
                         }});\n\
                 }}"
            ),
            Fix::PushId => format!(
                "for index in 0..{count} {{\n    \
                     // Changes the parent Id, so every Id derived inside differs.\n    \
                     ui.push_id(index, |ui| {{\n        \
                         egui::CollapsingHeader::new(\"Details\").show(ui, |ui| {{\n            \
                             ui.label(format!(\"Body of item #{{index}}\"));\n        \
                         }});\n    \
                     }});\n\
                 }}"
            ),
        }
    }

    fn notes(&self) -> &'static [Note] {
        &[
            Note {
                heading: "An Id is an address in Memory",
                body: "egui keeps no widget objects, so a widget cannot remember anything by \
                       itself. Whether a header is open, where a ScrollArea is scrolled, which \
                       TextEdit has focus — all of it lives in `Context::memory`, keyed by `Id`. \
                       The Id is how this frame's call finds what last frame's call stored. Two \
                       widgets with the same Id are, as far as egui is concerned, one widget.",
            },
            Note {
                heading: "You usually do not choose the Id",
                body: "Most widgets derive one. `CollapsingHeader::new(label)` uses the label \
                       text as its id salt, and the Id is derived from that salt plus the Id of \
                       the Ui it is added to. That works until the label repeats — or changes, \
                       which silently resets the state, because a different label means a \
                       different Id.",
            },
            Note {
                heading: "Read the Id, do not recompute it",
                body: "The Id above comes from `response.header_response.id`. Deriving it by \
                       hand with `ui.make_persistent_id(\"Details\")` produces a different \
                       value, because a container builds its widgets inside a child Ui with its \
                       own Id — `CollapsingHeader` wraps its contents in a `ui.vertical`. Where \
                       the derivation happens is an implementation detail; the Response is not.",
            },
            Note {
                heading: "One Id means one widget — not two that share",
                body: "egui keeps a registry of widget rects keyed by Id, and a second \
                       registration under the same Id overwrites the first: `existing.rect = \
                       widget_rect.rect; // last wins` (widget_rect.rs). So three clashing \
                       headers leave *one* entry, holding the last one's rectangle. Clicks over \
                       the first two hit nothing — they are not dead in the drawing sense, they \
                       simply no longer exist to hit-testing. Accessibility sees the same thing: \
                       a screen reader is offered one \"Details\" header, not three. And because \
                       that same Id is also the state key, the one header that does respond \
                       toggles all three at once.",
            },
            Note {
                heading: "id_salt vs push_id",
                body: "`.id_salt(x)` replaces the salt of one widget. `ui.push_id(x, ..)` \
                       replaces the parent Id for a whole block, so everything derived inside it \
                       becomes distinct at once. In a loop whose body holds several stateful \
                       widgets, push_id is the one you want: it fixes all of them, and you \
                       cannot forget one.",
            },
            Note {
                heading: "The warning is a debug-build feature",
                body: "`Options::warn_on_id_clash` defaults to `cfg!(debug_assertions)`. In a \
                       release build — including this page, if you are reading it in a browser — \
                       egui says nothing at all about a clash. The widgets just quietly share \
                       state, and the symptom you are left with is a control that does not \
                       respond. Note also that the red overlay is a development aid, not a \
                       readable report: egui paints one label at each of the two clashing \
                       rectangles, per pair, so with three widgets the middle one receives two \
                       different texts in the same place and they overlap. The `distinct Id(s)` \
                       line above is this lesson's own, and is the one to read.",
            },
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
