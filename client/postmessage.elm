module Main exposing (..)

import Json.Encode exposing (..)
import Html exposing (..)


type Message
    = NeedsReview String
    | WontReview
    | WillReview Int
    | HaveTime


main =
    text (encode 0 repres)

msg =
    NeedsReview "philipp"

repres =
    case msg of
        NeedsReview coder ->
            Json.Encode.object [ ( "NeedsReview", Json.Encode.string coder ) ]

        WontReview ->
            Json.Encode.string "WontReview"

        WillReview reviewId ->
            Json.Encode.object [ ( "WillReview", Json.Encode.int reviewId ) ]

        HaveTime ->
            Json.Encode.string "HaveTime"
