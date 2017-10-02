module Request exposing (Request(..), encode)

import Json.Encode as Encode


type Request
    = LoadIdentity
    | SendIdentity String
    | NeedReviewer
    | HaveTimeForReview
    | WontReview Int
    | WillReview Int


encode : Request -> Encode.Value
encode msg =
    case msg of
        LoadIdentity ->
            Encode.object
                [ ( "LoadIdentity", Encode.object [] ) ]

        SendIdentity token ->
            Encode.object
                [ ( "SendIdentity"
                  , Encode.object [ ( "token", Encode.string token ) ]
                  )
                ]

        NeedReviewer ->
            Encode.object
                [ ( "NeedReviewer", Encode.object [] ) ]

        HaveTimeForReview ->
            Encode.object
                [ ( "HaveTimeForReview", Encode.object [] ) ]

        WillReview id ->
            Encode.object
                [ ( "WillReview"
                  , Encode.object [ ( "review_id", Encode.int id ) ]
                  )
                ]

        WontReview id ->
            Encode.object
                [ ( "WontReview"
                  , Encode.object [ ( "review_id", Encode.int id ) ]
                  )
                ]
