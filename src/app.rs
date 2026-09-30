use egui::gui_zoom::kb_shortcuts;
use egui_extras::syntax_highlighting::{CodeTheme, code_view_ui};

use crate::i18n::{I18n, Tr};
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
fn zoom_controls(ui: &mut egui::Ui, tr: Tr<'_>) {
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
        .on_hover_text(tr.fmt(
            "app-zoom-in",
            &[(
                "shortcut",
                ctx.format_shortcut(&kb_shortcuts::ZOOM_IN).into(),
            )],
        ))
        .clicked()
        || pressed_in
    {
        wanted = zoom + ZOOM_STEP;
    }

    let percent = (zoom * 100.0).round();
    if ui
        .add(egui::Button::new(format!("{percent:.0}%")).frame(false))
        .on_hover_text(tr.fmt(
            "app-zoom-reset",
            &[
                ("size", format!("{:.0}", 13.0 * zoom).into()),
                (
                    "shortcut",
                    ctx.format_shortcut(&kb_shortcuts::ZOOM_RESET).into(),
                ),
            ],
        ))
        .clicked()
        || pressed_reset
    {
        wanted = MIN_ZOOM;
    }

    if ui
        .add_enabled(zoom > MIN_ZOOM, egui::Button::new("A\u{2212}"))
        .on_hover_text(tr.fmt(
            "app-zoom-out",
            &[(
                "shortcut",
                ctx.format_shortcut(&kb_shortcuts::ZOOM_OUT).into(),
            )],
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

/// Dark/light toggle. egui's own `global_theme_preference_switch` does the
/// same, but its tooltip is hard-coded English.
fn theme_switch(ui: &mut egui::Ui, tr: Tr<'_>) {
    let (icon, tooltip, next) = match ui.ctx().theme() {
        egui::Theme::Dark => ("☀", "app-theme-light", egui::Theme::Light),
        egui::Theme::Light => ("🌙", "app-theme-dark", egui::Theme::Dark),
    };
    if ui
        .add(egui::Button::new(icon).frame(false))
        .on_hover_text(tr.get(tooltip))
        .clicked()
    {
        ui.ctx().set_theme(next);
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
    i18n: I18n,
    lessons: Vec<Box<dyn Lesson>>,
    selected: usize,
}

impl DocsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // We handle Cmd +/-/0 ourselves in `zoom_controls`, clamped to a range
        // that cannot make the guide unreadably small.
        cc.egui_ctx
            .options_mut(|options| options.zoom_with_keyboard = false);

        let mut app = Self::default();
        // Start in the reader's own language when there is a translation for
        // it. Read fresh on every launch — nothing is persisted.
        let system = sys_locale::get_locale().and_then(|locale| app.i18n.best_match(&locale));
        if let Some(code) = system {
            app.set_language(code);
        }
        app
    }

    /// Switch language, keeping every lesson's state. Returns `false` for a
    /// language there is no translation for.
    pub fn set_language(&mut self, code: &str) -> bool {
        let old = self.i18n.tr().language();
        if !self.i18n.set_language(code) {
            return false;
        }
        if let Some(old) = self.i18n.tr_in(old) {
            let new = self.i18n.tr();
            for lesson in &mut self.lessons {
                lesson.retranslate(old, new);
            }
        }
        true
    }

    pub fn i18n(&self) -> &I18n {
        &self.i18n
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;

        egui::Panel::top("top_bar").show(ui, |ui| {
            let tr = self.i18n.tr();
            ui.horizontal(|ui| {
                ui.heading("egui-docs");
                ui.label(format!("egui {EGUI_VERSION}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme_switch(ui, tr);
                    ui.separator();
                    zoom_controls(ui, tr);
                    ui.separator();

                    // Every language is listed by its own name, so a reader
                    // can find theirs without reading the current one.
                    let name = |code: &str| {
                        self.i18n
                            .tr_in(code)
                            .map_or_else(|| code.to_owned(), |tr| tr.get("language-name"))
                    };
                    let current = tr.language();
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(name(current))
                        .show_ui(ui, |ui| {
                            for code in self.i18n.codes() {
                                if ui.selectable_label(code == current, name(code)).clicked() {
                                    chosen = Some(code);
                                }
                            }
                        })
                        .response
                        .on_hover_text(tr.get("app-language"));
                });
            });
        });

        if let Some(code) = chosen {
            self.set_language(code);
        }
    }

    /// Table of contents. Sections with no lessons yet are still listed, so the
    /// plan for the guide stays visible.
    fn table_of_contents(&mut self, ui: &mut egui::Ui) {
        let tr = self.i18n.tr();
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

                        let title = tr.get(section.title_id());
                        if lessons.is_empty() {
                            ui.add_enabled(false, egui::Label::new(title));
                            continue;
                        }

                        ui.label(egui::RichText::new(title).strong());
                        ui.indent(section.title_id(), |ui| {
                            for index in lessons {
                                let selected = index == self.selected;
                                if ui
                                    .selectable_label(selected, self.lessons[index].title(tr))
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
        let tr = self.i18n.tr();
        let lesson = &mut self.lessons[self.selected];

        egui::Panel::right("controls")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.heading(tr.get("app-settings"));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    lesson.controls(ui, tr);
                });
            });
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        let tr = self.i18n.tr();
        let lesson = &mut self.lessons[self.selected];

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(lesson.title(tr));
                ui.label(lesson.summary(tr));
                ui.add_space(12.0);

                // Live widget.
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    lesson.demo(ui, tr);
                });

                ui.add_space(12.0);

                // The code for exactly what is shown above.
                ui.label(egui::RichText::new(tr.get("app-code")).strong());
                let code = lesson.code(tr);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
                    code_view_ui(ui, &theme, &code, "rs");
                });
                if ui.button(tr.get("app-copy")).clicked() {
                    ui.ctx().copy_text(code);
                }

                ui.add_space(16.0);

                for note in lesson.notes() {
                    let id = format!("{}-{note}", lesson.id());
                    ui.label(egui::RichText::new(tr.get(&id)).strong());
                    ui.label(prose(ui, &tr.get(&format!("{id}.body"))));
                    ui.add_space(10.0);
                }

                let references = lesson.references();
                if !references.is_empty() {
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label(tr.get("app-look-up"));
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

    /// Jump to a lesson by its [`Lesson::id`]. Used by tests.
    pub fn select_lesson(&mut self, id: &str) {
        if let Some(index) = self.lessons.iter().position(|l| l.id() == id) {
            self.selected = index;
        }
    }

    /// Ids of every lesson, in sidebar order. Used by tests.
    pub fn lesson_ids(&self) -> Vec<&'static str> {
        self.lessons.iter().map(|lesson| lesson.id()).collect()
    }
}

/// Always English, whatever the system says — which keeps the snapshot tests
/// independent of the machine they run on. [`DocsApp::new`] is what follows
/// the system language.
impl Default for DocsApp {
    fn default() -> Self {
        let i18n = I18n::new();
        let lessons = lessons::all(i18n.tr());
        Self {
            i18n,
            lessons,
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
