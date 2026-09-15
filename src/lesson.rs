//! The shape of a single lesson.
//!
//! Every lesson is three synchronized views of the same state:
//!   * [`Lesson::demo`]     — the real widget, running.
//!   * [`Lesson::controls`] — knobs that mutate the lesson state.
//!   * [`Lesson::code`]     — the code you would write to get exactly what you see.
//!
//! The code is *generated from the same state the demo reads*, never hand-written
//! next to it. That is the whole point: a snippet cannot drift from its demo.

/// Where a lesson sits in the table of contents.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    Fundamentals,
    Widgets,
    Layout,
    Style,
    Text,
    Interaction,
    Painting,
    Containers,
    State,
}

impl Section {
    /// Reading order of the whole guide. Sections without lessons yet are still
    /// listed, so the shape of the project stays visible while it is being built.
    pub const ALL: &'static [Self] = &[
        Self::Fundamentals,
        Self::Widgets,
        Self::Layout,
        Self::Style,
        Self::Text,
        Self::Interaction,
        Self::Painting,
        Self::Containers,
        Self::State,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Fundamentals => "Fundamentals",
            Self::Widgets => "Widgets",
            Self::Layout => "Layout",
            Self::Style => "Style & Theme",
            Self::Text => "Text & Fonts",
            Self::Interaction => "Input & Interaction",
            Self::Painting => "Painting",
            Self::Containers => "Containers & Windows",
            Self::State => "State & Persistence",
        }
    }
}

/// A paragraph of explanation, shown under the demo.
pub struct Note {
    pub heading: &'static str,
    pub body: &'static str,
}

/// A link out to the official docs or to egui's own source.
pub struct Link {
    pub label: &'static str,
    pub url: &'static str,
}

pub trait Lesson {
    fn title(&self) -> &'static str;

    fn section(&self) -> Section;

    /// One sentence, shown right under the title.
    fn summary(&self) -> &'static str;

    /// The live widget. Takes `&mut self` because a demo may keep runtime state
    /// (click counters, text buffers, ...).
    fn demo(&mut self, ui: &mut egui::Ui);

    /// The knobs. Mutates the same state `demo` reads.
    fn controls(&mut self, ui: &mut egui::Ui);

    /// The snippet, derived from the current state.
    fn code(&self) -> String;

    fn notes(&self) -> &'static [Note] {
        &[]
    }

    fn links(&self) -> &'static [Link] {
        &[]
    }
}
