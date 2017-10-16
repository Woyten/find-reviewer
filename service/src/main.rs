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
use application::FindReviewerRequest;
use application::FindReviewerResponse;
use authentication::Authentication;
use authentication::UserDatabase;
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
static USER_DATABASE_NAME: &str = "find-reviewer-users.json";

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

    let user_database = load_user_database();

    let address = configuration.address.clone();
    let application = SharedApplication::new(Mutex::new(Application::new(configuration)));
    let authentication = SharedAuthentication::new(Mutex::new(Authentication::new(user_database)));

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

fn load_user_database() -> UserDatabase {
    File::open(USER_DATABASE_NAME)
        .map(|open_file| serde_json::from_reader(open_file).expect(&format!("Could not parse {}", USER_DATABASE_NAME)))
        .unwrap_or_else(|err| {
            println!("Could not read {}, error: {}", USER_DATABASE_NAME, err);
            panic!()
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

    let token = extract_token_from_cookie(request);

    let server_response = distribute_request_under_services(&parsed, &token, application, authentication);
    let mut resp = Response::with((Status::Ok, serde_json::to_string_pretty(&server_response).unwrap()));

    set_token_cookie(&mut resp, parsed, server_response, token);
    resp
}

fn distribute_request_under_services(
    parsed: &ServerRequest,
    token: &Option<String>,
    application: &SharedApplication,
    authentication: &SharedAuthentication,
) -> ServerResponse {
    match token {
        &Some(ref token) => authentication
            .lock()
            .unwrap()
            .process_request(token)
            .map_or(ServerResponse::UnknownIdentity {}, |coder| {
                adapt_application_request(&parsed, &coder).map_or(ServerResponse::KnownIdentity { username: coder }, |app_request| {
                    adapt_application_response(application.lock().unwrap().dispatch_request(app_request))
                })
            }),
        &None => match parsed {
            &ServerRequest::SendIdentity { ref token } => authentication
                .lock()
                .unwrap()
                .process_request(token)
                .map_or(ServerResponse::UnknownIdentity {}, |coder| ServerResponse::KnownIdentity { username: coder }),
            _ => ServerResponse::UnknownIdentity {},
        },
    }
}

fn set_token_cookie(resp: &mut Response, parsed: ServerRequest, server_response: ServerResponse, token: Option<String>) {
    let time = time::now() + time::Duration::weeks(4);
    match get_most_current_token(parsed, server_response, token) {
        Some(send_token) => resp.headers.set(SetCookie(vec![
            format!("token={}; Path=/find-reviewer; Expires={}", send_token, time.rfc822()),
        ])),
        None => (),
    }
}

fn get_most_current_token(request: ServerRequest, response: ServerResponse, token_from_cookie: Option<String>) -> Option<String> {
    match response {
        ServerResponse::UnknownIdentity {} => None,
        _ => match request {
            ServerRequest::SendIdentity { token } => Some(token),
            _ => match token_from_cookie {
                Some(_) => token_from_cookie,
                None => None,
            },
        },
    }
}

fn extract_token_from_cookie<'a>(request: &'a Request) -> Option<String> {
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

fn adapt_application_request(request: &ServerRequest, coder: &String) -> Option<FindReviewerRequest> {
    match request {
        &ServerRequest::NeedReviewer {} => Some(FindReviewerRequest::NeedReviewer {
            coder: coder.clone(),
        }),
        &ServerRequest::HaveTimeForReview {} => Some(FindReviewerRequest::HaveTimeForReview {
            reviewer: coder.clone(),
        }),
        &ServerRequest::WillReview { review_id } => Some(FindReviewerRequest::WillReview { review_id }),
        &ServerRequest::WontReview { review_id } => Some(FindReviewerRequest::WontReview { review_id }),
        _ => None,
    }
}

fn adapt_application_response(response: FindReviewerResponse) -> ServerResponse {
    match response {
        FindReviewerResponse::Accepted {} => ServerResponse::Accepted {},
        FindReviewerResponse::AlreadyRegistered {} => ServerResponse::AlreadyRegistered {},
        FindReviewerResponse::NoReviewerNeeded {} => ServerResponse::NoReviewerNeeded {},
        FindReviewerResponse::ReviewNotFound {} => ServerResponse::ReviewNotFound {},
        FindReviewerResponse::NeedsReviewer { coder, review_id } => ServerResponse::NeedsReviewer { coder, review_id },
    }
}
