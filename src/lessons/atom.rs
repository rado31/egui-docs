//! Lesson: `egui::Atom`.
//!
//! A Button's content is not "a label": it is a row of atoms laid out inside
//! the button's frame. The lesson puts a button into a width you control and
//! outlines every atom, so `grow` and `shrink` can be watched doing their job.

use egui::AtomExt as _;

use crate::code::{CodeBuilder, color_code, f32_code, indent, line_comments};
use crate::i18n::Tr;
use crate::lesson::{Lesson, Section, retranslate_text};

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
    /// Stable, for the overlay's ids. The name shown to the reader is the
    /// message `atom-part-<name>`.
    fn name(self) -> &'static str {
        match self {
            Self::Icon => "icon",
            Self::Label => "label",
            Self::Grow => "spacer",
            Self::Shortcut => "shortcut",
            Self::Dot => "custom",
        }
    }

    fn display_name(self, tr: Tr<'_>) -> String {
        tr.get(&format!("atom-part-{}", self.name()))
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

impl AtomLesson {
    /// Takes the language for the label, which the reader can edit.
    pub fn new(tr: Tr<'_>) -> Self {
        Self {
            width: 340.0,
            wrap: WrapKind::Default,
            icon: true,
            label: tr.get("atom-label"),
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

fn px_wide(tr: Tr<'_>, width: f32) -> String {
    tr.fmt("atom-px-wide", &[("width", format!("{width:.0}").into())])
}

impl Lesson for AtomLesson {
    fn id(&self) -> &'static str {
        "atom"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn demo(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
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

        ui.label(egui::RichText::new(tr.get("atom-measurements")).strong());
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
                    ui.label(
                        egui::RichText::new(part.display_name(tr))
                            .monospace()
                            .color(color),
                    );
                    ui.label(
                        egui::RichText::new(self.atom_code(*part))
                            .monospace()
                            .weak(),
                    );
                    ui.label(match rect {
                        Some(rect) => px_wide(tr, rect.width()),
                        None => tr.get("atom-nothing-after"),
                    });
                    ui.end_row();
                }

                let rect = response.response.rect;
                ui.label(
                    egui::RichText::new(tr.get("atom-part-button"))
                        .monospace()
                        .color(LIMIT_COLOR),
                );
                ui.label(
                    egui::RichText::new(tr.fmt(
                        "atom-offered",
                        &[("width", format!("{:.0}", self.width).into())],
                    ))
                    .monospace()
                    .weak(),
                );
                let wanted = response
                    .response
                    .intrinsic_size()
                    .map_or(String::new(), |size| {
                        tr.fmt(
                            "atom-would-like",
                            &[("width", format!("{:.0}", size.x).into())],
                        )
                    });
                ui.label(format!("{}{wanted}", px_wide(tr, rect.width())));
                ui.end_row();
            });
    }

    fn controls(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        ui.label(tr.get("atom-width-offered"));
        ui.add(
            egui::Slider::new(&mut self.width, 80.0..=480.0)
                .step_by(10.0)
                .suffix(" px"),
        );
        ui.add_space(8.0);

        ui.label(tr.get("atom-atoms"));
        ui.checkbox(&mut self.icon, Part::Icon.display_name(tr));
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
        ui.checkbox(&mut self.shortcut, Part::Shortcut.display_name(tr));
        ui.checkbox(&mut self.dot, "Atom::custom(..)");
        ui.add_space(8.0);

        ui.label(tr.get("atom-wrap-mode"));
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.wrap,
                WrapKind::Default,
                tr.get("atom-wrap-default"),
            );
            ui.selectable_value(&mut self.wrap, WrapKind::Truncate, "truncate");
            ui.selectable_value(&mut self.wrap, WrapKind::Wrap, "wrap");
            ui.selectable_value(&mut self.wrap, WrapKind::Extend, "extend");
        });
        ui.add_space(8.0);

        ui.checkbox(&mut self.show_outlines, tr.get("atom-outlines"));

        ui.add_space(12.0);
        ui.label(egui::RichText::new(tr.get("atom-try")).italics());
    }

    fn code(&self, tr: Tr<'_>) -> String {
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
                 // {atom_ui}\n\
                 let response = {button};\n\n\
                 if let Some(rect) = response.rect(dot) {{\n    \
                     ui.painter().circle_filled(rect.center(), {radius}, {color});\n\
                 }}",
                width = f32_code(self.width),
                radius = f32_code(DOT_SIZE / 2.0),
                color = color_code(STATUS_COLOR),
                atom_ui = tr.get("atom-code-atom-ui"),
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
            out.push_str(&format!(
                "use egui::AtomExt as _; // {}\n\n",
                tr.get("atom-code-ext")
            ));
        }
        if self.dot {
            out.push_str("let dot = egui::Id::new(\"status_dot\");\n\n");
        }
        out.push_str(&line_comments(&tr.get("atom-code-justified")));
        out.push_str(&format!(
            "ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {{\n\
             {}\n\
             }});",
            indent(&body, 1)
        ));
        out
    }

    fn notes(&self) -> &'static [&'static str] {
        &[
            "everything-atom",
            "grow",
            "shrink",
            "wrap-mode-note",
            "custom",
            "screen-reader",
        ]
    }

    fn retranslate(&mut self, old: Tr<'_>, new: Tr<'_>) {
        retranslate_text(&mut self.label, "atom-label", old, new);
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
