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
