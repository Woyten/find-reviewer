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
    let application = SharedApplication::new(Mutex::new(Application::new(configuration)));

    let mut mount = Mount::new();
    mount
        .mount("/find-reviewer", {
            let application = application.clone();
            move |request: &mut Request| Ok(process_request(request, &application))
        })
        .mount("/", Static::new(Path::new("www")));

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        application.lock().unwrap().process_timeouts();
    });

    Iron::new(mount).http("localhost:3000").unwrap();
}

fn load_configuration() -> ApplicationConfiguration {
    let config = File::open(CONFIG_FILE_NAME)
        .map(|open_file| serde_json::from_reader(open_file).expect(&format!("Could not parse {}", CONFIG_FILE_NAME)))
        .unwrap_or_else(|err| {
            println!("Could not read {}: {}\nFile will be created", CONFIG_FILE_NAME, err);
            ApplicationConfiguration::default()
        });

    File::create(CONFIG_FILE_NAME)
        .map(|created_file| {
            serde_json::to_writer_pretty(created_file, &config).expect(&format!("Could not serialize {}", CONFIG_FILE_NAME))
        })
        .unwrap_or_else(|err| println!("Could not write {}: {}", CONFIG_FILE_NAME, err));

    config
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
