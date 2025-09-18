use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Epic {
    // pub id: u32,
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<u32>,
}

impl Epic {
    // pub fn new(id: u32, name: String, description: String) -> Self {
    pub fn new(name: String, description: String) -> Self {
        Epic {
            // id,
            name,
            description,
            status: Status::Open,
            stories: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Story {
    // pub id: u32,
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    // pub fn new(id: u32, name: String, description: String) -> Self {
    pub fn new(name: String, description: String) -> Self {
        Story {
            // id,
            name,
            description,
            status: Status::Open,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DBState {
    pub last_item_id: u32,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}

impl DBState {
    pub fn new() -> Self {
        DBState {
            last_item_id: 0,
            epics: HashMap::new(),
            stories: HashMap::new(),
        }
    }

    pub fn create_epic(&mut self, name: String, description: String) {
        self.last_item_id += 1;
        // let id = self.last_item_id;
        // let new_epic = Epic::new(id, name, description);
        // self.epics.insert(id, new_epic);
        let new_epic = Epic::new(name, description);
        self.epics.insert(self.last_item_id, new_epic);
    }

    pub fn create_story(&mut self, name: String, description: String) {
        self.last_item_id += 1;
        // let id = self.last_item_id;
        // let new_story = Story::new(id, name, description);
        // self.stories.insert(id, new_story);
        let new_story = Story::new(name, description);
        self.stories.insert(self.last_item_id, new_story);
    }
}
