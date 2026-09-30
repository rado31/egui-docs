//! The lesson registry. Add a lesson here and it appears in the sidebar. Its
//! text goes in `locales/<language>/<lesson id>.ftl`.
//!
//! Order within a section is the order lessons are listed here.

mod atom;
mod button;
mod id;
mod response;
mod ui;

use crate::i18n::Tr;
use crate::lesson::Lesson;

/// `tr` is the starting language, for lessons whose state holds text.
pub fn all(tr: Tr<'_>) -> Vec<Box<dyn Lesson>> {
    vec![
        Box::new(response::ResponseLesson::default()),
        Box::new(ui::UiLesson::default()),
        Box::new(id::IdLesson::default()),
        Box::new(atom::AtomLesson::new(tr)),
        Box::new(button::ButtonLesson::new(tr)),
    ]
}
