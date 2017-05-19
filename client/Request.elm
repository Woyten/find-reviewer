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
            Encode.object
                [ ( "NeedReviewer"
                  , Encode.object
                        [ ( "coder", Encode.string coder ) ]
                  )
                ]

        WontReview id ->
            Encode.object
                [ ( "WontReview"
                  , Encode.object
                        [ ( "review_id", Encode.int id ) ]
                  )
                ]

        WillReview id ->
            Encode.object
                [ ( "WillReview"
                  , Encode.object
                        [ ( "review_id", Encode.int id ) ]
                  )
                ]

        HaveTimeForReview reviewer ->
            Encode.object
                [ ( "HaveTimeForReview"
                  , Encode.object
                        [ ( "reviewer", Encode.string reviewer ) ]
                  )
                ]
