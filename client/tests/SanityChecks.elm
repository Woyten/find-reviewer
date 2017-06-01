module SanityChecks exposing (..)

import Test exposing (..)
import Expect
import Response


suite : Test
suite =
    describe "sanity checks for the server/client interface"
        [ test "toString works properly for sum types"
            (\() -> Expect.equal (toString Response.Accepted) "Accepted")
        ]
