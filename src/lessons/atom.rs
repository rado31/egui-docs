//! Lesson: `egui::Atom`.
//!
//! A Button's content is not "a label": it is a row of atoms laid out inside
//! the button's frame. The lesson puts a button into a width you control and
//! outlines every atom, so `grow` and `shrink` can be watched doing their job.

use egui::AtomExt as _;

use crate::code::{CodeBuilder, color_code, f32_code, indent};
use crate::lesson::{Lesson, Note, Section};

/// Overlay colours, one per atom. Fixed rather than themed for the same reason
/// as in the Ui lesson: annotations have to stay distinct in both themes.
const ICON_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 140, 245);
const LABEL_COLOR: egui::Color32 = egui::Color32::from_rgb(35, 165, 105);
const GROW_COLOR: egui::Color32 = egui::Color32::from_rgb(205, 135, 25);
const SHORTCUT_COLOR: egui::Color32 = egui::Color32::from_rgb(170, 90, 210);
const DOT_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 70, 95);
/// The width the button is placed into.
const LIMIT_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 140, 150);

/// What the status dot is painted with. Part of the demo, not the overlay.
const STATUS_COLOR: egui::Color32 = egui::Color32::from_rgb(60, 180, 90);
const DOT_SIZE: f32 = 10.0;

const ICON: &str = "🗀";
const SHORTCUT: &str = "Ctrl+O";

#[derive(Clone, Copy, PartialEq, Eq)]
enum WrapKind {
    /// No call — the `Ui` decides.
    Default,
    Truncate,
    Wrap,
    Extend,
}

impl WrapKind {
    fn call(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Truncate => Some(".truncate()"),
            Self::Wrap => Some(".wrap()"),
            Self::Extend => Some(".wrap_mode(egui::TextWrapMode::Extend)"),
        }
    }
}

/// One atom of the button, in order. `demo` and `code` both walk the same
/// list, so they cannot disagree about what the button contains.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Icon,
    Label,
    Grow,
    Shortcut,
    Dot,
}

impl Part {
    fn name(self) -> &'static str {
        match self {
            Self::Icon => "icon",
            Self::Label => "label",
            Self::Grow => "spacer",
            Self::Shortcut => "shortcut",
            Self::Dot => "custom",
        }
    }

    fn color(self) -> egui::Color32 {
        match self {
            Self::Icon => ICON_COLOR,
            Self::Label => LABEL_COLOR,
            Self::Grow => GROW_COLOR,
            Self::Shortcut => SHORTCUT_COLOR,
            Self::Dot => DOT_COLOR,
        }
    }

    /// Where this atom's rect is reported. The dot's id is the one the taught
    /// code uses; the others exist only so the overlay can find them.
    fn id(self) -> egui::Id {
        match self {
            Self::Dot => egui::Id::new("status_dot"),
            other => egui::Id::new(("atom_lesson_overlay", other.name())),
        }
    }
}

pub struct AtomLesson {
    width: f32,
    wrap: WrapKind,

    icon: bool,
    label: String,
    label_shrink: bool,
    use_label_max_width: bool,
    label_max_width: f32,
    grow: bool,
    shortcut: bool,
    dot: bool,

    show_outlines: bool,
}

impl Default for AtomLesson {
    fn default() -> Self {
        Self {
            width: 340.0,
            wrap: WrapKind::Default,
            icon: true,
            label: "Quarterly report – final.pdf".to_owned(),
            label_shrink: true,
            use_label_max_width: false,
            label_max_width: 120.0,
            grow: true,
            shortcut: true,
            dot: false,
            show_outlines: true,
        }
    }
}

impl AtomLesson {
    fn parts(&self) -> Vec<Part> {
        [
            (self.icon, Part::Icon),
            (true, Part::Label),
            (self.grow, Part::Grow),
            (self.shortcut, Part::Shortcut),
            (self.dot, Part::Dot),
        ]
        .into_iter()
        .filter_map(|(on, part)| on.then_some(part))
        .collect()
    }

    fn label_has_ext(&self) -> bool {
        self.label_shrink || self.use_label_max_width
    }

