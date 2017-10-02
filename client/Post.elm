module Post exposing (findReviewer)

import Http
import Request
import Response


findReviewer : (Result Http.Error Response.Response -> msg) -> Request.Request -> Cmd msg
findReviewer handleResult request =
    Http.send handleResult <|
        Http.post "find-reviewer" (Http.jsonBody <| Request.encode request) Response.decode
