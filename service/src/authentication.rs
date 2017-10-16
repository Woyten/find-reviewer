use std::collections::HashMap;

pub struct Authentication {
    database: UserDatabase,
}

#[derive(Deserialize, Serialize)]
pub struct UserDatabase {
    users: HashMap<String, String>,
}

pub enum AuthenticationResponse {
    KnownIdentity { coder: String },
    UnknownIdentity {},
}

impl Authentication {
    pub fn new(users: UserDatabase) -> Authentication {
        Authentication { database: users }
    }

    pub fn process_request(&mut self, token: &String) -> AuthenticationResponse {
        match self.database.users.get(token)
        {
            Some(coder) => AuthenticationResponse::KnownIdentity {
                coder: coder.clone(),
            },
            _ => AuthenticationResponse::UnknownIdentity {},
        }
    }
}
