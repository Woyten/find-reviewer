module Request exposing (..)

import Json.Encode as Encode


type ServerMessage
    = NeedReviewer String
    | WontReview Int
    | WillReview Int
    | HaveTimeForReview String


encodeRequest msg =
    case msg of
        NeedReviewer coder ->
            Encode.object [ ( "NeedsReviewer", Encode.string coder ) ]

        WontReview id ->
            Encode.object [ ( "WontReview", Encode.int id ) ]

        WillReview id ->
            Encode.object [ ( "WillReview", Encode.int id ) ]

        HaveTimeForReview coder ->
            Encode.object [ ( "HaveTimeForReview", Encode.string coder ) ]