    fn atom(&self, part: Part) -> egui::Atom<'_> {
        match part {
            Part::Icon => ICON.into(),
            Part::Label => {
                let mut atom: egui::Atom<'_> = self.label.as_str().into();
                if self.label_shrink {
                    atom = atom.atom_shrink(true);
                }
                if self.use_label_max_width {
                    atom = atom.atom_max_width(self.label_max_width);
                }
                atom
            }
            Part::Grow => egui::Atom::grow(),
            Part::Shortcut => egui::RichText::new(SHORTCUT).weak().into(),
            Part::Dot => egui::Atom::custom(part.id(), egui::Vec2::splat(DOT_SIZE)),
        }
    }

    fn atom_code(&self, part: Part) -> String {
        match part {
            Part::Icon => format!("{ICON:?}"),
            Part::Label => {
                let mut code = format!("{:?}", self.label);
                if self.label_shrink {
                    code.push_str(".atom_shrink(true)");
                }
                if self.use_label_max_width {
                    code.push_str(&format!(
                        ".atom_max_width({})",
                        f32_code(self.label_max_width)
                    ));
                }
                code
            }
            Part::Grow => "egui::Atom::grow()".to_owned(),
            Part::Shortcut => format!("egui::RichText::new({SHORTCUT:?}).weak()"),
            Part::Dot => format!(
                "egui::Atom::custom(dot, egui::Vec2::splat({}))",
                f32_code(DOT_SIZE)
            ),
        }
    }

    /// The button, built from the current knobs.
    ///
    /// With `overlay`, every atom also gets an `atom_id` so its rect comes back
    /// in the response. That changes nothing about the layout — an id only asks
    /// egui to *report* where the atom ended up.
    fn button(&self, overlay: bool) -> egui::Button<'_> {
        let mut atoms = egui::Atoms::default();
        for part in self.parts() {
            let atom = self.atom(part);
            atoms.push_right(if overlay {
                atom.atom_id(part.id())
            } else {
                atom
            });
        }

        let button = egui::Button::new(atoms);
        match self.wrap {
            WrapKind::Default => button,
            WrapKind::Truncate => button.truncate(),
            WrapKind::Wrap => button.wrap(),
            WrapKind::Extend => button.wrap_mode(egui::TextWrapMode::Extend),
        }
    }

    /// Every part's rect, in order. `Atom::grow()` is empty, so egui reports it
    /// as a zero-size point in the middle of its cell; its real extent is the
    /// room between its neighbours, less one gap on each side.
    fn rects(
        &self,
        ui: &egui::Ui,
        response: &egui::AtomLayoutResponse,
    ) -> Vec<(Part, Option<egui::Rect>)> {
        let parts = self.parts();
        let reported: Vec<Option<egui::Rect>> =
            parts.iter().map(|part| response.rect(part.id())).collect();
        let gap = ui.spacing().icon_spacing;

        parts
            .iter()
            .enumerate()
            .map(|(index, &part)| {
                if part != Part::Grow {
                    return (part, reported[index]);
                }
                let before = index.checked_sub(1).and_then(|i| reported[i]);
                let after = reported.get(index + 1).copied().flatten();
                let rect = match (before, after) {
                    (Some(before), Some(after)) => {
                        let left = before.right() + gap;
                        let right = (after.left() - gap).max(left);
                        // As tall as its neighbours, so it reads as part of the row.
                        let top = before.top().min(after.top());
                        let bottom = before.bottom().max(after.bottom());
                        Some(egui::Rect::from_x_y_ranges(left..=right, top..=bottom))
                    }
                    // Nothing after it: it still takes the slack, but there is
                    // nothing it visibly pushes, and no edge to measure it by.
                    _ => None,
                };
                (part, rect)
            })
            .collect()
    }
}

