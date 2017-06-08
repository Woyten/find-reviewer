module Response exposing (Response(..), responseDecoder)

import Json.Decode as Decode
import Dict


type Response
    = Accepted
    | AlreadyRegistered
    | NoReviewerNeeded
    | NeedsReviewer String Int
    | ReviewNotFound


responseDecoder =
    Decode.oneOf [ stringDecoder, objectDecoder ]


requestsWithoutParams =
    [ Accepted, AlreadyRegistered, NoReviewerNeeded, ReviewNotFound ]


dict =
    requestsWithoutParams
        |> List.map (\x -> ( toString x, x ))
        |> Dict.fromList


stringDecoder =
    Decode.string
        |> Decode.map resolveStatus
        |> Decode.andThen unwrap


resolveStatus status =
    dict |> Dict.get status


unwrap value =
    value
        |> Maybe.map Decode.succeed
        |> Maybe.withDefault (Decode.fail <| "Expecting one of " ++ (toString requestsWithoutParams))


objectDecoder =
    Decode.field "NeedsReviewer" needsReviewerDecoder


needsReviewerDecoder =
    Decode.map2 NeedsReviewer
        (Decode.field "coder" Decode.string)
        (Decode.field "review_id" Decode.int)
