//! Lesson: `egui::Response`.
//!
//! Every widget call returns one. This lesson is a live dump of it: poke the
//! target below and watch which flags light up.

use egui::{Sense, Vec2};

use crate::code::{CodeBuilder, indent};
use crate::lesson::{Lesson, Link, Note, Section};
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

/// One pulse per momentary flag, so each can fade independently.
#[derive(Default)]
struct Pulses {
    clicked: Pulse,
    double_clicked: Pulse,
    triple_clicked: Pulse,
    secondary_clicked: Pulse,
    middle_clicked: Pulse,
    drag_started: Pulse,
    drag_stopped: Pulse,
    gained_focus: Pulse,
    lost_focus: Pulse,
}

pub struct ResponseLesson {
    sense: SenseKind,
    enabled: bool,
    show_geometry: bool,
    pulses: Pulses,
}

impl Default for ResponseLesson {
    fn default() -> Self {
        Self {
            sense: SenseKind::ClickAndDrag,
            enabled: true,
            show_geometry: true,
            pulses: Pulses::default(),
        }
    }
}

impl Lesson for ResponseLesson {
    fn title(&self) -> &'static str {
        "Response"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn summary(&self) -> &'static str {
        "What every widget hands back — and the difference between a state and an event."
    }

    fn demo(&mut self, ui: &mut egui::Ui) {
        let target = egui::Button::new("Poke me")
            .min_size(Vec2::new(220.0, 64.0))
            .sense(self.sense.sense());
        let response = ui.add_enabled(self.enabled, target);

        ui.add_space(12.0);

        // States: true for as long as the condition holds. `state_flag` takes
        // no Pulse — a state stays lit by itself.
        ui.label(egui::RichText::new("states").strong());
        ui.horizontal_wrapped(|ui| {
            state_flag(ui, "hovered", response.hovered());
            state_flag(ui, "contains_pointer", response.contains_pointer());
            state_flag(ui, "dragged", response.dragged());
            state_flag(ui, "has_focus", response.has_focus());
            state_flag(
                ui,
                "is_pointer_button_down_on",
                response.is_pointer_button_down_on(),
            );
        });

        ui.add_space(8.0);

        // Events: true for exactly one frame. `event_flag` *requires* a Pulse,
        // so the fade that makes a one-frame flag visible cannot be forgotten.
        ui.label(egui::RichText::new("events — true for one frame only").strong());
        ui.horizontal_wrapped(|ui| {
            let p = &mut self.pulses;
            event_flag(ui, "clicked", response.clicked(), &mut p.clicked);
            event_flag(
                ui,
                "double_clicked",
                response.double_clicked(),
                &mut p.double_clicked,
            );
            event_flag(
                ui,
                "triple_clicked",
                response.triple_clicked(),
                &mut p.triple_clicked,
            );
            event_flag(
                ui,
                "secondary_clicked",
                response.secondary_clicked(),
                &mut p.secondary_clicked,
            );
            event_flag(
                ui,
                "middle_clicked",
                response.middle_clicked(),
                &mut p.middle_clicked,
            );
            event_flag(
                ui,
                "drag_started",
                response.drag_started(),
                &mut p.drag_started,
            );
            event_flag(
                ui,
                "drag_stopped",
                response.drag_stopped(),
                &mut p.drag_stopped,
            );
            event_flag(
                ui,
                "gained_focus",
                response.gained_focus(),
                &mut p.gained_focus,
            );
            event_flag(ui, "lost_focus", response.lost_focus(), &mut p.lost_focus);
        });

        if self.show_geometry {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("values").strong());
            egui::Grid::new("response_values")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("rect");
                    ui.label(format!(
                        "{:.0} x {:.0}  at ({:.0}, {:.0})",
                        response.rect.width(),
                        response.rect.height(),
                        response.rect.min.x,
                        response.rect.min.y
                    ));
                    ui.end_row();

                    ui.label("drag_delta()");
                    let delta = response.drag_delta();
                    ui.label(format!("({:.1}, {:.1})", delta.x, delta.y));
                    ui.end_row();

                    ui.label("interact_pointer_pos()");
                    match response.interact_pointer_pos() {
                        Some(pos) => ui.label(format!("Some(({:.0}, {:.0}))", pos.x, pos.y)),
                        None => ui.label("None"),
                    };
                    ui.end_row();

                    ui.label("id");
                    ui.label(format!("{:?}", response.id));
                    ui.end_row();
                });
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label("What the widget senses");
        ui.selectable_value(&mut self.sense, SenseKind::Click, "click");
        ui.selectable_value(&mut self.sense, SenseKind::Drag, "drag");
        ui.selectable_value(&mut self.sense, SenseKind::ClickAndDrag, "click_and_drag");
        ui.add_space(8.0);

        ui.checkbox(&mut self.enabled, "enabled");
        ui.checkbox(&mut self.show_geometry, "show values");

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(
                "Try: right-click, double-click, press and drag, then Tab to focus it.",
            )
            .italics(),
        );
    }

    fn code(&self) -> String {
        let mut widget = CodeBuilder::new(r#"egui::Button::new("Poke me")"#);
        widget
            .call(".min_size(egui::vec2(220.0, 64.0))")
            .call(format!(".sense({})", self.sense.code()));

        let widget = indent(&widget.build(), 1);
        let add = if self.enabled {
            format!("let response = ui.add(\n{widget},\n);")
        } else {
            format!("let response = ui.add_enabled(\n    false,\n{widget},\n);")
        };

        format!(
            "{add}\n\n\
             // States — true while the condition holds:\n\
             response.hovered();\n\
             response.dragged();\n\n\
             // Events — true for exactly one frame:\n\
             if response.clicked() {{ /* ... */ }}\n\
             if response.drag_started() {{ /* ... */ }}\n\n\
             // Values:\n\
             response.rect;\n\
             response.drag_delta();\n\
             response.interact_pointer_pos();"
        )
    }

    fn notes(&self) -> &'static [Note] {
        &[
            Note {
                heading: "States vs events",
                body: "`hovered()` is a state: it stays true the whole time the pointer is over \
                       the widget. `clicked()` is an event: it is true for exactly one frame, \
                       then false again. At 60 fps that is 16 ms — which is why the chips above \
                       fade out instead of blinking. If you need to remember an event, you must \
                       store it yourself (a counter, a bool in your app struct); the Response is \
                       gone at the end of the frame.",
            },
            Note {
                heading: "Sense decides what is even possible",
                body: "A widget only reports what it senses. Switch the knob to `drag` and \
                       `clicked()` stops firing no matter how you click — the widget is not \
                       listening for clicks. `Sense` is also what makes a widget eligible for \
                       hover highlighting and focus at all.",
            },
            Note {
                heading: "Disabled widgets still return a Response",
                body: "`add_enabled(false, ..)` does not remove the widget: you still get a \
                       Response back, its `rect` is still valid, but no interaction flag will \
                       ever be true. That is why you can write `if response.clicked()` \
                       unconditionally without checking whether the widget is enabled.",
            },
            Note {
                heading: "The Response is the whole API surface",
                body: "There is no widget object to query later, and no event handler to \
                       register. Everything egui will ever tell you about a widget is in the \
                       struct returned by the call — including `rect` for layout, `id` for \
                       memory, and helpers like `on_hover_text(..)` that consume and return it.",
            },
        ]
    }

    fn links(&self) -> &'static [Link] {
        &[
            Link {
                label: "docs.rs — Response",
                url: "https://docs.rs/egui/0.36.2/egui/struct.Response.html",
            },
            Link {
                label: "docs.rs — Sense",
                url: "https://docs.rs/egui/0.36.2/egui/struct.Sense.html",
            },
            Link {
                label: "source — response.rs",
                url: "https://github.com/emilk/egui/blob/0.36.2/crates/egui/src/response.rs",
            },
        ]
    }
}
