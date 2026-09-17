//! Lesson: `egui::Ui`.
//!
//! (Not to be confused with `crate::ui`, which holds helpers shared by the
//! lessons. This module is the lesson *about* the `Ui` type.)
//!
//! A `Ui` is hard to see, because it draws nothing of its own. So the lesson
//! draws it: the region it was given, the region it has filled, what is left,
//! and where the next widget would land — painted right over a live one.

use crate::code::{f32_code, indent};
use crate::lesson::{Lesson, Note, Section};

/// Overlay colours. Fixed rather than taken from the theme: these are
/// annotations on top of the demo, not part of it, and they have to stay
/// distinguishable from each other in both light and dark mode. Mid-tones, for
/// that reason — the first, brighter set was washed out on the light theme's
/// white background (checked by rendering the lesson in both).
const MAX_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 140, 245);
const MIN_COLOR: egui::Color32 = egui::Color32::from_rgb(35, 165, 105);
const AVAILABLE_COLOR: egui::Color32 = egui::Color32::from_rgb(205, 135, 25);
const CURSOR_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 70, 95);
/// Neutral on purpose: rows are structure *inside* the region, not one of the
/// four things the `Ui` reports, so they should sit behind the coloured lines.
const ROW_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 140, 150);

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutKind {
    TopDown,
    LeftToRight,
    Wrapped,
    RightToLeft,
    BottomUp,
}

impl LayoutKind {
    const ALL: &'static [Self] = &[
        Self::TopDown,
        Self::LeftToRight,
        Self::Wrapped,
        Self::RightToLeft,
        Self::BottomUp,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::TopDown => "top_down",
            Self::LeftToRight => "left_to_right",
            Self::Wrapped => "left_to_right + wrap",
            Self::RightToLeft => "right_to_left",
            Self::BottomUp => "bottom_up",
        }
    }

    fn layout(self) -> egui::Layout {
        match self {
            Self::TopDown => egui::Layout::top_down(egui::Align::Min),
            Self::LeftToRight => egui::Layout::left_to_right(egui::Align::Center),
            Self::Wrapped => egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
            Self::RightToLeft => egui::Layout::right_to_left(egui::Align::Center),
            Self::BottomUp => egui::Layout::bottom_up(egui::Align::Min),
        }
    }

    /// Only a horizontal layout has rows. A vertical one has columns, and its
    /// cursor spans to infinity along the main axis, so there is no "row" to
    /// read off it.
    fn is_horizontal(self) -> bool {
        matches!(self, Self::LeftToRight | Self::Wrapped | Self::RightToLeft)
    }

    fn code(self) -> &'static str {
        match self {
            Self::TopDown => "egui::Layout::top_down(egui::Align::Min)",
            Self::LeftToRight => "egui::Layout::left_to_right(egui::Align::Center)",
            Self::Wrapped => {
                "egui::Layout::left_to_right(egui::Align::Center)\n    .with_main_wrap(true)"
            }
            Self::RightToLeft => "egui::Layout::right_to_left(egui::Align::Center)",
            Self::BottomUp => "egui::Layout::bottom_up(egui::Align::Min)",
        }
    }
}

/// What the demo region reports about itself, read *after* the last widget —
/// which is exactly the state the next widget would be placed into.
struct Measured {
    max: egui::Rect,
    min: egui::Rect,
    before_wrap: egui::Rect,
    available: egui::Vec2,
    next: egui::Pos2,
    /// One rect per row of a horizontal layout: the row's height as egui sees
    /// it, spanning the widgets in it. Empty for vertical layouts.
    rows: Vec<egui::Rect>,
}

pub struct UiLesson {
    width: f32,
    height: f32,
    widgets: usize,
    layout: LayoutKind,
    show_max: bool,
    show_min: bool,
    show_available: bool,
    show_cursor: bool,
    show_rows: bool,
}

impl Default for UiLesson {
    fn default() -> Self {
        Self {
            width: 320.0,
            height: 160.0,
            widgets: 3,
            layout: LayoutKind::TopDown,
            show_max: true,
            show_min: true,
            show_available: true,
            show_cursor: true,
            show_rows: true,
        }
    }
}

