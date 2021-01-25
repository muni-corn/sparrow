use ansi_term::Style;

pub mod data;
pub mod errors;
pub mod prompts;
pub mod schedule;
pub mod spans;
pub mod task;
pub mod methods;

pub use data::*;
pub use errors::{SparrowError, SparrowResult};
pub use schedule::Schedule;
pub use spans::*;
pub use task::Task;

pub struct Formatting {
    pub prompt: Style,
    pub prompt_format: Style,
    pub error: Style,
}
