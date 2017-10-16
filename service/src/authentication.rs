use std::collections::HashMap;

pub struct Authentication {
    database: UserDatabase,
}

#[derive(Deserialize, Serialize)]
pub struct UserDatabase {
    users: HashMap<String, String>,
}

impl Authentication {
    pub fn new(users: UserDatabase) -> Authentication {
        Authentication { database: users }
    }

    pub fn process_request(&mut self, token: &String) -> Option<String> {
        match self.database.users.get(token) {
            Some(coder) => Some(coder.clone()),
            _ => None,
        }
    }
}
