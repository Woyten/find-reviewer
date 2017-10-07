use std::collections::HashMap;
type User = String;
type Token = String;

#[derive(Deserialize, Serialize)]
pub struct Authentication {
    database: HashMap<Token, User>,
}

pub enum AuthenticationRequest {
    LoadIdentity {},
    SendIdentity { token: String },
}

pub enum AuthenticationResponse {
    KnownIdentity { coder: String },
    UnknownIdendity {},
}

impl Authentication {
    pub fn new(file_name: String) {
        unimplemented!()
    }

    pub fn process_request(request: AuthenticationRequest) -> AuthenticationResponse {
        unimplemented!()
    }
}
