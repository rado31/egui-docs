//! The shape of a single lesson.
//!
//! Every lesson is three synchronized views of the same state:
//!   * [`Lesson::demo`]     — the real widget, running.
//!   * [`Lesson::controls`] — knobs that mutate the lesson state.
//!   * [`Lesson::code`]     — the code you would write to get exactly what you see.
//!
//! The code is *generated from the same state the demo reads*, never hand-written
//! next to it. That is the whole point: a snippet cannot drift from its demo.

use crate::i18n::Tr;

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

    /// The id of this section's title in `locales/*/app.ftl`.
    pub fn title_id(self) -> &'static str {
        match self {
            Self::Fundamentals => "section-fundamentals",
            Self::Widgets => "section-widgets",
            Self::Layout => "section-layout",
            Self::Style => "section-style",
            Self::Text => "section-text",
            Self::Interaction => "section-interaction",
            Self::Painting => "section-painting",
            Self::Containers => "section-containers",
            Self::State => "section-state",
        }
    }
}

/// A lesson's text lives in `locales/<language>/<id>.ftl`, keyed by the
/// lesson's [`Lesson::id`]:
///
/// ```ftl
/// button-title = Button
/// button-summary = One sentence, shown right under the title.
/// button-no-callback = A note's heading
///     .body = The note itself. `code` and *emphasis* are understood.
/// ```
///
/// Everything else a lesson shows — control labels, hints, comments in the
/// generated code — is looked up through the [`Tr`] it is handed.
pub trait Lesson {
    /// Stable and never translated: the prefix of every message id of this
    /// lesson, and the name of its `.ftl` file.
    fn id(&self) -> &'static str;

    fn section(&self) -> Section;

    /// The live widget. Takes `&mut self` because a demo may keep runtime state
    /// (click counters, text buffers, ...).
    fn demo(&mut self, ui: &mut egui::Ui, tr: Tr<'_>);

    /// The knobs. Mutates the same state `demo` reads.
    fn controls(&mut self, ui: &mut egui::Ui, tr: Tr<'_>);

    /// The snippet, derived from the current state — and from the same
    /// translated text the demo shows.
    fn code(&self, tr: Tr<'_>) -> String;

    /// Ids of the notes shown under the demo, in order, without the lesson
    /// prefix: `"no-callback"` is the message `button-no-callback`.
    fn notes(&self) -> &'static [&'static str] {
        &[]
    }

    /// Names to look up — in `docs.rs`, in your editor, or in egui's source.
    ///
    /// Deliberately not URLs. A link pinned to one release breaks on the next,
    /// and a hand-written docs.rs path is easy to get wrong in a way nothing
    /// checks: `egui::Button` is re-exported into the crate root, but its page
    /// only exists under `egui/widgets/`. A name works in every version.
    ///
    /// Never translated: they are names in code.
    fn references(&self) -> &'static [&'static str] {
        &[]
    }

    /// The language changed from `old` to `new`. A lesson whose state holds
    /// text — an editable button label — swaps it here, but only if the reader
    /// has not edited it: their own text is theirs to keep.
    fn retranslate(&mut self, _old: Tr<'_>, _new: Tr<'_>) {}

    fn title(&self, tr: Tr<'_>) -> String {
        tr.get(&format!("{}-title", self.id()))
    }

    /// One sentence, shown right under the title.
    fn summary(&self, tr: Tr<'_>) -> String {
        tr.get(&format!("{}-summary", self.id()))
    }
}

/// Replace `text` with `new`'s version of message `id` if it still holds
/// `old`'s — i.e. if the reader has not edited it. For [`Lesson::retranslate`].
pub fn retranslate_text(text: &mut String, id: &str, old: Tr<'_>, new: Tr<'_>) {
    if *text == old.get(id) {
        *text = new.get(id);
    }
}
