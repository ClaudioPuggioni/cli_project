use std::collections::HashMap;

pub enum Status {
    Open,
    InProgress,
    Closed,
}

pub struct Epic {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<u16>,
}

impl Epic {
    pub fn new(name: String, description: String, id: u16) -> Self {
        Epic {
            id,
            name,
            description,
            status: Status::Open,
            stories: Vec::new(),
        }
    }
}

pub struct Story {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    pub fn new(name: String, description: String, id: u16) -> Self {
        Story {
            id,
            name,
            description,
            status: Status::Open,
        }
    }
}

pub struct DBState {
    pub last_item_id: u16,
    pub epics: HashMap<u16, Epic>,
    pub stories: HashMap<u16, Story>,
}

impl DBState {
    pub fn new() -> DBState {
        DBState {
            last_item_id: 0,
            epics: HashMap::new(),
            stories: HashMap::new(),
        }
    }

    pub fn create_epic(&mut self, name: String, description: String) {
        self.last_item_id += 1;
        let new_id = self.last_item_id;
        let new_epic = Epic::new(name, description, new_id);
        self.epics.insert(new_id, new_epic);
    }

    pub fn create_story(&mut self, name: String, description: String) {
        self.last_item_id += 1;
        let new_id = self.last_item_id;
        let new_story = Story::new(name, description, new_id);
        self.stories.insert(new_id, new_story);
    }
}
