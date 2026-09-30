//! Lesson: `eframe::App` — the program around every other lesson.
//!
//! A real eframe app cannot run inside this one, so the demo draws a mock
//! window (or browser tab) and runs the app's `ui` body inside it: the same
//! panels, the same widgets, the same state the snippet declares.

use egui::{Color32, Sense, Vec2};

use crate::code::{CodeBuilder, f32_code, indent, line_comments};
use crate::i18n::Tr;
use crate::lesson::{Lesson, Section, retranslate_text};

/// The side panel's id, in the demo and in the snippet alike.
const SIDE_PANEL_ID: &str = "side_panel";
/// The `<canvas>` the web build draws into. `index.html` has to agree.
const CANVAS_ID: &str = "the_canvas_id";

/// The range the size knobs (and the mock's resize corner) allow, in window
/// points. The mock draws the largest one at the full width of the demo.
const MIN_SIZE: Vec2 = Vec2::new(200.0, 150.0);
const MAX_SIZE: Vec2 = Vec2::new(2000.0, 1500.0);
/// Mock points per window point, at most: 800 × 600 is drawn at 320 × 240.
const MAX_SCALE: f32 = 0.4;
/// What the mock starts at when nothing says otherwise — the OS picks the
/// real window's size, the page picks the canvas's.
const NOMINAL_SIZE: Vec2 = Vec2::new(800.0, 500.0);

/// Which frame the demo draws around the app. A view of the demo, not a knob
/// the snippet reflects — the snippet shows both mains when `web` is on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Preview {
    Window,
    Browser,
}

pub struct EframeLesson {
    app_name: String,

    use_title: bool,
    title: String,

    use_inner_size: bool,
    inner_size: Vec2,

    resizable: bool,
    side_panel: bool,
    web: bool,

    // Demo state, not knobs.
    preview: Preview,
    /// The mock window's current size. `with_inner_size` is only the size it
    /// opens at: dragging the corner changes this, never the code.
    window_size: Vec2,
    /// The inner size the mock was last "launched" with. When the knob moves
    /// away from it, the mock reopens at the new size.
    launched_with: Option<Vec2>,
    /// Whether the reader has dragged the window since it was launched.
    resized: bool,
    /// The browser preview's canvas: resizable whatever the viewport says,
    /// because on the web the page decides.
    canvas_size: Vec2,
    /// The one field of the `MyApp` the snippet declares.
    clicks: usize,
}

impl EframeLesson {
    /// Takes the language for the one piece of state that is text: the window
    /// title, which the reader can edit.
    pub fn new(tr: Tr<'_>) -> Self {
        Self {
            app_name: "my_app".to_owned(),
            use_title: true,
            title: tr.get("eframe-title-text"),
            use_inner_size: true,
            inner_size: Vec2::new(800.0, 600.0),
            resizable: true,
            side_panel: false,
            web: false,
            preview: Preview::Window,
            window_size: Vec2::new(800.0, 600.0),
            launched_with: Some(Vec2::new(800.0, 600.0)),
            resized: false,
            canvas_size: NOMINAL_SIZE,
            clicks: 0,
        }
    }

