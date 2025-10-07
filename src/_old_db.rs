// use std::fs;

use anyhow::{Context, Result, anyhow};
// use serde::{Deserialize, Serialize};

use crate::models::{DBState, Epic, Status, Story};

pub struct JiraDatabase {
    pub database: Box<dyn Database>,
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

    pub fn write_db(&self, db: &DBState) -> Result<(), anyhow::Error> {
        self.database
            .write_db(db)
            .map_err(|e| anyhow!("Writing to database failed {}", e))
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let jira_db = self.read_db();

        match jira_db {
            Ok(mut db) => {
                db.last_item_id += 1;
                db.epics.insert(db.last_item_id, epic);
                self.write_db(&db).with_context(|| {
                    format!(
                        "Failed to write to database in create_epic for id {}",
                        db.last_item_id
                    )
                })?;
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

                        self.write_db(&db).with_context(|| {
                            format!(
                                "Failed to write to database in create_story for id {}",
                                db.last_item_id
                            )
                        })?;

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

                        self.write_db(&db).with_context(|| {
                            format!(
                                "Failed to write to database in delete_epic for epic {}",
                                epic_id
                            )
                        })?;

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

                            self.write_db(&db).with_context(|| {
                                format!("Failed to write to database in delete_story for story with id {} and epic id {}", &story_id, &epic_id)
                            })?;

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

                        self.write_db(&db).with_context(|| {
                            format!(
                                "Failed to write to database in update_epic_status with epic id {}",
                                &epic_id
                            )
                        })?;

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

                        self.write_db(&db).with_context(|| {
                            format!("Failed to write to database in update_story_status with story id {}", &story_id)
                        })?;

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

pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;

    pub struct MockDB {
        last_written_state: RefCell<DBState>,
    }

    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState> {
            // TODO: fix this error by deriving the appropriate traits for Story
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write_db(&self, db_state: &DBState) -> Result<()> {
            let latest_state = &self.last_written_state;
            // TODO: fix this error by deriving the appropriate traits for DBState
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic.clone());
        assert!(result.is_ok());

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        // print!("db_state.last_item_id {}", db_state.last_item_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert!(result.is_err());
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story.clone(), epic_id);
        assert!(result.is_ok());

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert!(db_state.epics.get(&epic_id).unwrap().stories.contains(&id));
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert!(result.is_ok());

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert!(result.is_ok());

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert!(result.is_ok());

        let story_id = result.unwrap();

        let non_existent_epic_id = 999;

        let result = db.delete_story(non_existent_epic_id, story_id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert!(result.is_ok());

        let non_existent_story_id = 999;

        let result = db.delete_story(epic_id, non_existent_story_id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert!(result.is_ok());

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert!(result.is_ok());

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert!(
            !db_state
                .epics
                .get(&epic_id)
                .unwrap()
                .stories
                .contains(&story_id)
        );
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert!(result.is_err());
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);
        assert!(result.is_ok());

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert!(result.is_err());
    }

    #[test]
    fn update_story_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert!(result.is_ok());

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert!(result.is_ok());

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);
        assert!(result.is_ok());

        let db_state = db.read_db().unwrap();
        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::Closed
        );
    }

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
                name: "story 1".to_owned(),
                description: "story 1".to_owned(),
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
            assert_eq!(read_result, state);
        }
    }
    /////////// OLD TESTS - Start ///////////

    // mod database {
    //     use std::collections::HashMap;
    //     use std::io::Write;

    // use super::*;

    // #[test]
    // fn read_db_should_fail_with_invalid_path() {
    //     let db = JSONFileDatabase {
    //         file_path: "INVALID_PATH".to_owned(),
    //     };
    //     assert!(db.read_db().is_err());
    // }

    // #[test]
    // fn read_db_should_fail_with_invalid_json() {
    //     let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

    //     let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
    //     write!(tmpfile, "{}", file_contents).unwrap();

    //     let db = JSONFileDatabase {
    //         file_path: tmpfile
    //             .path()
    //             .to_str()
    //             .expect("failed to convert tmpfile path to str")
    //             .to_string(),
    //     };

    //     let result = db.read_db();

    //     assert!(result.is_err());
    // }

    // #[test]
    // fn read_db_should_parse_json_file() {
    //     let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

    //     let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
    //     write!(tmpfile, "{}", file_contents).unwrap();

    //     let db = JSONFileDatabase {
    //         file_path: tmpfile
    //             .path()
    //             .to_str()
    //             .expect("failed to convert tmpfile path to str")
    //             .to_string(),
    //     };

    //     let result = db.read_db();

    //     assert!(result.is_ok());
    // }

    // #[test]
    // fn write_db_should_work() {
    //     let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

    //     let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
    //     write!(tmpfile, "{}", file_contents).unwrap();

    //     let db = JSONFileDatabase {
    //         file_path: tmpfile
    //             .path()
    //             .to_str()
    //             .expect("failed to convert tmpfile path to str")
    //             .to_string(),
    //     };

    //     let story = Story {
    //         name: "epic 1".to_owned(),
    //         description: "epic 1".to_owned(),
    //         status: Status::Open,
    //     };
    //     let epic = Epic {
    //         name: "epic 1".to_owned(),
    //         description: "epic 1".to_owned(),
    //         status: Status::Open,
    //         stories: vec![2],
    //     };

    //     let mut stories = HashMap::new();
    //     stories.insert(2, story);

    //     let mut epics = HashMap::new();
    //     epics.insert(1, epic);

    //     let state = DBState {
    //         last_item_id: 2,
    //         epics,
    //         stories,
    //     };

    //     let write_result = db.write_db(&state);
    //     let read_result = db.read_db().unwrap();

    //     assert!(write_result.is_ok());
    //     // TODO: fix this error by deriving the appropriate traits for DBState
    //     assert_eq!(read_result, state);
    // }

    /////////// OLD TESTS - End ///////////
    // }
}
