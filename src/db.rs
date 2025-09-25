// use std::fs;

use anyhow::{Result, anyhow};
// use serde::{Deserialize, Serialize};

use crate::models::{DBState, Epic, Status, Story};

pub struct JiraDatabase {
    database: Box<dyn Database>,
}

impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase { file_path }),
        }
    }

    pub fn read_db(&self) -> Result<DBState, anyhow::Error> {
        self.database
            .read_db()
            .map_err(|e| anyhow!("Reading database failed {}", e))
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let jira_db = self.read_db();
        // .map_err(|e| anyhow!("Jira database retrieval failed {}", e));

        match jira_db {
            Ok(mut db) => {
                db.last_item_id += 1;
                db.epics.insert(db.last_item_id, epic);
                Ok(db.last_item_id)
            }
            Err(e) => Err(anyhow!("Epic insertion failed {}", e)),
        }
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                db.last_item_id += 1;
                let associated_epic = db.epics.get_mut(&epic_id);
                match associated_epic {
                    Some(epic) => {
                        epic.stories.push(db.last_item_id);
                        db.stories.insert(db.last_item_id, story);
                        Ok(db.last_item_id)
                    }
                    None => Err(anyhow!("Epic with id {} was not found", epic_id)),
                }
            }
            Err(e) => Err(anyhow!("Story insertion failed {}", e)),
        }
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                let associated_epic = db.epics.get_mut(&epic_id);
                match associated_epic {
                    Some(epic) => {
                        for story_id in &epic.stories {
                            db.stories.remove(story_id);
                        }
                        db.epics.remove(&epic_id);
                        Ok(())
                    }
                    None => Err(anyhow!("Epic with id {} was not found", &epic_id)),
                }
            }
            Err(e) => Err(anyhow!("Epic deletion failed {}", e)),
        }
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                let associated_epic = db.epics.get_mut(&epic_id);

                match associated_epic {
                    Some(epic) => {
                        if let Some(target_index) =
                            epic.stories.iter().position(|id| id == &story_id)
                        {
                            epic.stories.swap_remove(target_index);
                            db.stories.remove(&story_id);
                            Ok(())
                        } else {
                            Err(anyhow!(
                                "Story with id {} was not found in epic with id {}",
                                &story_id,
                                &epic_id
                            ))
                        }
                    }
                    None => Err(anyhow!("Epic with id {} was not found", &epic_id)),
                }
            }
            Err(e) => Err(anyhow!("Story deletion failed {}", e)),
        }
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                let associated_epic = db.epics.get_mut(&epic_id);

                match associated_epic {
                    Some(epic) => {
                        epic.status = status;
                        Ok(())
                    }
                    None => Err(anyhow!("Epic with id {} was not found", &epic_id)),
                }
            }
            Err(e) => Err(anyhow!("Failed to update epic status {}", e)),
        }
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                let associated_story = db.stories.get_mut(&story_id);

                match associated_story {
                    Some(story) => {
                        story.status = status;
                        Ok(())
                    }
                    None => Err(anyhow!("Story with id {} was not found", &story_id)),
                }
            }
            Err(e) => Err(anyhow!("Failed to update story status {}", e)),
        }
    }
}

trait Database {
    fn read_db(&self) -> Result<DBState>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState> {
        let db_content = std::fs::read_to_string(&self.file_path)?;
        let parsed = serde_json::from_str(&db_content)?;
        Ok(parsed)
    }

    fn write_db(&self, db_state: &DBState) -> Result<()> {
        std::fs::write(&self.file_path, serde_json::to_vec(db_state)?)?;
        Ok(())
    }
}

// pub mod test_utils {
//     use std::{cell::RefCell, collections::HashMap};

//     use super::*;

//     pub struct MockDB {
//         last_written_state: RefCell<DBState>
//     }

//     impl MockDB {
//         pub fn new() -> Self {
//             Self { last_written_state: RefCell::new(DBState { last_item_id: 0, epics: HashMap::new(), stories: HashMap::new() }) }
//         }
//     }

//     impl Database for MockDB {
//         fn read_db(&self) -> Result<DBState> {
//             // TODO: fix this error by deriving the appropriate traits for Story
//             let state = self.last_written_state.borrow().clone();
//             Ok(state)
//         }

//         fn write_db(&self, db_state: &DBState) -> Result<()> {
//             let latest_state = &self.last_written_state;
//             // TODO: fix this error by deriving the appropriate traits for DBState
//             *latest_state.borrow_mut() = db_state.clone();
//             Ok(())
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    mod database {
        use std::collections::HashMap;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "INVALID_PATH".to_owned(),
            };
            assert!(db.read_db().is_err());
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("failed to convert tmpfile path to str")
                    .to_string(),
            };

            let result = db.read_db();

            assert!(result.is_err());
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("failed to convert tmpfile path to str")
                    .to_string(),
            };

            let result = db.read_db();

            assert!(result.is_ok());
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("failed to convert tmpfile path to str")
                    .to_string(),
            };

            let story = Story {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
            };
            let epic = Epic {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
                stories: vec![2],
            };

            let mut stories = HashMap::new();
            stories.insert(2, story);

            let mut epics = HashMap::new();
            epics.insert(1, epic);

            let state = DBState {
                last_item_id: 2,
                epics,
                stories,
            };

            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            assert!(write_result.is_ok());
            // TODO: fix this error by deriving the appropriate traits for DBState
            assert_eq!(read_result, state);
        }
    }
}
