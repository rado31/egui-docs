//! Lesson: `egui::Ui`.
//!
//! (Not to be confused with `crate::ui`, which holds helpers shared by the
//! lessons. This module is the lesson *about* the `Ui` type.)
//!
//! A `Ui` is hard to see, because it draws nothing of its own. So the lesson
//! draws it: the region it was given, the region it has filled, what is left,
//! and where the next widget would land — painted right over a live one.

use crate::code::{f32_code, indent};
use crate::i18n::Tr;
use crate::lesson::{Lesson, Section};

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
    fn id(&self) -> &'static str {
        "ui"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn demo(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        let region = ui.allocate_ui_with_layout(
            egui::vec2(self.width, self.height),
            self.layout.layout(),
            |ui| {
                let mut rows: Vec<egui::Rect> = Vec::new();
                for index in 1..=self.widgets {
                    let response = ui.add(egui::Button::new(
                        tr.fmt("ui-widget", &[("index", index.into())]),
                    ));

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

        ui.label(egui::RichText::new(tr.get("ui-measurements")).strong());
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
                    tr.fmt(
                        "ui-next-offset",
                        &[
                            ("x", format!("{:.0}", measured.next.x - origin.x).into()),
                            ("y", format!("{:.0}", measured.next.y - origin.y).into()),
                        ],
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
                        &tr.get("ui-row-heights"),
                        format!("{} = {total:.0}", heights.join(&spacing)),
                    );
                }
            });
    }

    fn controls(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        ui.label(tr.get("ui-region"));
        ui.add(
            egui::Slider::new(&mut self.width, 120.0..=520.0)
                .step_by(10.0)
                .text(tr.get("ui-width")),
        );
        ui.add(
            egui::Slider::new(&mut self.height, 60.0..=260.0)
                .step_by(10.0)
                .text(tr.get("ui-height")),
        );
        ui.add(egui::Slider::new(&mut self.widgets, 1..=8).text(tr.get("ui-widgets")));

        ui.add_space(10.0);
        ui.label(tr.get("ui-layout"));
        for kind in LayoutKind::ALL {
            ui.selectable_value(&mut self.layout, *kind, kind.label());
        }

        ui.add_space(10.0);
        ui.label(tr.get("ui-draw"));
        ui.checkbox(&mut self.show_max, "max_rect");
        ui.checkbox(&mut self.show_min, "min_rect");
        ui.checkbox(&mut self.show_available, "available");
        ui.checkbox(&mut self.show_cursor, tr.get("ui-next-position"));
        ui.add_enabled(
            self.layout.is_horizontal(),
            egui::Checkbox::new(&mut self.show_rows, tr.get("ui-rows")),
        )
        .on_disabled_hover_text(tr.get("ui-rows-disabled"));

        ui.add_space(12.0);
        ui.label(egui::RichText::new(tr.get("ui-try")).italics());
    }

    fn code(&self, tr: Tr<'_>) -> String {
        let mut body = String::new();
        for index in 1..=self.widgets {
            let label = tr.fmt("ui-widget", &[("index", index.into())]);
            body.push_str(&format!("ui.add(egui::Button::new({label:?}));\n"));
        }

        let reads: &[(bool, &str, String)] = &[
            (self.show_max, "ui.max_rect();", tr.get("ui-code-max")),
            (self.show_min, "ui.min_rect();", tr.get("ui-code-min")),
            (
                self.show_available,
                "ui.available_rect_before_wrap();",
                tr.get("ui-code-before-wrap"),
            ),
            (
                self.show_available,
                "ui.available_size();",
                tr.get("ui-code-available"),
            ),
            (
                self.show_cursor,
                "ui.next_widget_position();",
                tr.fmt("ui-code-next", &[("index", (self.widgets + 1).into())]),
            ),
            (
                self.show_rows && self.layout.is_horizontal(),
                "ui.cursor().y_range();",
                tr.get("ui-code-rows"),
            ),
        ];

        let drawn: Vec<&(bool, &str, String)> = reads.iter().filter(|(on, ..)| *on).collect();
        if !drawn.is_empty() {
            body.push_str(&format!("\n// {}\n", tr.get("ui-code-read-after")));
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

    fn notes(&self) -> &'static [&'static str] {
        &[
            "rect-and-cursor",
            "request-not-limit",
            "min-grows",
            "available",
            "align",
            "child-ui",
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
