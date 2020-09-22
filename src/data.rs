use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::SparrowError;
use crate::Task;

#[derive(Default, Deserialize, Serialize)]
pub struct Data {
    tasks: Vec<Task>
}

impl Data {
    pub fn from_file(path: &Path) -> Result<Self, SparrowError> {
        if !path.exists() {
            Ok(Self::default())
        } else {
            Ok(serde_yaml::from_reader(fs::File::open(path)?)?)
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), SparrowError> {
        Ok(fs::write(path, serde_yaml::to_string(self)?)?)
    }
}
