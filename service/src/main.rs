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
use authentication::UserDatabase;
use iron::headers::Cookie;
use iron::headers::SetCookie;
use iron::method::Method;
use iron::prelude::*;
use iron::status::Status;
use mount::Mount;
use shared::*;
use staticfile::Static;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;


mod application;
mod authentication;
mod shared;

static CONFIG_FILE_NAME: &str = "find-reviewer.json";
static USER_DATABASE_NAME: &str = "find-reviewer-users.json";

type SharedApplication = Arc<Mutex<Application>>;

fn main() {
    let configuration = load_configuration();
    save_configuration(&configuration);

    let user_database = load_user_database();
    save_user_database(&user_database);

    let address = configuration.address.clone();
    let application = SharedApplication::new(Mutex::new(Application::new(configuration)));
    let authentication = Authentication::new(user_database);

    start_timeout_loop(application.clone());
    start_service(&address, application, authentication);
}

fn load_configuration() -> ApplicationConfiguration {
    File::open(CONFIG_FILE_NAME)
        .map(|open_file| serde_json::from_reader(open_file).expect(&format!("Could not parse {}", CONFIG_FILE_NAME)))
        .unwrap_or_else(|err| {
            println!("Could not read {}: {}\nFile will be created.", CONFIG_FILE_NAME, err);
            ApplicationConfiguration::default()
        })
}

fn load_user_database() -> UserDatabase {
    File::open(USER_DATABASE_NAME)
        .map(|open_file| serde_json::from_reader(open_file).expect(&format!("Could not parse {}", USER_DATABASE_NAME)))
        .unwrap_or_else(|err| {
            println!("Could not read {}: {}\nFile will be created.", USER_DATABASE_NAME, err);
            UserDatabase::default()
        })
}

fn save_configuration(configuration: &ApplicationConfiguration) {
    File::create(CONFIG_FILE_NAME)
        .map(|created_file| {
            serde_json::to_writer_pretty(created_file, &configuration).expect(&format!("Could not serialize {}", CONFIG_FILE_NAME))
        })
        .unwrap_or_else(|err| println!("Could not write {}: {}", CONFIG_FILE_NAME, err));
}

fn save_user_database(user_database: &UserDatabase) {
    File::create(USER_DATABASE_NAME)
        .map(|created_file| {
            serde_json::to_writer_pretty(created_file, &user_database).expect(&format!("Could not serialize {}", USER_DATABASE_NAME))
        })
        .unwrap_or_else(|err| println!("Could not write {}: {}", USER_DATABASE_NAME, err));
}

fn start_timeout_loop(application: SharedApplication) {
    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(1));
        application.lock().unwrap().process_timeouts();
    });
}

fn start_service(address: &str, application: SharedApplication, authentication: Authentication) {
    let mut mount = Mount::new();
    mount
        .mount("/find-reviewer", move |request: &mut Request| Ok(process_request(request, &application, &authentication)))
        .mount("/", Static::new(Path::new("www")));

    Iron::new(mount).http(address).unwrap();
}

fn process_request(request: &mut Request, application: &SharedApplication, authentication: &Authentication) -> Response {
    if request.method != Method::Post {
        return Response::with((Status::BadRequest, "Must be a POST request"));
    }

    let parsed = match serde_json::from_reader(request.body.by_ref()) {
        Err(message) => return Response::with((Status::BadRequest, format!("JSON error: {}", message))),
        Ok(request) => request,
    };

    let token = extract_token_from_cookie(request);

    let (server_response, cookie) = distribute_request_under_services(&parsed, &token, application, authentication);
    let mut resp = Response::with((Status::Ok, serde_json::to_string_pretty(&server_response).unwrap()));

    set_cookie_to_token(&mut resp, cookie);
    resp
}

fn distribute_request_under_services(
    parsed: &ServerRequest,
    token: &Option<String>,
    application: &SharedApplication,
    authentication: &Authentication,
) -> (ServerResponse, Option<String>) {
    match token {
        &Some(ref token) => authentication
            .process_request(token)
            .map_or((ServerResponse::UnknownIdentity {}, None), |coder| {
                (
                    match parsed {
                        &ServerRequest::NeedReviewer {} => application.lock().unwrap().need_reviewer(coder.clone()),
                        &ServerRequest::HaveTimeForReview {} => application.lock().unwrap().have_time_for_review(coder),
                        &ServerRequest::WillReview { review_id } => application.lock().unwrap().will_review(review_id),
                        &ServerRequest::WontReview { review_id } => application.lock().unwrap().wont_review(review_id),
                        _ => ServerResponse::KnownIdentity {
                            username: coder.clone(),
                        },
                    },
                    Some(token.clone()),
                )
            }),
        &None => match parsed {
            &ServerRequest::SendIdentity { ref token } => authentication
                .process_request(token)
                .map_or((ServerResponse::UnknownIdentity {}, None), |coder| {
                    (
                        ServerResponse::KnownIdentity {
                            username: coder.clone(),
                        },
                        Some(token.clone()),
                    )
                }),
            _ => (ServerResponse::UnknownIdentity {}, None),
        },
    }
}

fn set_cookie_to_token(resp: &mut Response, token: Option<String>) {
    let time = time::now() + time::Duration::weeks(4);
    token.map(|value| {
        resp.headers.set(SetCookie(vec![
            format!("token={}; Path=/find-reviewer; Expires={}", value, time.rfc822()),
        ]))
    });
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
