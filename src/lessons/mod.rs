//! The lesson registry. Add a lesson here and it appears in the sidebar.
//!
//! Order within a section is the order lessons are listed here.

mod button;
mod id;
mod response;
mod ui;

use crate::lesson::Lesson;

pub fn all() -> Vec<Box<dyn Lesson>> {
    vec![
        Box::new(response::ResponseLesson::default()),
        Box::new(ui::UiLesson::default()),
        Box::new(id::IdLesson::default()),
        Box::new(button::ButtonLesson::default()),
    ]
}