    /// `MyApp::ui`, run for real. `code` spells out exactly these calls.
    fn app_ui(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        if self.side_panel {
            egui::Panel::left(SIDE_PANEL_ID).show(ui, |ui| {
                ui.label(tr.get("eframe-side-text"));
            });
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(tr.get("eframe-hello"));
            if ui.button(tr.get("eframe-button")).clicked() {
                self.clicks += 1;
            }
            ui.label(format!("{} {}", tr.get("eframe-clicks"), self.clicks));
        });
    }

    /// What the OS would put in the title bar: eframe falls back to the app
    /// name when no title is set.
    fn window_title(&self) -> &str {
        if self.use_title {
            &self.title
        } else {
            &self.app_name
        }
    }

    /// A window or browser tab of the right proportions, with the app inside.
    fn mock(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        let browser = self.preview == Preview::Browser;

        // A new inner size is a relaunch: the window opens at it again.
        let launch = self.use_inner_size.then_some(self.inner_size);
        if launch != self.launched_with {
            self.launched_with = launch;
            self.window_size = launch.unwrap_or(NOMINAL_SIZE);
            self.resized = false;
        }

        // One fixed scale, so the width knob moves only the width and the
        // height knob only the height.
        let scale = (ui.available_width() / MAX_SIZE.x).min(MAX_SCALE);
        let size = if browser {
            self.canvas_size
        } else {
            self.window_size
        };
        let bar_height = 26.0;
        let body_size = size * scale;

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(body_size.x, bar_height + body_size.y),
            Sense::hover(),
        );
        let (bar, body) = rect.split_top_bottom_at_y(rect.top() + bar_height);

        let visuals = ui.visuals().clone();
        let painter = ui.painter();
        let radius = 6;
        painter.rect_filled(
            bar,
            egui::CornerRadius {
                nw: radius,
                ne: radius,
                sw: 0,
                se: 0,
            },
            visuals.widgets.noninteractive.bg_fill,
        );

        let font = egui::TextStyle::Body.resolve(ui.style());
        let small = egui::TextStyle::Small.resolve(ui.style());
        if browser {
            let address = bar.shrink2(egui::vec2(10.0, 4.0));
            painter.rect_filled(address, 4, visuals.extreme_bg_color);
            painter.text(
                address.left_center() + egui::vec2(8.0, 0.0),
                egui::Align2::LEFT_CENTER,
                format!("index.html  <canvas id=\"{CANVAS_ID}\">"),
                small,
                visuals.weak_text_color(),
            );
        } else {
            // The three buttons are decoration: colour alone says "window".
            for (index, color) in [
                Color32::from_rgb(237, 106, 94),
                Color32::from_rgb(245, 191, 79),
                Color32::from_rgb(98, 197, 84),
            ]
            .into_iter()
            .enumerate()
            {
                let center = bar.left_center() + egui::vec2(14.0 + 18.0 * index as f32, 0.0);
                painter.circle_filled(center, 5.0, color);
            }
            painter.text(
                bar.center(),
                egui::Align2::CENTER_CENTER,
                self.window_title(),
                font,
                visuals.strong_text_color(),
            );

            let mut size = if self.use_inner_size || self.resized {
                format!("{:.0} × {:.0}", self.window_size.x, self.window_size.y)
            } else {
                tr.get("eframe-size-os")
            };
            if !self.resizable {
                size = format!("{size} · {}", tr.get("eframe-fixed"));
            }
            painter.text(
                bar.right_center() - egui::vec2(10.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                size,
                small,
                visuals.weak_text_color(),
            );
        }

        let mut app = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        app.set_clip_rect(body);
        self.app_ui(&mut app, tr);

        // After the app, so the corner wins over whatever the app drew there.
        // A browser window can always be resized; a native one only if the
        // viewport allows it.
        if browser || self.resizable {
            self.resize_corner(ui, body, scale);
        }

        ui.painter().rect_stroke(
            rect,
            radius,
            visuals.widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Outside,
        );
    }

    /// A drag handle in the bottom-right corner of `body`, like a real
    /// window's. `scale` turns the drag in mock points into window points.
    fn resize_corner(&mut self, ui: &mut egui::Ui, body: egui::Rect, scale: f32) {
        let corner = egui::Rect::from_min_max(body.max - Vec2::splat(14.0), body.max);
        let response = ui.interact(corner, ui.id().with("eframe_resize"), Sense::drag());
        if response.hovered() || response.dragged() {
            ui.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
        }
        if response.dragged() {
            let size = if self.preview == Preview::Browser {
                &mut self.canvas_size
            } else {
                self.resized = true;
                &mut self.window_size
            };
            *size = (*size + response.drag_delta() / scale).clamp(MIN_SIZE, MAX_SIZE);
        }

        // The usual grip: short diagonal strokes, 4 points apart.
        let stroke = egui::Stroke::new(1.0, ui.style().interact(&response).fg_stroke.color);
        let at = corner.max - Vec2::splat(2.0);
        for offset in [4.0, 8.0, 12.0] {
            ui.painter().line_segment(
                [at - egui::vec2(offset, 0.0), at - egui::vec2(0.0, offset)],
                stroke,
            );
        }
    }

    fn native_main(&self) -> String {
        let mut viewport = CodeBuilder::new("egui::ViewportBuilder::default()");
        viewport
            .call_if(self.use_title, format!(".with_title({:?})", self.title))
            .call_if(
                self.use_inner_size,
                format!(
                    ".with_inner_size([{}, {}])",
                    f32_code(self.inner_size.x),
                    f32_code(self.inner_size.y)
                ),
            )
            .call_if(!self.resizable, ".with_resizable(false)");

        let options = if self.use_title || self.use_inner_size || !self.resizable {
            format!(
                "let options = eframe::NativeOptions {{\n    viewport: {},\n    ..Default::default()\n}};",
                indent(&viewport.build(), 1).trim_start()
            )
        } else {
            "let options = eframe::NativeOptions::default();".to_owned()
        };

        let cfg = if self.web {
            "#[cfg(not(target_arch = \"wasm32\"))]\n"
        } else {
            ""
        };
        format!(
            "{cfg}fn main() -> eframe::Result {{\n{}\n    eframe::run_native(\n        {:?},\n        options,\n        Box::new(|_cc| Ok(Box::new(MyApp::default()))),\n    )\n}}",
            indent(&options, 1),
            self.app_name,
        )
    }

    fn web_main(&self) -> String {
        format!(
            r#"#[cfg(target_arch = "wasm32")]
fn main() {{
    use eframe::wasm_bindgen::JsCast as _;

    let canvas = eframe::web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id({CANVAS_ID:?}))
        .expect("no <canvas id=\"{CANVAS_ID}\"> in index.html")
        .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
        .expect("not a <canvas>");

    wasm_bindgen_futures::spawn_local(async move {{
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(MyApp::default()))),
            )
            .await
            .expect("failed to start eframe");
    }});
}}"#
        )
    }

    fn app_code(&self, tr: Tr<'_>) -> String {
        let mut body = String::new();
        if self.side_panel {
            body.push_str(&format!(
                "egui::Panel::left({SIDE_PANEL_ID:?}).show(ui, |ui| {{\n    ui.label({:?});\n}});\n",
                tr.get("eframe-side-text")
            ));
        }
        body.push_str(&format!(
            "egui::CentralPanel::default().show(ui, |ui| {{\n    ui.heading({:?});\n    if ui.button({:?}).clicked() {{\n        self.clicks += 1;\n    }}\n    ui.label(format!(\"{} {{}}\", self.clicks));\n}});",
            tr.get("eframe-hello"),
            tr.get("eframe-button"),
            tr.get("eframe-clicks"),
        ));

        format!(
            "#[derive(Default)]\nstruct MyApp {{\n    clicks: usize,\n}}\n\nimpl eframe::App for MyApp {{\n    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {{\n{}\n    }}\n}}",
            indent(&body, 2)
        )
    }
}

