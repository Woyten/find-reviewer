extern crate iron;
extern crate mount;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate staticfile;

use iron::method::Method;
use iron::prelude::*;
use iron::status::Status;
use mount::Mount;
use rand::Rng;
use staticfile::Static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

static CONFIG_FILE_NAME: &str = "find-reviewer.json";

#[derive(Debug, Deserialize, Eq, PartialEq)]
enum FindReviewerRequest {
    NeedReviewer { coder: String },
    HaveTimeForReview { reviewer: String },
    WillReview { review_id: u32 },
    WontReview { review_id: u32 },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
enum FindReviewerResponse {
    Accepted {},
    NoReviewerNeeded {},
    AlreadyRegistered {},
    NeedsReviewer { coder: String, review_id: u32 },
    ReviewNotFound {},
}

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

    let response = dispatch_request(&mut application.lock().unwrap(), parsed);

    Response::with((Status::Ok, serde_json::to_string_pretty(&response).unwrap()))
}

fn dispatch_request(application: &mut Application, request: FindReviewerRequest) -> FindReviewerResponse {
    match request {
        FindReviewerRequest::NeedReviewer { coder } => application.need_reviewer(coder),
        FindReviewerRequest::HaveTimeForReview { reviewer } => application.have_time_for_review(&reviewer),
        FindReviewerRequest::WillReview { review_id } => application.will_review(review_id),
        FindReviewerRequest::WontReview { review_id } => application.wont_review(review_id),
    }
}

struct Application {
    configuration: ApplicationConfiguration,
    waiting_coders: HashSet<String>,
    active_reviews: HashMap<u32, Review>,
}

#[derive(Serialize, Deserialize)]
struct ApplicationConfiguration {
    timeout: Duration,
    wip_limit: usize,
}

impl Default for ApplicationConfiguration {
    fn default() -> Self {
        ApplicationConfiguration {
            timeout: Duration::from_secs(30),
            wip_limit: 5,
        }
    }
}

#[derive(Clone)]
struct Review {
    pub coder: String,
    pub enqueued_coder: Option<String>,
    pub started: Instant,
}

impl Application {
    fn new(configuration: ApplicationConfiguration) -> Application {
        Application {
            configuration,
            waiting_coders: HashSet::new(),
            active_reviews: HashMap::new(),
        }
    }

    fn need_reviewer(&mut self, incoming_coder: String) -> FindReviewerResponse {
        if self.is_already_registered(&incoming_coder) {
            FindReviewerResponse::AlreadyRegistered {}
        } else if self.waiting_coders.len() >= self.configuration.wip_limit {
            let random_waiting_coder = self.waiting_coders.iter().next().unwrap().clone();
            self.start_review(random_waiting_coder, Some(incoming_coder))
        } else {
            self.waiting_coders.insert(incoming_coder);
            FindReviewerResponse::Accepted {}
        }
    }

    fn is_already_registered(&self, coder: &String) -> bool {
        self.waiting_coders.contains(coder) ||
            self.active_reviews
                .values()
                .any(|review| &review.coder == coder || review.enqueued_coder.as_ref() == Some(coder))
    }

    fn have_time_for_review(&mut self, incoming_reviewer: &String) -> FindReviewerResponse {
        let random_coder_except_incoming_reviewer = self.waiting_coders
            .iter()
            .filter(|&coder| coder != incoming_reviewer)
            .next()
            .cloned();
        match random_coder_except_incoming_reviewer {
            Some(coder) => self.start_review(coder, None),
            None => FindReviewerResponse::NoReviewerNeeded {},
        }
    }

    fn start_review(&mut self, coder: String, enqueued_coder: Option<String>) -> FindReviewerResponse {
        let review = Review {
            coder: coder.clone(),
            enqueued_coder,
            started: Instant::now(),
        };
        self.remove_coder(&coder);
        let review_id = self.insert_review(review);

        FindReviewerResponse::NeedsReviewer { coder, review_id }
    }

    fn generate_id(&mut self) -> u32 {
        loop {
            let id = rand::thread_rng().gen();
            if !self.active_reviews.contains_key(&id) {
                return id;
            }
        }
    }

    fn will_review(&mut self, review_id: u32) -> FindReviewerResponse {
        match self.active_reviews.remove(&review_id) {
            Some(review) => {
                review
                    .enqueued_coder
                    .map(|coder| self.waiting_coders.insert(coder));
                FindReviewerResponse::Accepted {}
            }
            None => FindReviewerResponse::ReviewNotFound {},
        }
    }

    fn wont_review(&mut self, review_id: u32) -> FindReviewerResponse {
        match self.active_reviews.remove(&review_id) {
            Some(review) => {
                self.waiting_coders.insert(review.coder);
                FindReviewerResponse::Accepted {}
            }
            None => FindReviewerResponse::ReviewNotFound {},
        }
    }

    fn remove_coder(&mut self, coder: &String) {
        self.waiting_coders.remove(coder);
    }

    fn insert_review(&mut self, review: Review) -> u32 {
        let id = self.generate_id();
        self.active_reviews.insert(id, review);
        id
    }

    fn process_timeouts(&mut self) {
        let now = Instant::now();
        let timed_out: Vec<_> = self.active_reviews
            .iter()
            .filter(|&(_, review)| (now - review.started) > self.configuration.timeout)
            .map(|(&id, _)| id)
            .collect();
        for review_id in timed_out {
            self.wont_review(review_id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_coder_no_more_than_once() {
        let mut app = create_application();

        let resp = app.need_reviewer("coder1".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted {});

        let resp = app.need_reviewer("coder1".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered {});

        let resp = app.need_reviewer("coder2".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted {});

        let resp = app.need_reviewer("coder3".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted {});

        let resp = app.need_reviewer("coder2".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered {});

        let resp = app.need_reviewer("coder3".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered {});
    }

    #[test]
    fn respect_wip_limit() {
        let mut application = create_application();
        let coders = (0..5)
            .map(|x| format!("coder{}", x))
            .collect::<HashSet<_>>();
        for coder in &coders {
            let request = create_need_reviewer_request(coder);
            assert_eq!(dispatch_request(&mut application, request), FindReviewerResponse::Accepted {});
        }
        let answer = dispatch_request(&mut application, create_need_reviewer_request("anothercoder"));
        match answer {
            FindReviewerResponse::NeedsReviewer {
                coder,
                review_id: _,
            } => assert!(coders.contains(&coder)),
            _ => panic!(),
        }
    }

    fn create_application() -> Application {
        Application::new(ApplicationConfiguration::default())
    }

    fn create_need_reviewer_request(coder: &str) -> FindReviewerRequest {
        FindReviewerRequest::NeedReviewer {
            coder: coder.into(),
        }
    }
}
