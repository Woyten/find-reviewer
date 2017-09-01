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
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Deserialize, Eq, PartialEq)]
enum FindReviewerRequest {
    NeedReviewer { coder: String },
    HaveTimeForReview { reviewer: String },
    WillReview { review_id: usize },
    WontReview { review_id: usize },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
enum FindReviewerResponse {
    Accepted,
    NoReviewerNeeded,
    AlreadyRegistered,
    NeedsReviewer { coder: String, review_id: usize },
    ReviewNotFound,
}

type SharedApplication = Arc<Mutex<Application<RandomIdGenerator>>>;

fn main() {
    let application = SharedApplication::new(Mutex::new(Application::new(ApplicationConfiguration::default())));

    let mut mount = Mount::new();
    mount
        .mount("/find-reviewer", {
            let application = application.clone();
            move |request: &mut Request| Ok(dispatch_request(request, &application))
        })
        .mount("/", Static::new(Path::new("www")));

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        application.lock().unwrap().process_timeouts();
    });

    Iron::new(mount).http("localhost:3000").unwrap();
}

fn dispatch_request(request: &mut Request, application: &SharedApplication) -> Response {
    if request.method != Method::Post {
        return Response::with((Status::BadRequest, "Must be a POST request"));
    }

    let parsed = match serde_json::from_reader(request.body.by_ref()) {
        Err(message) => return Response::with((Status::BadRequest, format!("JSON error: {}", message))),
        Ok(request) => request,
    };

    let response = {
        let mut application = application.lock().unwrap();
        match parsed {
            FindReviewerRequest::NeedReviewer { coder } => application.need_reviewer(coder),
            FindReviewerRequest::HaveTimeForReview { reviewer } => application.have_time_for_review(&reviewer),
            FindReviewerRequest::WillReview { review_id } => application.will_review(review_id),
            FindReviewerRequest::WontReview { review_id } => application.wont_review(review_id),
        }
    };

    Response::with((Status::Ok, serde_json::to_string_pretty(&response).unwrap()))
}

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

struct Application<G> {
    configuration: ApplicationConfiguration,
    waiting_coders: HashSet<String, BuildHasherDefault<DefaultHasher>>,
    active_reviews: HashMap<usize, Review>,
    id_generator: G,
}

struct ApplicationConfiguration {
    pub timeout: Duration,
    pub wip_limit: usize,
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

impl<G: IdGenerator> Application<G> {
    fn new(configuration: ApplicationConfiguration) -> Application<G> {
        Application {
            configuration,
            waiting_coders: HashSet::default(),
            active_reviews: HashMap::default(),
            id_generator: G::new(),
        }
    }

    fn need_reviewer(&mut self, incoming_coder: String) -> FindReviewerResponse {
        if self.is_already_registered(&incoming_coder) {
            FindReviewerResponse::AlreadyRegistered
        } else if self.waiting_coders.len() >= self.configuration.wip_limit {
            let random_waiting_coder = self.waiting_coders.iter().next().unwrap().clone();
            self.start_review(random_waiting_coder, Some(incoming_coder))
        } else {
            self.waiting_coders.insert(incoming_coder);
            FindReviewerResponse::Accepted
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
            None => FindReviewerResponse::NoReviewerNeeded,
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

    fn generate_id(&mut self) -> usize {
        loop {
            let id = self.id_generator.generate_id();
            if !self.active_reviews.contains_key(&id) {
                return id;
            }
        }
    }

    fn will_review(&mut self, review_id: usize) -> FindReviewerResponse {
        match self.active_reviews.remove(&review_id) {
            Some(review) => {
                review
                    .enqueued_coder
                    .map(|coder| self.waiting_coders.insert(coder));
                FindReviewerResponse::Accepted
            }
            None => FindReviewerResponse::ReviewNotFound,
        }
    }

    fn wont_review(&mut self, review_id: usize) -> FindReviewerResponse {
        match self.active_reviews.remove(&review_id) {
            Some(review) => {
                self.waiting_coders.insert(review.coder);
                FindReviewerResponse::Accepted
            }
            None => FindReviewerResponse::ReviewNotFound,
        }
    }

    fn remove_coder(&mut self, coder: &String) {
        self.waiting_coders.remove(coder);
    }

    fn insert_review(&mut self, review: Review) -> usize {
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

trait IdGenerator {
    fn generate_id(&mut self) -> usize;

    fn new() -> Self;
}

struct RandomIdGenerator;

impl IdGenerator for RandomIdGenerator {
    fn generate_id(&mut self) -> usize {
        rand::thread_rng().gen_range(0, 16383)
    }

    fn new() -> Self {
        RandomIdGenerator
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct SequenceIdGenerator {
        id: usize,
    }

    impl IdGenerator for SequenceIdGenerator {
        fn generate_id(&mut self) -> usize {
            self.id += 1;
            self.id
        }

        fn new() -> Self {
            SequenceIdGenerator { id: 0 }
        }
    }

    #[test]
    fn add_coder_no_more_than_once() {
        let mut app = create_application();

        let resp = app.need_reviewer("coder1".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder1".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered);

        let resp = app.need_reviewer("coder2".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder3".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder2".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered);

        let resp = app.need_reviewer("coder3".to_owned());
        assert_eq!(resp, FindReviewerResponse::AlreadyRegistered);
    }

    use rand::SeedableRng;

    #[test]
    fn respect_wip_limit() {
        rand::weak_rng().reseed([1, 2, 3, 4]);

        let mut app = create_application();

        let resp = app.need_reviewer("coder1".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder2".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder3".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder4".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder5".to_owned());
        assert_eq!(resp, FindReviewerResponse::Accepted);

        let resp = app.need_reviewer("coder6".to_owned());
        assert_eq!(
            resp,
            FindReviewerResponse::NeedsReviewer {
                coder: "coder3".to_owned(),
                review_id: 1,
            }
        );
    }

    fn create_application() -> Application<SequenceIdGenerator> {
        Application::new(ApplicationConfiguration::default())
    }
}
