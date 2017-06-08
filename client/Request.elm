module Request exposing (Request(..), encodeRequest)

import Json.Encode as Encode


type Request
    = NeedReviewer String
    | HaveTimeForReview String
    | WontReview Int
    | WillReview Int


encodeRequest msg =
    case msg of
        NeedReviewer coder ->
            Encode.object
                [ ( "NeedReviewer"
                  , Encode.object
                        [ ( "coder", Encode.string coder ) ]
                  )
                ]

        HaveTimeForReview reviewer ->
            Encode.object
                [ ( "HaveTimeForReview"
                  , Encode.object
                        [ ( "reviewer", Encode.string reviewer ) ]
                  )
                ]

        WillReview id ->
            Encode.object
                [ ( "WillReview"
                  , Encode.object
                        [ ( "review_id", Encode.int id ) ]
                  )
                ]

        WontReview id ->
            Encode.object
                [ ( "WontReview"
                  , Encode.object
                        [ ( "review_id", Encode.int id ) ]
                  )
                ]