impl Lesson for AtomLesson {
    fn title(&self) -> &'static str {
        "Atom"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn summary(&self) -> &'static str {
        "What a Button is made of: a row of atoms that grow, shrink, or leave room for you."
    }

    fn demo(&mut self, ui: &mut egui::Ui) {
        let origin = ui.next_widget_position();

        let response = ui
            .with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                ui.set_max_width(self.width);
                let response = self.button(self.show_outlines).atom_ui(ui);

                if let Some(rect) = response.rect(Part::Dot.id()) {
                    ui.painter()
                        .circle_filled(rect.center(), DOT_SIZE / 2.0, STATUS_COLOR);
                }
                response
            })
            .inner;

        let rects = self.rects(ui, &response);

        if self.show_outlines {
            // The width the button was offered. With `Extend` the button can
            // run past it — this line is how that shows.
            let limit_x = origin.x + self.width;
            let rect = response.response.rect;
            ui.painter().extend(egui::Shape::dashed_line(
                &[
                    egui::pos2(limit_x, rect.top() - 6.0),
                    egui::pos2(limit_x, rect.bottom() + 6.0),
                ],
                egui::Stroke::new(1.5, LIMIT_COLOR),
                4.0,
                3.0,
            ));

            for (part, rect) in &rects {
                if let Some(rect) = rect {
                    let stroke = egui::Stroke::new(1.5, part.color());
                    if *part == Part::Grow {
                        ui.painter()
                            .rect_filled(*rect, 0, part.color().gamma_multiply(0.2));
                    }
                    ui.painter()
                        .rect_stroke(*rect, 0, stroke, egui::StrokeKind::Outside);
                }
            }
        }

        ui.add_space(14.0);

        ui.label(
            egui::RichText::new("Atoms, left to right — each name in the colour of its outline")
                .strong(),
        );
        ui.add_space(4.0);

        egui::Grid::new("atom_measurements")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for (part, rect) in &rects {
                    let color = if self.show_outlines {
                        part.color()
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.label(egui::RichText::new(part.name()).monospace().color(color));
                    ui.label(
                        egui::RichText::new(self.atom_code(*part))
                            .monospace()
                            .weak(),
                    );
                    ui.label(match rect {
                        Some(rect) => format!("{:.0} px wide", rect.width()),
                        None => "— nothing after it to push".to_owned(),
                    });
                    ui.end_row();
                }

                let rect = response.response.rect;
                ui.label(egui::RichText::new("button").monospace().color(LIMIT_COLOR));
                ui.label(
                    egui::RichText::new(format!("offered {:.0} px", self.width))
                        .monospace()
                        .weak(),
                );
                let wanted = response
                    .response
                    .intrinsic_size()
                    .map_or(String::new(), |size| format!(", would like {:.0}", size.x));
                ui.label(format!("{:.0} px wide{wanted}", rect.width()));
                ui.end_row();
            });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Width offered to the button");
        ui.add(
            egui::Slider::new(&mut self.width, 80.0..=480.0)
                .step_by(10.0)
                .suffix(" px"),
        );
        ui.add_space(8.0);

        ui.label("Atoms");
        ui.checkbox(&mut self.icon, "icon");
        ui.text_edit_singleline(&mut self.label);
        ui.indent("label_atom", |ui| {
            ui.checkbox(&mut self.label_shrink, "atom_shrink(true)");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.use_label_max_width, "atom_max_width");
                ui.add_enabled(
                    self.use_label_max_width,
                    egui::DragValue::new(&mut self.label_max_width)
                        .range(20.0..=400.0)
                        .suffix(" px"),
                );
            });
        });
        ui.checkbox(&mut self.grow, "Atom::grow()");
        ui.checkbox(&mut self.shortcut, "shortcut");
        ui.checkbox(&mut self.dot, "Atom::custom(..)");
        ui.add_space(8.0);

        ui.label("Wrap mode");
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.wrap, WrapKind::Default, "Ui's");
            ui.selectable_value(&mut self.wrap, WrapKind::Truncate, "truncate");
            ui.selectable_value(&mut self.wrap, WrapKind::Wrap, "wrap");
            ui.selectable_value(&mut self.wrap, WrapKind::Extend, "extend");
        });
        ui.add_space(8.0);

        ui.checkbox(&mut self.show_outlines, "outline atoms");

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Narrow the width to 200 px, then turn off atom_shrink on the \
                 label: egui picks the first text atom to shrink instead — the \
                 icon, which cannot.",
            )
            .italics(),
        );
    }

    fn code(&self) -> String {
        let parts = self.parts();
        let atoms: Vec<String> = parts.iter().map(|&part| self.atom_code(part)).collect();

        // A single atom needs no tuple.
        let content = if let [only] = atoms.as_slice() {
            only.clone()
        } else {
            let lines: Vec<String> = atoms.iter().map(|atom| format!("{atom},")).collect();
            format!("(\n{}\n)", indent(&lines.join("\n"), 1))
        };

        let mut calls: Vec<&str> = self.wrap.call().into_iter().collect();
        if self.dot {
            calls.push(".atom_ui(ui)");
        }
        // Chained calls after a multi-line head sit at the head's own indent,
        // as rustfmt puts them; after a one-line head, one level in.
        let button = if content.contains('\n') {
            std::iter::once(format!("egui::Button::new({content})"))
                .chain(calls.iter().map(|call| (*call).to_owned()))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            let mut builder = CodeBuilder::new(format!("egui::Button::new({content})"));
            for call in &calls {
                builder.call(*call);
            }
            builder.build()
        };

        let body = if self.dot {
            format!(
                "ui.set_max_width({width});\n\n\
                 // `atom_ui` instead of `ui.add`: it hands back each custom atom's rect.\n\
                 let response = {button};\n\n\
                 if let Some(rect) = response.rect(dot) {{\n    \
                     ui.painter().circle_filled(rect.center(), {radius}, {color});\n\
                 }}",
                width = f32_code(self.width),
                radius = f32_code(DOT_SIZE / 2.0),
                color = color_code(STATUS_COLOR),
            )
        } else {
            format!(
                "ui.set_max_width({});\n\nui.add(\n{},\n);",
                f32_code(self.width),
                indent(&button, 1)
            )
        };

        let mut out = String::new();
        if self.label_has_ext() {
            out.push_str("use egui::AtomExt as _; // the .atom_*() methods\n\n");
        }
        if self.dot {
            out.push_str("let dot = egui::Id::new(\"status_dot\");\n\n");
        }
        out.push_str(&format!(
            "// Justified, so the button fills the width — which gives grow\n\
             // something to fill and shrink something to fit into.\n\
             ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {{\n\
             {}\n\
             }});",
            indent(&body, 1)
        ));
        out
    }

    fn notes(&self) -> &'static [Note] {
        &[
            Note {
                heading: "Everything inside a Button is an Atom",
                body: "`Button::new` takes `impl IntoAtoms`. A `&str`, a `RichText`, an `Image` \
                       or an `Atom` is one atom; a tuple of up to six of them is a row. Button, \
                       Checkbox, RadioButton and menu buttons are all built on the same \
                       `AtomLayout`, so what you learn here applies to all of them. Even \
                       `.shortcut_text(..)` is nothing special: it is `push_right(Atom::grow())` \
                       followed by the text, made weak.",
            },
            Note {
                heading: "grow takes the slack — if there is any",
                body: "Space left over after every atom is sized is split equally between the \
                       atoms marked `grow`. But a Button is normally only as wide as its \
                       content, so there is no slack and `grow` does nothing. It needs a width \
                       from outside: a justified layout, as here, or `.min_size(..)` on the \
                       button. Without one, `Atom::grow()` is a silent no-op.",
            },
            Note {
                heading: "Exactly one atom shrinks — egui picks one if you do not",
                body: "When the atoms do not fit, one atom gives up space: the one marked \
                       `atom_shrink(true)`. It is sized last, with whatever width the others \
                       left. Mark none and egui marks the *first text atom* for you. In this \
                       button that is the icon — a single glyph, which cannot get any narrower. \
                       So nothing gives, and the button runs past the dashed line as if it could \
                       not shrink at all. Marking two is a bug: a debug assertion, and in release \
                       only the first counts.",
            },
            Note {
                heading: "The wrap mode decides what shrinking means",
                body: "`.truncate()` cuts the shrinking atom with an ellipsis, `.wrap()` breaks \
                       it onto more lines and makes the button taller, and `Extend` never \
                       shrinks at all — the button runs past the dashed line. With no call the \
                       `Ui` decides: `Wrap` in a vertical layout, but `Extend` in \
                       `ui.horizontal`. The same button behaves differently depending on where \
                       you put it. `atom_max_width` on an atom switches that atom to truncating \
                       regardless.",
            },
            Note {
                heading: "Atom::custom leaves a hole for you to paint in",
                body: "`Atom::custom(id, size)` is an empty atom that takes part in the layout \
                       like any other. Its rect is only known after layout, so show the button \
                       with `.atom_ui(ui)` instead of `ui.add(..)`, and ask the response: \
                       `response.rect(id)`. It returns an Option — `None` when the button was \
                       not visible and so never painted. Never unwrap it. The outlines on this \
                       page are drawn the same way: every atom gets an `.atom_id(..)`, which \
                       asks egui to report its rect and changes nothing else.",
            },
            Note {
                heading: "A screen reader hears every text atom",
                body: "A Button's accessible name is all of its text atoms joined with spaces, \
                       so this one is announced as \"🗀 Quarterly report – final.pdf Ctrl+O\" — \
                       the icon glyph included. An emoji or icon-font glyph is text as far as egui \
                       is concerned. An `Image` atom is not: an icon drawn as an image stays \
                       silent. Its alt text is only used when the button has no text at all.",
            },
        ]
    }

    fn references(&self) -> &'static [&'static str] {
        &[
            "egui::Atom",
            "egui::AtomExt",
            "egui::IntoAtoms",
            "egui::AtomLayout",
            "egui::Button::atom_ui",
            "crates/egui/src/atomics/atom_layout.rs",
        ]
    }
}
