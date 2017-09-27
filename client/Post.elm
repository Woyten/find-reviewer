module Post exposing (sendRequest)

import Http
import Request
import Response


sendRequest : (Result Http.Error Response.Response -> msg) -> Request.Request -> Cmd msg
sendRequest handleResult request =
    Http.send handleResult <| post request


post : Request.Request -> Http.Request Response.Response
post request =
    Http.post "find-reviewer" (Http.jsonBody <| Request.encode request) Response.decode
