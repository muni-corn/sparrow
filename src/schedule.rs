use std::fmt::Display;
use crate::{
    task::Task, Bedtime, CalendarEvent, Config, SparrowError,
};
use serde::{Deserialize, Serialize};

pub trait Schedule<'d>: Sized + Clone + Deserialize<'d> + Serialize {
    type Display: Display;

    fn make(
        config: &Config,
        tasks: &[Task],
        events: &[CalendarEvent],
        bedtime: &Bedtime,
    ) -> Result<Self, SparrowError>;

    fn display(
        &'d self,
        config: &'d Config,
    ) -> Self::Display;
}

