extern crate iron;
extern crate mount;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate staticfile;
extern crate time;

use application::Application;
use application::ApplicationConfiguration;
use authentication::Authentication;
use iron::headers::Cookie;
use iron::headers::SetCookie;
use iron::method::Method;
use iron::prelude::*;
use iron::status::Status;
use mount::Mount;
use staticfile::Static;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;


mod application;
mod authentication;

static CONFIG_FILE_NAME: &str = "find-reviewer.json";

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum ServerRequest {
    NeedReviewer {},
    HaveTimeForReview {},
    WillReview { review_id: u32 },
    WontReview { review_id: u32 },
    LoadIdentity {},
    SendIdentity { token: String },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum ServerResponse {
    Accepted {},
    NoReviewerNeeded {},
    AlreadyRegistered {},
    NeedsReviewer { coder: String, review_id: u32 },
    ReviewNotFound {},
    KnownIdentity { username: String },
    UnknownIdentity {},
}


type SharedApplication = Arc<Mutex<Application>>;
type SharedAuthentication = Arc<Mutex<Authentication>>;

fn main() {
    let configuration = load_configuration();
    save_configuration(&configuration);

    let address = configuration.address.clone();
    let application = SharedApplication::new(Mutex::new(Application::new(configuration)));
    let authentication = SharedAuthentication::new(Mutex::new(Authentication::new()));

    start_timeout_loop(application.clone());
    start_service(&address, application, authentication);
}

fn load_configuration() -> ApplicationConfiguration {
    File::open(CONFIG_FILE_NAME)
        .map(|open_file| serde_json::from_reader(open_file).expect(&format!("Could not parse {}", CONFIG_FILE_NAME)))
        .unwrap_or_else(|err| {
            println!("Could not read {}: {}\nFile will be created", CONFIG_FILE_NAME, err);
            ApplicationConfiguration::default()
        })
}

fn save_configuration(configuration: &ApplicationConfiguration) {
    File::create(CONFIG_FILE_NAME)
        .map(|created_file| {
            serde_json::to_writer_pretty(created_file, &configuration).expect(&format!("Could not serialize {}", CONFIG_FILE_NAME))
        })
        .unwrap_or_else(|err| println!("Could not write {}: {}", CONFIG_FILE_NAME, err));
}

fn start_timeout_loop(application: SharedApplication) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        application.lock().unwrap().process_timeouts();
    });
}

fn start_service(address: &str, application: SharedApplication, authentication: SharedAuthentication) {
    let mut mount = Mount::new();
    mount
        .mount("/find-reviewer", move |request: &mut Request| Ok(process_request(request, &application, &authentication)))
        .mount("/", Static::new(Path::new("www")));

    Iron::new(mount).http(address).unwrap();
}

fn process_request(request: &mut Request, application: &SharedApplication, authentication: &SharedAuthentication) -> Response {
    if request.method != Method::Post {
        return Response::with((Status::BadRequest, "Must be a POST request"));
    }

    let parsed = match serde_json::from_reader(request.body.by_ref()) {
        Err(message) => return Response::with((Status::BadRequest, format!("JSON error: {}", message))),
        Ok(request) => request,
    };

    let token = extract_token(request);

    let server_response = match token.clone() {
        Some(token) => match adapt_application_request(&parsed, token.clone()) {
            Some(app_request) => adapt_application_response(application.lock().unwrap().dispatch_request(app_request)),
            None => adapt_authentication_response(
                authentication
                    .lock()
                    .unwrap()
                    .process_request(adapt_authentication_request(&parsed, token.clone()).unwrap()),
            ),
        },
        None => ServerResponse::UnknownIdentity {},
    };

    let mut resp = Response::with((Status::Ok, serde_json::to_string_pretty(&server_response).unwrap()));
    let time = time::now() + time::Duration::weeks(4);
    match get_token(parsed, server_response, token.clone()) {
        Some(send_token) => resp.headers.set(SetCookie(vec![
            format!("token={}; Path=/find-reviewer; Expires={}", send_token, time.rfc822()),
        ])),
        None => (),
    }
    resp
}

fn get_token(request: ServerRequest, response: ServerResponse, token_from_cookie: Option<String>) -> Option<String> {
    match response {
        ServerResponse::UnknownIdentity {} => None,
        _ => match request {
            ServerRequest::SendIdentity { token } => Some(token),
            _ => match token_from_cookie {
                Some(token) => Some(token),
                None => None,
            },
        },
    }
}

fn extract_token<'a>(request: &'a Request) -> Option<String> {
    request.headers.get::<Cookie>().and_then(|cookies| {
        cookies
            .iter()
            .map(|x| x.split('=').collect::<Vec<_>>())
            .filter_map(|splitted| if let (Some(&"token"), Some(&value)) = (splitted.get(0), splitted.get(1)) {
                Some(String::from(value))
            } else {
                None
            })
            .next()
    })
}

fn adapt_application_request(request: &ServerRequest, coder: String) -> Option<application::FindReviewerRequest> {
    match *request {
        ServerRequest::NeedReviewer {} => Some(application::FindReviewerRequest::NeedReviewer { coder }),
        ServerRequest::HaveTimeForReview {} => Some(application::FindReviewerRequest::HaveTimeForReview { reviewer: coder }),
        ServerRequest::WillReview { review_id } => Some(application::FindReviewerRequest::WillReview { review_id }),
        ServerRequest::WontReview { review_id } => Some(application::FindReviewerRequest::WontReview { review_id }),
        _ => None,
    }
}

fn adapt_application_response(response: application::FindReviewerResponse) -> ServerResponse {
    match response {
        application::FindReviewerResponse::Accepted {} => ServerResponse::Accepted {},
        application::FindReviewerResponse::AlreadyRegistered {} => ServerResponse::AlreadyRegistered {},
        application::FindReviewerResponse::NoReviewerNeeded {} => ServerResponse::NoReviewerNeeded {},
        application::FindReviewerResponse::ReviewNotFound {} => ServerResponse::ReviewNotFound {},
        application::FindReviewerResponse::NeedsReviewer { coder, review_id } => ServerResponse::NeedsReviewer { coder, review_id },
    }
}

fn adapt_authentication_response(response: authentication::AuthenticationResponse) -> ServerResponse {
    match response {
        authentication::AuthenticationResponse::KnownIdentity { coder } => ServerResponse::KnownIdentity { username: coder },
        authentication::AuthenticationResponse::UnknownIdentity {} => ServerResponse::UnknownIdentity {},
    }
}

fn adapt_authentication_request(request: &ServerRequest, token: String) -> Option<authentication::AuthenticationRequest> {
    match request {
        &ServerRequest::LoadIdentity {} => Some(authentication::AuthenticationRequest::LoadIdentity { token }),
        &ServerRequest::SendIdentity {
            token: ref sent_token,
        } => Some(authentication::AuthenticationRequest::SendIdentity {
            token: sent_token.clone(),
        }),
        _ => None,
    }
}
