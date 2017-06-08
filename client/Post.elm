module Post exposing (sendRequest)

import Http
import Request
import Response


sendRequest handleResult request =
    Http.send handleResult (post request)


post request =
    Http.post
        "http://localhost:3000/find-reviewer/"
        (Http.jsonBody (Request.encodeRequest request))
        Response.responseDecoder
