//! Lesson: `egui::Button`.
//!
//! Chosen as the reference lesson because it is the smallest widget that still
//! teaches the core loop of immediate mode: you *call* the widget, and it hands
//! back a `Response` describing what the user did this frame.

use egui::{Color32, Sense, Stroke, Vec2};

use crate::code::{CodeBuilder, color_code, f32_code, indent};
use crate::lesson::{Lesson, Note, Section};
use crate::ui::{Pulse, event_flag, state_flag};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SenseKind {
    Click,
    Drag,
    ClickAndDrag,
}

impl SenseKind {
    fn code(self) -> &'static str {
        match self {
            Self::Click => "egui::Sense::click()",
            Self::Drag => "egui::Sense::drag()",
            Self::ClickAndDrag => "egui::Sense::click_and_drag()",
        }
    }

    fn sense(self) -> Sense {
        match self {
            Self::Click => Sense::click(),
            Self::Drag => Sense::drag(),
            Self::ClickAndDrag => Sense::click_and_drag(),
        }
    }
}

pub struct ButtonLesson {
    label: String,
    enabled: bool,

    frame: bool,
    small: bool,
    selected: bool,

    use_fill: bool,
    fill: Color32,

    use_stroke: bool,
    stroke_width: f32,
    stroke_color: Color32,

    use_corner_radius: bool,
    corner_radius: u8,

    use_min_size: bool,
    min_size: Vec2,

    use_shortcut: bool,
    shortcut: String,

    sense: SenseKind,

    // Runtime state of the demo itself, not a knob.
    clicks: usize,
    click_pulse: Pulse,
}

impl Default for ButtonLesson {
    fn default() -> Self {
        Self {
            label: "Click me".to_owned(),
            enabled: true,
            frame: true,
            small: false,
            selected: false,
            use_fill: false,
            fill: Color32::from_rgb(60, 110, 200),
            use_stroke: false,
            stroke_width: 1.0,
            stroke_color: Color32::WHITE,
            use_corner_radius: false,
            corner_radius: 6,
            use_min_size: false,
            min_size: Vec2::new(120.0, 32.0),
            use_shortcut: false,
            shortcut: "Ctrl+S".to_owned(),
            sense: SenseKind::Click,
            clicks: 0,
            click_pulse: Pulse::default(),
        }
    }
}

impl ButtonLesson {
    /// The widget, built from the current knobs. Both `demo` and `code` are
    /// derived from this same set of fields, which is what keeps them in sync.
    fn button(&self) -> egui::Button<'_> {
        let mut button = egui::Button::new(self.label.as_str());

        if self.small {
            button = button.small();
        }
        if !self.frame {
            button = button.frame(false);
        }
        if self.selected {
            button = button.selected(true);
        }
        if self.use_fill {
            button = button.fill(self.fill);
        }
        if self.use_stroke {
            button = button.stroke(Stroke::new(self.stroke_width, self.stroke_color));
        }
        if self.use_corner_radius {
            button = button.corner_radius(self.corner_radius);
        }
        if self.use_min_size {
            button = button.min_size(self.min_size);
        }
        if self.use_shortcut {
            button = button.shortcut_text(self.shortcut.as_str());
        }
        if self.sense != SenseKind::Click {
            button = button.sense(self.sense.sense());
        }

        button
    }
}

