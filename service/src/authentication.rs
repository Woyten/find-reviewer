use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Authentication {
    database: HashMap<String, String>,
}

pub enum AuthenticationRequest {
    LoadIdentity { token: String },
    SendIdentity { token: String },
}

pub enum AuthenticationResponse {
    KnownIdentity { coder: String },
    UnknownIdentity {},
}

impl Authentication {
    pub fn new() -> Authentication {
        let mut auth = Authentication {
            database: HashMap::new(),
        };
        auth.database
            .insert(String::from("token1"), String::from("coder1"));
        auth.database
            .insert(String::from("token2"), String::from("coder2"));

        auth
    }

    pub fn process_request(&mut self, request: AuthenticationRequest) -> AuthenticationResponse {
        match match request {
            AuthenticationRequest::LoadIdentity { token } => self.database.get(&token),
            AuthenticationRequest::SendIdentity { token } => self.database.get(&token),
        } {
            Some(coder) => AuthenticationResponse::KnownIdentity {
                coder: coder.clone(),
            },
            _ => AuthenticationResponse::UnknownIdentity {},
        }
    }
}
