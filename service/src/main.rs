extern crate iron;
extern crate mount;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate staticfile;

use application::Application;
use application::ApplicationConfiguration;
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

static CONFIG_FILE_NAME: &str = "find-reviewer.json";

type SharedApplication = Arc<Mutex<Application>>;

fn main() {
    let configuration = load_configuration();
    save_configuration(&configuration);

    let address = configuration.address.clone();
    let application = SharedApplication::new(Mutex::new(Application::new(configuration)));

    start_timeout_loop(application.clone());
    start_service(&address, application);
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

fn start_service(address: &str, application: SharedApplication) {
    let mut mount = Mount::new();
    mount
        .mount("/find-reviewer", move |request: &mut Request| Ok(process_request(request, &application)))
        .mount("/", Static::new(Path::new("www")));

    Iron::new(mount).http(address).unwrap();
}

fn process_request(request: &mut Request, application: &SharedApplication) -> Response {
    if request.method != Method::Post {
        return Response::with((Status::BadRequest, "Must be a POST request"));
    }

    let parsed = match serde_json::from_reader(request.body.by_ref()) {
        Err(message) => return Response::with((Status::BadRequest, format!("JSON error: {}", message))),
        Ok(request) => request,
    };

    let response = application.lock().unwrap().dispatch_request(parsed);

    Response::with((Status::Ok, serde_json::to_string_pretty(&response).unwrap()))
}