impl Lesson for ButtonLesson {
    fn title(&self) -> &'static str {
        "Button"
    }

    fn section(&self) -> Section {
        Section::Widgets
    }

    fn summary(&self) -> &'static str {
        "A clickable widget — and the shortest path to understanding Response."
    }

    fn demo(&mut self, ui: &mut egui::Ui) {
        let response = ui.add_enabled(self.enabled, self.button());

        if response.clicked() {
            self.clicks += 1;
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            // `clicked()` is true for a single frame, so it gets a Pulse and
            // fades out — see the Response lesson for the full picture.
            event_flag(ui, "clicked()", response.clicked(), &mut self.click_pulse);
            state_flag(ui, "hovered()", response.hovered());

            ui.separator();
            ui.label("clicks:");
            ui.strong(self.clicks.to_string());
            if ui.button("reset").clicked() {
                self.clicks = 0;
            }
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Text");
        ui.text_edit_singleline(&mut self.label);
        ui.add_space(8.0);

        ui.checkbox(&mut self.enabled, "enabled");
        ui.checkbox(&mut self.frame, "frame");
        ui.checkbox(&mut self.small, "small");
        ui.checkbox(&mut self.selected, "selected");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.use_fill, "fill");
            ui.add_enabled_ui(self.use_fill, |ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.fill,
                    egui::color_picker::Alpha::Opaque,
                );
            });
        });

        ui.checkbox(&mut self.use_stroke, "stroke");
        ui.add_enabled_ui(self.use_stroke, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut self.stroke_width)
                        .range(0.0..=8.0)
                        .speed(0.1),
                );
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.stroke_color,
                    egui::color_picker::Alpha::Opaque,
                );
            });
        });

        ui.checkbox(&mut self.use_corner_radius, "corner_radius");
        ui.add_enabled_ui(self.use_corner_radius, |ui| {
            ui.add(egui::Slider::new(&mut self.corner_radius, 0..=32));
        });

        ui.checkbox(&mut self.use_min_size, "min_size");
        ui.add_enabled_ui(self.use_min_size, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut self.min_size.x)
                        .range(0.0..=400.0)
                        .prefix("w "),
                );
                ui.add(
                    egui::DragValue::new(&mut self.min_size.y)
                        .range(0.0..=200.0)
                        .prefix("h "),
                );
            });
        });

        ui.checkbox(&mut self.use_shortcut, "shortcut_text");
        ui.add_enabled_ui(self.use_shortcut, |ui| {
            ui.text_edit_singleline(&mut self.shortcut);
        });

        ui.add_space(8.0);
        ui.label("sense");
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.sense, SenseKind::Click, "click");
            ui.selectable_value(&mut self.sense, SenseKind::Drag, "drag");
            ui.selectable_value(&mut self.sense, SenseKind::ClickAndDrag, "both");
        });
    }

    fn code(&self) -> String {
        let mut widget = CodeBuilder::new(format!("egui::Button::new({:?})", self.label));
        widget
            .call_if(self.small, ".small()")
            .call_if(!self.frame, ".frame(false)")
            .call_if(self.selected, ".selected(true)")
            .call_if(self.use_fill, format!(".fill({})", color_code(self.fill)))
            .call_if(
                self.use_stroke,
                format!(
                    ".stroke(egui::Stroke::new({}, {}))",
                    f32_code(self.stroke_width),
                    color_code(self.stroke_color)
                ),
            )
            .call_if(
                self.use_corner_radius,
                format!(".corner_radius({})", self.corner_radius),
            )
            .call_if(
                self.use_min_size,
                format!(
                    ".min_size(egui::vec2({}, {}))",
                    f32_code(self.min_size.x),
                    f32_code(self.min_size.y)
                ),
            )
            .call_if(
                self.use_shortcut,
                format!(".shortcut_text({:?})", self.shortcut),
            )
            .call_if(
                self.sense != SenseKind::Click,
                format!(".sense({})", self.sense.code()),
            );

        let widget = indent(&widget.build(), 1);

        let add = if self.enabled {
            format!("let response = ui.add(\n{widget},\n);")
        } else {
            format!("let response = ui.add_enabled(\n    false,\n{widget},\n);")
        };

        format!("{add}\n\nif response.clicked() {{\n    self.clicks += 1;\n}}")
    }

    fn notes(&self) -> &'static [Note] {
        &[
            Note {
                heading: "There is no click callback",
                body: "In a retained GUI you register a handler and wait to be called back. \
                       egui has no handler: you call the widget every frame, and it returns a \
                       Response describing what happened to it *this* frame. `clicked()` is \
                       just a bool on that struct. This is the single idea the whole library \
                       is built on — every widget works this way.",
            },
            Note {
                heading: "The widget is a value, not an object",
                body: "`egui::Button::new(..)` builds a plain struct that lives for one frame. \
                       Each `.fill(..)`-style method takes it by value and returns it back, so \
                       they chain. Nothing is stored between frames: turn a knob and the next \
                       frame simply builds a different Button. That is why the code on the left \
                       can be regenerated from state — there is no hidden widget object to sync.",
            },
            Note {
                heading: "add vs add_enabled",
                body: "`ui.add(widget)` shows it normally. `ui.add_enabled(false, widget)` greys \
                       it out and makes it non-interactive — the Response still comes back, but \
                       `clicked()` will never be true. Use `ui.add_enabled_ui(..)` when you want \
                       to disable a whole group at once.",
            },
            Note {
                heading: "Button::new takes Atoms, not just text",
                body: "The signature is `new(impl IntoAtoms)`. An Atom is a piece of button \
                       content — text, an image, a gap — and a string is just the simplest one. \
                       That is what makes `.shortcut_text(..)` and image buttons possible. \
                       Atoms get their own lesson; a plain &str is enough for now.",
            },
        ]
    }

    fn references(&self) -> &'static [&'static str] {
        &[
            "egui::Button",
            "egui::Response",
            "egui::IntoAtoms",
            "crates/egui/src/widgets/button.rs",
        ]
    }
}
