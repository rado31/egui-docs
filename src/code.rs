//! Tiny helper for building a builder-pattern snippet out of lesson state.
//!
//! `egui` widgets are almost all `Foo::new(..).bar(..).baz(..)`, so a lesson only
//! has to say which calls are currently active:
//!
//! ```ignore
//! let mut b = CodeBuilder::new(r#"egui::Button::new("Click me")"#);
//! b.call_if(self.small, ".small()");
//! b.call_if(self.use_fill, format!(".fill({})", color_code(self.fill)));
//! b.build()
//! ```

pub struct CodeBuilder {
    head: String,
    calls: Vec<String>,
}

impl CodeBuilder {
    pub fn new(head: impl Into<String>) -> Self {
        Self {
            head: head.into(),
            calls: Vec::new(),
        }
    }

    /// Append a chained call, e.g. `".small()"`.
    pub fn call(&mut self, call: impl Into<String>) -> &mut Self {
        self.calls.push(call.into());
        self
    }

    /// Append a chained call only when the knob is on.
    pub fn call_if(&mut self, condition: bool, call: impl Into<String>) -> &mut Self {
        if condition {
            self.call(call);
        }
        self
    }

    /// `Head::new(..)` on the first line, one chained call per following line.
    pub fn build(&self) -> String {
        let mut out = self.head.clone();
        for call in &self.calls {
            out.push('\n');
            out.push_str("    ");
            out.push_str(call);
        }
        out
    }
}

/// Indent every line of `text` by `levels * 4` spaces, for nesting a snippet
/// inside another expression. Empty lines stay empty.
pub fn indent(text: &str, levels: usize) -> String {
    let pad = " ".repeat(levels * 4);
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `Color32` as the literal you would type yourself.
pub fn color_code(color: egui::Color32) -> String {
    let [r, g, b, a] = color.to_array();
    if a == 255 {
        format!("egui::Color32::from_rgb({r}, {g}, {b})")
    } else {
        format!("egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {a})")
    }
}

/// `1.0` rather than `1`, so the literal keeps its `f32` type.
pub fn f32_code(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}
