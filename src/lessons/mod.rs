//! The lesson registry. Add a lesson here and it appears in the sidebar.
//!
//! Order within a section is the order lessons are listed here.

mod button;
mod response;

use crate::lesson::Lesson;

pub fn all() -> Vec<Box<dyn Lesson>> {
    vec![
        Box::new(response::ResponseLesson::default()),
        Box::new(button::ButtonLesson::default()),
    ]
}