/// One measured rectangle, drawn over the demo.
///
/// `dashed` distinguishes the region that was *offered* (`max_rect`) from the
/// one that was actually *used* (`min_rect`) — they start out on top of each
/// other, and a solid line over a dashed one is still readable.
///
/// Deliberately unlabelled. A first version wrote the method name into each
/// corner and the four labels promptly landed on top of the widgets and on each
/// other. The table below the demo is the legend instead: each row is printed
/// in this same colour.
fn outline(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32, dashed: bool) {
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.5, color);

    if dashed {
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        painter.extend(egui::Shape::dashed_line(&corners, stroke, 5.0, 4.0));
    } else {
        painter.rect_stroke(rect, 0, stroke, egui::StrokeKind::Inside);
    }
}

impl Lesson for UiLesson {
    fn title(&self) -> &'static str {
        "Ui"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn summary(&self) -> &'static str {
        "A region and a cursor inside it — every widget you add moves that cursor."
    }

    fn demo(&mut self, ui: &mut egui::Ui) {
        let region = ui.allocate_ui_with_layout(
            egui::vec2(self.width, self.height),
            self.layout.layout(),
            |ui| {
                let mut rows: Vec<egui::Rect> = Vec::new();
                for index in 1..=self.widgets {
                    let response = ui.add(egui::Button::new(format!("Widget #{index}")));

                    // In a horizontal layout the cursor's vertical extent *is* the
                    // current row. `response.rect` would not do: it is the button
                    // alone, ~18 px tall, while egui counts the row's full height
                    // as used (`Placer::advance_after_rects` grows `min_rect` by
                    // the widget's frame, not the widget).
                    if self.layout.is_horizontal() {
                        let row_y = ui.cursor().y_range();
                        match rows.last_mut() {
                            Some(row) if row.y_range() == row_y => {
                                *row = row.union(response.rect);
                            }
                            _ => rows
                                .push(egui::Rect::from_x_y_ranges(response.rect.x_range(), row_y)),
                        }
                    }
                }

                Measured {
                    max: ui.max_rect(),
                    min: ui.min_rect(),
                    before_wrap: ui.available_rect_before_wrap(),
                    available: ui.available_size(),
                    next: ui.next_widget_position(),
                    rows,
                }
            },
        );
        let measured = region.inner;

        // `allocate_ui_with_layout` gives back only the space actually used —
        // that is the lesson's own point — but the outlines are drawn over the
        // space that was *asked for*. Reserve the difference, or they are
        // painted over whatever comes next.
        let unused = (measured.max.bottom() - region.response.rect.bottom()).max(0.0);
        ui.add_space(unused);

        // Painted *after* the region is closed, so the annotations land on top
        // of the widgets instead of under them.
        if self.show_rows {
            for row in &measured.rows {
                ui.painter().rect_stroke(
                    *row,
                    0,
                    egui::Stroke::new(1.0, ROW_COLOR),
                    egui::StrokeKind::Inside,
                );
            }
        }

        if self.show_available {
            ui.painter().rect_filled(
                measured.before_wrap,
                0,
                AVAILABLE_COLOR.gamma_multiply(0.15),
            );
            outline(ui, measured.before_wrap, AVAILABLE_COLOR, false);
        }

        if self.show_max {
            outline(ui, measured.max, MAX_COLOR, true);
        }

        if self.show_min {
            outline(ui, measured.min, MIN_COLOR, false);
        }

        if self.show_cursor {
            ui.painter().circle_filled(measured.next, 3.5, CURSOR_COLOR);
        }

        ui.add_space(10.0);

        ui.label(
            egui::RichText::new("Measurements — each name is printed in the colour of its outline")
                .strong(),
        );
        ui.add_space(4.0);

        let origin = measured.max.min;
        let item_spacing = ui.spacing().item_spacing.y;
        egui::Grid::new("ui_measurements")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                let mut row = |color: egui::Color32, name: &str, value: String| {
                    ui.label(egui::RichText::new(name).monospace().color(color));
                    ui.label(value);
                    ui.end_row();
                };

                row(
                    MAX_COLOR,
                    "max_rect()",
                    format!("{:.0} x {:.0}", measured.max.width(), measured.max.height()),
                );
                row(
                    MIN_COLOR,
                    "min_rect()",
                    format!("{:.0} x {:.0}", measured.min.width(), measured.min.height()),
                );
                row(
                    AVAILABLE_COLOR,
                    "available_rect_before_wrap()",
                    format!(
                        "{:.0} x {:.0}",
                        measured.before_wrap.width(),
                        measured.before_wrap.height()
                    ),
                );
                row(
                    AVAILABLE_COLOR,
                    "available_size()",
                    format!("{:.0} x {:.0}", measured.available.x, measured.available.y),
                );
                // Screen coordinates would be a meaningless pair of large
                // numbers that change when the window moves; the offset from
                // the region's own corner is the part worth reading.
                row(
                    CURSOR_COLOR,
                    "next_widget_position()",
                    format!(
                        "+{:.0}, +{:.0}  from max_rect.min",
                        measured.next.x - origin.x,
                        measured.next.y - origin.y
                    ),
                );

                if self.show_rows && !measured.rows.is_empty() {
                    // Spelled out as a sum, so the height of `min_rect()` two
                    // rows up can be checked by eye: rows plus the spacing
                    // between them.
                    let spacing = format!(" + {item_spacing:.0} + ");
                    let heights: Vec<String> = measured
                        .rows
                        .iter()
                        .map(|row| format!("{:.0}", row.height()))
                        .collect();
                    let total = measured.rows.last().map_or(0.0, |r| r.bottom())
                        - measured.rows.first().map_or(0.0, |r| r.top());
                    row(
                        ROW_COLOR,
                        "cursor() row heights",
                        format!("{} = {total:.0}", heights.join(&spacing)),
                    );
                }
            });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Region offered to the Ui");
        ui.add(
            egui::Slider::new(&mut self.width, 120.0..=520.0)
                .step_by(10.0)
                .text("width"),
        );
        ui.add(
            egui::Slider::new(&mut self.height, 60.0..=260.0)
                .step_by(10.0)
                .text("height"),
        );
        ui.add(egui::Slider::new(&mut self.widgets, 1..=8).text("widgets"));

        ui.add_space(10.0);
        ui.label("Layout");
        for kind in LayoutKind::ALL {
            ui.selectable_value(&mut self.layout, *kind, kind.label());
        }

        ui.add_space(10.0);
        ui.label("Draw");
        ui.checkbox(&mut self.show_max, "max_rect");
        ui.checkbox(&mut self.show_min, "min_rect");
        ui.checkbox(&mut self.show_available, "available");
        ui.checkbox(&mut self.show_cursor, "next position");
        ui.add_enabled(
            self.layout.is_horizontal(),
            egui::Checkbox::new(&mut self.show_rows, "rows"),
        )
        .on_disabled_hover_text("Only a horizontal layout has rows.");

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Try: pull the height down to 60 with 5 widgets. Nothing is cut off — \
                 max_rect reports back more than you asked for.",
            )
            .italics(),
        );
    }

    fn code(&self) -> String {
        let next_comment = format!("where widget #{} would start", self.widgets + 1);
        let mut body = String::new();
        for index in 1..=self.widgets {
            body.push_str(&format!(
                "ui.add(egui::Button::new(\"Widget #{index}\"));\n"
            ));
        }

        let reads: &[(bool, &str, &str)] = &[
            (
                self.show_max,
                "ui.max_rect();",
                "the region widgets try to fit in",
            ),
            (self.show_min, "ui.min_rect();", "everything added so far"),
            (
                self.show_available,
                "ui.available_rect_before_wrap();",
                "what is left on this row",
            ),
            (
                self.show_available,
                "ui.available_size();",
                "what is left after a wrap",
            ),
            (
                self.show_cursor,
                "ui.next_widget_position();",
                next_comment.as_str(),
            ),
            (
                self.show_rows && self.layout.is_horizontal(),
                "ui.cursor().y_range();",
                "after each widget: the row it went into",
            ),
        ];

        let drawn: Vec<&(bool, &str, &str)> = reads.iter().filter(|(on, ..)| *on).collect();
        if !drawn.is_empty() {
            body.push_str("\n// Read after the last widget — the state the next one sees:\n");
            for (_, call, comment) in drawn {
                body.push_str(&format!("{call:<33}// {comment}\n"));
            }
        }

        format!(
            "ui.allocate_ui_with_layout(\n    \
                 egui::vec2({}, {}),\n    \
                 {},\n    \
                 |ui| {{\n{}\n    }},\n);",
            f32_code(self.width),
            f32_code(self.height),
            self.layout.code(),
            indent(body.trim_end(), 2),
        )
    }

    fn notes(&self) -> &'static [Note] {
        &[
            Note {
                heading: "A Ui is a rectangle plus a cursor",
                body: "There is no `Ui` object drawn on screen — it is bookkeeping. `max_rect` is \
                       the region it was handed, `min_rect` is the part it has filled so far, and \
                       the cursor sits at the edge between them. Every widget call does the same \
                       three things: ask the placer for space at the cursor inside `max_rect`, \
                       paint into it, then advance the cursor and grow `min_rect` to include what \
                       was just added.",
            },
            Note {
                heading: "max_rect is a request, not a limit",
                body: "Pull the height down until the buttons no longer fit, and watch the table: \
                       `max_rect()` reports back more than the height that was asked for. When \
                       something does not fit, egui expands *both* rectangles and makes the \
                       parent find the room — it would rather overflow than cut a widget off. So \
                       `max_rect` is what widgets *aim* for, never a clip rectangle. Text is the \
                       one thing that truly respects it: a `Label` wraps to `max_rect`'s width, \
                       which is why a narrow region turns a label into a tall column of words.",
            },
            Note {
                heading: "min_rect only ever grows",
                body: "It is the union of every widget added to this `Ui`, so it never shrinks \
                       back — not even if the widget that caused it to grow disappears on the \
                       next frame. That is also what a container returns: `ui.horizontal(..)` \
                       gives you an `InnerResponse` whose `rect` is the child's final `min_rect`, \
                       which is how the parent knows how much space the group actually took.",
            },
            Note {
                heading: "available_size() vs available_rect_before_wrap()",
                body: "In a non-wrapping layout these two agree, and most code can use either. \
                       Switch the layout knob to `left_to_right + wrap` and watch them part ways: \
                       `available_rect_before_wrap()` is what is left *on the current row*, while \
                       `available_size()` reports the full row width — what a widget could get \
                       *after* wrapping onto a fresh row. Ask for the first when deciding whether \
                       something still fits beside the last widget.",
            },
            Note {
                heading: "Align is the cross axis, not the main one",
                body: "`Layout::left_to_right(egui::Align::Center)` moves left to right, and the \
                       `Align` decides where each widget sits *across* that direction — \
                       vertically. Hand a horizontal layout a tall region and the buttons float in \
                       the middle of it, which is exactly what the `left_to_right + wrap` knob \
                       shows: every wrapped row is as tall as the region, and the widgets are \
                       centred in it. `ui.horizontal(..)` uses this same layout, and looks normal \
                       only because it is handed a region one row high.\n\n\
                       It also explains the numbers. With the default height and two wrapped rows, \
                       `min_rect()` is 323 high: 160 + 3 + 160 — two rows as tall as the region, \
                       plus `item_spacing.y` between them. The button is only ~18 px, but egui \
                       counts its whole *frame* as used, and with `Align::Center` the frame is the \
                       full row. Note which scale each number is on: `max_rect()` and \
                       `min_rect()` measure the whole region, while `available_size()` and \
                       `available_rect_before_wrap()` measure only the current row — which is why \
                       they still say 160.",
            },
            Note {
                heading: "Every container hands you a new Ui",
                body: "`ui.horizontal(..)`, `ui.group(..)`, `ui.allocate_ui_with_layout(..)` and \
                       every panel build a *child* `Ui` with its own `max_rect`, its own cursor, \
                       its own `Layout` and its own `Id`. The `ui` inside the closure is not the \
                       `ui` outside it. That is why an Id derived inside a container differs from \
                       one derived outside — see the `Id` lesson — and why setting a width inside \
                       a closure does not affect the parent.",
            },
        ]
    }

    fn references(&self) -> &'static [&'static str] {
        &[
            "egui::Ui",
            "egui::Layout",
            "Ui::allocate_ui_with_layout",
            "crates/egui/src/placer.rs",
        ]
    }
}
