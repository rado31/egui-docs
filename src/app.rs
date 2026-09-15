use egui::gui_zoom::kb_shortcuts;
use egui_extras::syntax_highlighting::{CodeTheme, code_view_ui};

use crate::lesson::{Lesson, Section};
use crate::lessons;

/// The egui version every lesson in this app documents.
pub const EGUI_VERSION: &str = "0.36.2";

/// Zoom range offered by the toolbar.
///
/// `1.0` is egui's own default (13 px body text). Going *below* the default is
/// left to the keyboard shortcuts only — the toolbar is there to make the text
/// bigger, which is what people actually need.
pub const MIN_ZOOM: f32 = 1.0;
pub const MAX_ZOOM: f32 = 1.5;
const ZOOM_STEP: f32 = 0.1;

/// Browser-style zoom: `A-` / `A+` buttons plus Cmd/Ctrl + `-`, `+`, `0`.
///
/// We zoom the whole UI (`Context::zoom_factor`) rather than only the font
/// sizes: spacing, button padding and icons scale with the text, so nothing
/// gets cramped. This is egui's intended way to do accessibility scaling.
///
/// Lives here rather than in `ui.rs`: it is part of the app shell, called from
/// exactly one place ([`DocsApp::top_bar`]), and no lesson uses it.
fn zoom_controls(ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();

    let pressed_in = ui.input_mut(|i| {
        i.consume_shortcut(&kb_shortcuts::ZOOM_IN)
            || i.consume_shortcut(&kb_shortcuts::ZOOM_IN_SECONDARY)
    });
    let pressed_out = ui.input_mut(|i| i.consume_shortcut(&kb_shortcuts::ZOOM_OUT));
    let pressed_reset = ui.input_mut(|i| i.consume_shortcut(&kb_shortcuts::ZOOM_RESET));

    let zoom = ctx.zoom_factor();
    let mut wanted = zoom;

    // Laid out right-to-left by the caller, so add in reverse visual order.
    if ui
        .add_enabled(zoom < MAX_ZOOM, egui::Button::new("A+"))
        .on_hover_text(format!(
            "Larger text  ({})",
            ctx.format_shortcut(&kb_shortcuts::ZOOM_IN)
        ))
        .clicked()
        || pressed_in
    {
        wanted = zoom + ZOOM_STEP;
    }

    let percent = (zoom * 100.0).round();
    if ui
        .add(egui::Button::new(format!("{percent:.0}%")).frame(false))
        .on_hover_text(format!(
            "Text size — {:.0} px body text.\nReset: {}",
            13.0 * zoom,
            ctx.format_shortcut(&kb_shortcuts::ZOOM_RESET)
        ))
        .clicked()
        || pressed_reset
    {
        wanted = MIN_ZOOM;
    }

    if ui
        .add_enabled(zoom > MIN_ZOOM, egui::Button::new("A\u{2212}"))
        .on_hover_text(format!(
            "Smaller text  ({})",
            ctx.format_shortcut(&kb_shortcuts::ZOOM_OUT)
        ))
        .clicked()
        || pressed_out
    {
        wanted = zoom - ZOOM_STEP;
    }

    let wanted = (wanted.clamp(MIN_ZOOM, MAX_ZOOM) * 10.0).round() / 10.0;
    if wanted != zoom {
        ctx.set_zoom_factor(wanted);
    }
}

/// Lay out a paragraph of lesson prose.
///
/// Two bits of markup are understood: `` `backticks` `` for inline code and
/// `*asterisks*` for emphasis. That is the whole grammar — egui parses no
/// markup by itself, so a body string written with either would otherwise show
/// the punctuation. Building a `LayoutJob` is how several fonts live inside one
/// paragraph: every `append` carries its own `TextFormat`, and the job still
/// wraps as a single unit.
fn prose(ui: &egui::Ui, text: &str) -> egui::text::LayoutJob {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let text_color = ui.visuals().text_color();
    let code_color = ui.visuals().strong_text_color();
    let code_background = ui.visuals().code_bg_color;

    let mut job = egui::text::LayoutJob::default();
    // Without a wrap width the job is laid out on one endless line.
    job.wrap.max_width = ui.available_width();

    let mut push = |piece: &str, is_code: bool, italics: bool| {
        if piece.is_empty() {
            return;
        }
        job.append(
            piece,
            0.0,
            egui::TextFormat {
                font_id: if is_code { mono.clone() } else { body.clone() },
                color: if is_code { code_color } else { text_color },
                background: if is_code {
                    code_background
                } else {
                    egui::Color32::TRANSPARENT
                },
                italics,
                ..Default::default()
            },
        );
    };

    // Odd pieces are what stood between a pair of backticks. Emphasis is only
    // looked for outside them, so an asterisk inside code stays an asterisk.
    for (index, piece) in text.split('`').enumerate() {
        if index % 2 == 1 {
            push(piece, true, false);
        } else {
            for (index, part) in piece.split('*').enumerate() {
                push(part, false, index % 2 == 1);
            }
        }
    }

    job
}

