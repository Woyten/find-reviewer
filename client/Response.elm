module Response exposing (Response(..), decode)

import Json.Decode as Decode


type Response
    = Accepted
    | AlreadyRegistered
    | NoReviewerNeeded
    | NeedsReviewer String Int
    | ReviewNotFound


decode =
    Decode.oneOf
        [ Decode.field "Accepted" (Decode.succeed Accepted)
        , Decode.field "AlreadyRegistered" (Decode.succeed AlreadyRegistered)
        , Decode.field "NoReviewerNeeded" (Decode.succeed NoReviewerNeeded)
        , Decode.field "NeedsReviewer" decodeNeedsReviewer
        , Decode.field "ReviewNotFound" (Decode.succeed ReviewNotFound)
        ]


decodeNeedsReviewer =
    Decode.map2 NeedsReviewer
        (Decode.field "coder" Decode.string)
        (Decode.field "review_id" Decode.int)
