use std::collections::HashMap;

pub struct Authentication {
    database: UserDatabase,
}

#[derive(Deserialize, Serialize)]
pub struct UserDatabase {
    users: HashMap<String, String>,
}

impl UserDatabase {
    pub fn default() -> UserDatabase {
        let mut default_users: HashMap<String, String> = HashMap::new();
        default_users.insert(format!("token1"), format!("user1"));
        default_users.insert(format!("token2"), format!("user2"));
        UserDatabase {
            users: default_users,
        }
    }
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