pub struct DocsApp {
    lessons: Vec<Box<dyn Lesson>>,
    selected: usize,
}

impl DocsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // We handle Cmd +/-/0 ourselves in `zoom_controls`, clamped to a range
        // that cannot make the guide unreadably small.
        cc.egui_ctx
            .options_mut(|options| options.zoom_with_keyboard = false);

        Self::default()
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("egui-docs");
                ui.label(format!("egui {EGUI_VERSION}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_switch(ui);
                    ui.separator();
                    zoom_controls(ui);
                });
            });
        });
    }

    /// Table of contents. Sections with no lessons yet are still listed, so the
    /// plan for the guide stays visible.
    fn table_of_contents(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("toc")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for section in Section::ALL {
                        let lessons: Vec<usize> = self
                            .lessons
                            .iter()
                            .enumerate()
                            .filter(|(_, lesson)| lesson.section() == *section)
                            .map(|(index, _)| index)
                            .collect();

                        if lessons.is_empty() {
                            ui.add_enabled(false, egui::Label::new(section.title()));
                            continue;
                        }

                        ui.label(egui::RichText::new(section.title()).strong());
                        ui.indent(section.title(), |ui| {
                            for index in lessons {
                                let selected = index == self.selected;
                                if ui
                                    .selectable_label(selected, self.lessons[index].title())
                                    .clicked()
                                {
                                    self.selected = index;
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                });
            });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let lesson = &mut self.lessons[self.selected];

        egui::Panel::right("controls")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Settings");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    lesson.controls(ui);
                });
            });
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        let lesson = &mut self.lessons[self.selected];

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(lesson.title());
                ui.label(lesson.summary());
                ui.add_space(12.0);

                // Live widget.
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    lesson.demo(ui);
                });

                ui.add_space(12.0);

                // The code for exactly what is shown above.
                ui.label(egui::RichText::new("Code").strong());
                let code = lesson.code();
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
                    code_view_ui(ui, &theme, &code, "rs");
                });
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(code);
                }

                ui.add_space(16.0);

                for note in lesson.notes() {
                    ui.label(egui::RichText::new(note.heading).strong());
                    ui.label(prose(ui, note.body));
                    ui.add_space(10.0);
                }

                let references = lesson.references();
                if !references.is_empty() {
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Look up:");
                        for reference in references {
                            // `Extend` so a name is never broken across lines:
                            // `horizontal_wrapped` moves the whole chip to the
                            // next row instead.
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(*reference)
                                        .monospace()
                                        .background_color(ui.visuals().code_bg_color),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        }
                    });
                }
            });
        });
    }
}

impl DocsApp {
    /// The whole UI, in a form that does not need eframe — so the snapshot
    /// tests can render it headlessly.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Panel order matters: outermost first, CentralPanel always last.
        self.top_bar(ui);
        self.table_of_contents(ui);
        self.controls(ui);
        self.content(ui);
    }

    /// Jump to a lesson by title. Used by tests.
    pub fn select_lesson(&mut self, title: &str) {
        if let Some(index) = self.lessons.iter().position(|l| l.title() == title) {
            self.selected = index;
        }
    }
}

impl Default for DocsApp {
    fn default() -> Self {
        Self {
            lessons: lessons::all(),
            selected: 0,
        }
    }
}

impl eframe::App for DocsApp {
    // NOTE: since egui 0.36 an app is handed a `&mut Ui`, not a `&Context`,
    // and panels are added to that `Ui`. Older tutorials still show
    // `fn update(&mut self, ctx: &Context, ..)` — that signature is gone.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