impl Lesson for EframeLesson {
    fn id(&self) -> &'static str {
        "eframe"
    }

    fn section(&self) -> Section {
        Section::Fundamentals
    }

    fn demo(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        if self.web {
            ui.horizontal(|ui| {
                ui.label(tr.get("eframe-preview"));
                ui.selectable_value(
                    &mut self.preview,
                    Preview::Window,
                    tr.get("eframe-preview-window"),
                );
                ui.selectable_value(
                    &mut self.preview,
                    Preview::Browser,
                    tr.get("eframe-preview-browser"),
                );
            });
            ui.add_space(8.0);
        } else {
            self.preview = Preview::Window;
        }
        self.mock(ui, tr);
    }

    fn controls(&mut self, ui: &mut egui::Ui, tr: Tr<'_>) {
        ui.label("app_name");
        ui.text_edit_singleline(&mut self.app_name);
        ui.add_space(8.0);

        ui.label(egui::RichText::new("egui::ViewportBuilder").monospace());
        ui.checkbox(&mut self.use_title, "with_title");
        ui.add_enabled_ui(self.use_title, |ui| {
            ui.text_edit_singleline(&mut self.title);
        });

        ui.checkbox(&mut self.use_inner_size, "with_inner_size");
        ui.add_enabled_ui(self.use_inner_size, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut self.inner_size.x)
                        .range(MIN_SIZE.x..=MAX_SIZE.x)
                        .prefix("w "),
                );
                ui.add(
                    egui::DragValue::new(&mut self.inner_size.y)
                        .range(MIN_SIZE.y..=MAX_SIZE.y)
                        .prefix("h "),
                );
            });
        });

        ui.checkbox(&mut self.resizable, "with_resizable");
        ui.add_space(8.0);

        ui.label(egui::RichText::new("fn ui").monospace());
        ui.checkbox(&mut self.side_panel, "egui::Panel::left");
        ui.add_space(8.0);

        ui.checkbox(&mut self.web, tr.get("eframe-web-toggle"));
        if self.web {
            ui.label(egui::RichText::new(tr.get("eframe-web-hint")).weak());
        }
    }

    fn code(&self, tr: Tr<'_>) -> String {
        let mut deps = String::from("[dependencies]\neframe = \"0.36\"\n");
        if self.web {
            deps.push_str(
                "\n[target.'cfg(target_arch = \"wasm32\")'.dependencies]\nwasm-bindgen-futures = \"0.4\"\n",
            );
        }

        let mut out = line_comments(&tr.get("eframe-code-deps"));
        out.push_str(&line_comments(&deps));
        out.push_str("\nuse eframe::egui;\n\n");
        out.push_str(&self.native_main());
        if self.web {
            out.push_str("\n\n");
            out.push_str(&self.web_main());
        }
        out.push_str("\n\n");
        out.push_str(&self.app_code(tr));
        out
    }

    fn notes(&self) -> &'static [&'static str] {
        &[
            "ui-not-update",
            "central-last",
            "inner-size",
            "app-name",
            "creation",
            "web",
        ]
    }

    fn retranslate(&mut self, old: Tr<'_>, new: Tr<'_>) {
        retranslate_text(&mut self.title, "eframe-title-text", old, new);
    }

    fn references(&self) -> &'static [&'static str] {
        &[
            "eframe::App",
            "eframe::run_native",
            "eframe::NativeOptions",
            "egui::ViewportBuilder",
            "eframe::CreationContext",
            "eframe::WebRunner",
        ]
    }
}
