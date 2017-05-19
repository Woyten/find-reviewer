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
use staticfile::Static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;

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
    NoHelpNeeded,
    AlreadyRegistered,
    NeedsReview { coder: String, review_id: usize },
    ReviewNotFound,
}

type Application = Mutex<ApplicationState<RandomIdGenerator>>;

fn main() {
    let application = Mutex::new(ApplicationState::new());

    let mut mount = Mount::new();
    mount.mount("/find-reviewer", move |request: &mut Request| find_reviewer(request, &application)).mount("/", Static::new(Path::new("www")));

    // TODO: Handle timeouts middleware, log state middleware

    Iron::new(mount).http("localhost:3000").unwrap();
}

fn find_reviewer(request: &mut Request, application: &Application) -> IronResult<Response> {
    match request.method {
        Method::Post => dispatch(request, application),
        _ => Ok(Response::with((Status::BadRequest, "Must be a POST request"))),
    }

}

fn dispatch(request: &mut Request, application: &Application) -> IronResult<Response> {
    let parsed: FindReviewerRequest = match serde_json::from_reader(request.body.by_ref()) {
        Ok(request) => request,
        Err(message) => return Ok(Response::with((Status::BadRequest, format!("JSON error: {}", message)))),
    }; // TODO: map_err

    let response = {
        let mut state = application.lock().unwrap();
        match parsed {
            FindReviewerRequest::NeedReviewer { coder } => state.need_reviewer(coder),
            FindReviewerRequest::HaveTimeForReview { reviewer } => state.have_time_for_review(&reviewer),
            FindReviewerRequest::WillReview { review_id } => state.will_review(review_id),
            FindReviewerRequest::WontReview { review_id } => state.wont_review(review_id),
        }
    };

    Ok(Response::with((Status::Ok, serde_json::to_string_pretty(&response).unwrap())))
}

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

struct ApplicationState<G> {
    waiting_coders: HashSet<String, BuildHasherDefault<DefaultHasher>>,
    open_reviews: HashMap<usize, Review>,
    id_generator: G,
}

#[derive(Clone)]
struct Review {
    pub coder: String,
    pub enqueued_coder: Option<String>, 
    // start_time
}


impl<G: IdGenerator> ApplicationState<G> {
    fn new() -> ApplicationState<G> {
        ApplicationState {
            waiting_coders: HashSet::default(),
            open_reviews: HashMap::default(),
            id_generator: G::new(),
        }
    }

    fn need_reviewer(&mut self, incoming_coder: String) -> FindReviewerResponse {
        if self.is_already_registered(&incoming_coder) {
            FindReviewerResponse::AlreadyRegistered
        } else if self.waiting_coders.len() >= 5 {
            let random_waiting_coder = self.waiting_coders
                .iter()
                .next()
                .unwrap()
                .clone();
            self.start_review(random_waiting_coder, Some(incoming_coder))
        } else {
            self.insert_coder(Some(incoming_coder));
            FindReviewerResponse::Accepted
        }
    }

    fn is_already_registered(&self, coder: &String) -> bool {
        self.waiting_coders.contains(coder) || self.open_reviews.values().any(|review| &review.coder == coder || review.enqueued_coder.as_ref() == Some(coder))
    }

    fn have_time_for_review(&mut self, incoming_reviewer: &String) -> FindReviewerResponse {
        let random_coder_except_incoming_reviewer = self.waiting_coders
            .iter()
            .filter(|&coder| coder != incoming_reviewer)
            .next()
            .cloned();
        match random_coder_except_incoming_reviewer {
            Some(coder) => self.start_review(coder, None),
            None => FindReviewerResponse::NoHelpNeeded,
        }
    }

    fn start_review(&mut self, coder: String, enqueued_coder: Option<String>) -> FindReviewerResponse {
        let review = Review {
            coder: coder.clone(),
            enqueued_coder,
        };
        self.remove_coder(&coder);
        let review_id = self.insert_review(review);

        FindReviewerResponse::NeedsReview { coder, review_id }
    }

    fn generate_id(&mut self) -> usize {
        loop {
            let id = self.id_generator.generate_id();
            if !self.open_reviews.contains_key(&id) {
                return id;
            }
        }
    }

    fn will_review(&mut self, review_id: usize) -> FindReviewerResponse {
        match self.open_reviews.get(&review_id).cloned() { // FIXME: Superfluous clone
            Some(review) => {
                self.remove_review(review_id);
                self.insert_coder(review.enqueued_coder);
                FindReviewerResponse::Accepted
            }
            None => FindReviewerResponse::ReviewNotFound,
        }
    }

    fn wont_review(&mut self, review_id: usize) -> FindReviewerResponse {
        match self.open_reviews.get(&review_id).cloned() { // FIXME: Superfluous clone
            Some(review) => {
                self.remove_review(review_id);
                self.insert_coder(Some(review.coder));
                FindReviewerResponse::Accepted
            }
            None => FindReviewerResponse::ReviewNotFound,
        }
    }

    fn insert_coder(&mut self, coder: Option<String>) {
        if let Some(coder) = coder {
            self.waiting_coders.insert(coder.clone());
        }
    }

    fn remove_coder(&mut self, coder: &String) {
        self.waiting_coders.remove(coder);
    }

    fn insert_review(&mut self, review: Review) -> usize {
        let id = self.generate_id();
        self.open_reviews.insert(id, review);
        id
    }

    fn remove_review(&mut self, issue_id: usize) {
        self.open_reviews.remove(&issue_id);
    }
}

trait IdGenerator {
    fn generate_id(&mut self) -> usize;

    fn new() -> Self;
}

struct RandomIdGenerator;

impl IdGenerator for RandomIdGenerator {
    fn generate_id(&mut self) -> usize {
        rand::random()
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
    fn coders_can_only_be_added_once() {
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
    fn wip_limit_is_respected() {
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
        assert_eq!(resp,
                   FindReviewerResponse::NeedsReview {
                       coder: "coder3".to_owned(),
                       review_id: 1,
                   });
    }

    fn create_application() -> ApplicationState<SequenceIdGenerator> {
        ApplicationState::new()
    }

    impl ApplicationState<SequenceIdGenerator> {}
}
