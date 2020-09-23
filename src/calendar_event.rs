use crate::Occupancy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub name: String,
    pub span: Occupancy,
    pub event_type: CalendarEventType,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CalendarEventType {
    Break,
    Event,
}
