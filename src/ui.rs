//! Small UI helpers shared by the lessons.

/// Keeps a one-frame event visible long enough to read.
///
/// Momentary `Response` flags (`clicked()`, `drag_started()`, ...) are `true`
/// for exactly one frame — at 60 fps that is 16 ms, far too short to notice.
/// A `Pulse` remembers *when* the flag fired and fades out over
/// [`Pulse::DURATION`], requesting repaints while it does.
///
/// A `Pulse` is never driven by hand: [`event_flag`] owns that call, which is
/// what makes it impossible to draw an event without its fade.
#[derive(Default)]
pub struct Pulse {
    fired_at: Option<f64>,
}

impl Pulse {
    pub const DURATION: f64 = 0.7;

    /// Call once per frame with the current value of the flag.
    /// Returns `1.0` right after it fired, fading to `0.0`.
    fn update(&mut self, ui: &egui::Ui, fired: bool) -> f32 {
        let now = ui.input(|i| i.time);

        if fired {
            self.fired_at = Some(now);
        }

        let Some(fired_at) = self.fired_at else {
            return 0.0;
        };

        let elapsed = now - fired_at;
        if elapsed >= Self::DURATION {
            self.fired_at = None;
            return 0.0;
        }

        // Nothing else is animating, so ask for the next frame ourselves.
        ui.ctx().request_repaint();
        1.0 - (elapsed / Self::DURATION) as f32
    }
}

/// A chip for a `Response` **state** — true for as long as the condition holds
/// (`hovered()`, `dragged()`, `has_focus()`).
///
/// A state needs no fade: it stays lit on its own.
pub fn state_flag(ui: &mut egui::Ui, name: &str, value: bool) {
    chip(ui, name, value, 0.0);
}

/// A chip for a `Response` **event** — true for exactly one frame
/// (`clicked()`, `drag_started()`, `lost_focus()`).
///
/// The [`Pulse`] is taken by `&mut` and driven here, so an event can never be
/// drawn without the fade that makes it visible. That is the whole reason this
/// is a separate function from [`state_flag`] rather than a `glow: f32`
/// parameter: passing `0.0` for an event compiled fine and silently produced a
/// chip nobody could ever see blink.
pub fn event_flag(ui: &mut egui::Ui, name: &str, value: bool, pulse: &mut Pulse) {
    let glow = pulse.update(ui, value);
    chip(ui, name, value, glow);
}

/// The shared look of both flag kinds.
///
/// `glow` (0..=1) lights the chip up even when `value` is already `false`
/// again — that is how a one-frame flag stays readable.
fn chip(ui: &mut egui::Ui, name: &str, value: bool, glow: f32) {
    let lit = if value { 1.0 } else { glow.clamp(0.0, 1.0) };

    let bg = ui
        .visuals()
        .faint_bg_color
        .lerp_to_gamma(ui.visuals().selection.bg_fill, lit);
    let fg = ui
        .visuals()
        .weak_text_color()
        .lerp_to_gamma(ui.visuals().strong_text_color(), lit);

    // Lay the text out first, unwrapped, so we know how wide the chip has to be.
    //
    // A `Frame` with a `Label` inside would *not* work here: inside
    // `horizontal_wrapped` a nested Ui is only offered the width left in the
    // current row, so the label would wrap mid-word — one character per line —
    // and stretch the whole row. A widget that measures itself and allocates
    // exactly that much lets the outer layout wrap between chips instead.
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let galley =
        ui.painter()
            .layout_no_wrap(format!("{name}: {value}"), font, egui::Color32::PLACEHOLDER);

    let padding = egui::vec2(6.0, 3.0);
    let (rect, _response) =
        ui.allocate_exact_size(galley.size() + 2.0 * padding, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 4, bg);
        ui.painter().galley(rect.min + padding, galley, fg);
    }
}
