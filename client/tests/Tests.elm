module Tests exposing (..)

import Test exposing (..)
import Expect
import Response
import Json.Decode


suite : Test
suite =
    describe "decoder tests"
        (List.map
            (\x ->
                test ("Parses " ++ toString x)
                    (\() ->
                        Expect.equal (Ok x)
                            (Json.Decode.decodeString Response.responseDecoder
                                ("\"" ++ toString x ++ "\"")
                            )
                    )
            )
            Response.requestsWithoutParams
        )
