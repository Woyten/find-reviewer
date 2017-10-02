module Response exposing (Response(..), decode)

import Json.Decode as Decode


type Response
    = KnownIdentity String
    | UnknownIdentity
    | Accepted
    | AlreadyRegistered
    | NoReviewerNeeded
    | NeedsReviewer String Int
    | ReviewNotFound


decode : Decode.Decoder Response
decode =
    Decode.oneOf
        [ Decode.field "KnownIdentity" <|
            Decode.map KnownIdentity
                (Decode.field "username" Decode.string)
        , Decode.field "UnknownIdentity" <| Decode.succeed UnknownIdentity
        , Decode.field "Accepted" <| Decode.succeed Accepted
        , Decode.field "AlreadyRegistered" <| Decode.succeed AlreadyRegistered
        , Decode.field "NoReviewerNeeded" <| Decode.succeed NoReviewerNeeded
        , Decode.field "NeedsReviewer" <|
            Decode.map2 NeedsReviewer
                (Decode.field "coder" Decode.string)
                (Decode.field "review_id" Decode.int)
        , Decode.field "ReviewNotFound" <| Decode.succeed ReviewNotFound
        ]
