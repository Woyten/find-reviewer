module Tests exposing (..)

import Test exposing (..)
import Expect
import Response
import Json.Decode


requestsWithoutParams : List Response.Response
requestsWithoutParams =
    [ Response.UnknownIdentity
    , Response.Accepted
    , Response.AlreadyRegistered
    , Response.NoReviewerNeeded
    , Response.ReviewNotFound
    ]


suite : Test
suite =
    describe "decoder tests without params"
        (List.map
            (\x ->
                test ("Parses " ++ toString x)
                    (\() ->
                        Expect.equal (Ok x)
                            (Json.Decode.decodeString Response.decode
                                ("{\"" ++ toString x ++ "\": {}}")
                            )
                    )
            )
            requestsWithoutParams
        )


suite2 : Test
suite2 =
    describe "decoder test with params"
        [ test
            ("Parses KnownIdentity")
            (\() ->
                Expect.equal (Ok (Response.KnownIdentity "Kebes"))
                    (Json.Decode.decodeString
                        Response.decode
                        ("{\"KnownIdentity\": {\"username\":\"Kebes\" }}")
                    )
            )
        , test
            ("Parses NeedsReviewer")
            (\() ->
                Expect.equal (Ok (Response.NeedsReviewer "Kebes" 12345))
                    (Json.Decode.decodeString
                        Response.decode
                        ("{\"NeedsReviewer\": {\"coder\":\"Kebes\", \"review_id\":12345 }}")
                    )
            )
        ]
