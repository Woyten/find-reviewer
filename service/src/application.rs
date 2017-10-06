use rand;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum FindReviewerRequest {
    NeedReviewer { coder: String },
    HaveTimeForReview { reviewer: String },
    WillReview { review_id: u32 },
    WontReview { review_id: u32 },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum FindReviewerResponse {
    Accepted {},
    NoReviewerNeeded {},
    AlreadyRegistered {},
    NeedsReviewer { coder: String, review_id: u32 },
    ReviewNotFound {},
}

#[derive(Deserialize, Serialize)]
pub struct ApplicationConfiguration {
    pub address: String,
    pub timeout_in_s: u64,
    pub wip_limit: usize,
}

impl Default for ApplicationConfiguration {
    fn default() -> Self {
        ApplicationConfiguration {
            address: String::from("localhost:3000"),
            timeout_in_s: 30,
            wip_limit: 5,
        }
    }
}

#[derive(Clone)]
struct Review {
    coder: String,
    enqueued_coder: Option<String>,
    started: Instant,
}

pub struct Application {
    configuration: ApplicationConfiguration,
    waiting_coders: HashSet<String>,
    active_reviews: HashMap<u32, Review>,
}

impl Application {
    pub fn new(configuration: ApplicationConfiguration) -> Application {
        Application {
            configuration,
            waiting_coders: HashSet::new(),
            active_reviews: HashMap::new(),
        }
    }

    pub fn dispatch_request(&mut self, request: FindReviewerRequest) -> FindReviewerResponse {
        match request {
            FindReviewerRequest::NeedReviewer { coder } => self.need_reviewer(coder),
            FindReviewerRequest::HaveTimeForReview { reviewer } => self.have_time_for_review(&reviewer),
            FindReviewerRequest::WillReview { review_id } => self.will_review(review_id),
            FindReviewerRequest::WontReview { review_id } => self.wont_review(review_id),
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
        self.waiting_coders.contains(coder)
            || self.active_reviews
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

    pub fn process_timeouts(&mut self) {
        let now = Instant::now();
        let timed_out: Vec<_> = self.active_reviews
            .iter()
            .filter(|&(_, review)| (now - review.started).as_secs() > self.configuration.timeout_in_s)
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

        let request = create_need_reviewer_request("coder1");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::Accepted {});

        let request = create_need_reviewer_request("coder1");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::AlreadyRegistered {});

        let request = create_need_reviewer_request("coder2");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::Accepted {});

        let request = create_need_reviewer_request("coder3");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::Accepted {});

        let request = create_need_reviewer_request("coder2");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::AlreadyRegistered {});

        let request = create_need_reviewer_request("coder3");
        assert_eq!(app.dispatch_request(request), FindReviewerResponse::AlreadyRegistered {});
    }

    #[test]
    fn respect_wip_limit() {
        let mut application = create_application();
        let coders = (0..5)
            .map(|x| format!("coder{}", x))
            .collect::<HashSet<_>>();
        for coder in &coders {
            let request = create_need_reviewer_request(coder);
            assert_eq!(application.dispatch_request(request), FindReviewerResponse::Accepted {});
        }
        let answer = application.dispatch_request(create_need_reviewer_request("anothercoder"));
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
