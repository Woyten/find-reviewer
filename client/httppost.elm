module Main exposing (..)

import Html exposing (..)
import Html.Events exposing (..)
import Json.Decode exposing (..)
import Json.Encode exposing (..)
import Http
import Platform.Cmd
import Platform.Sub


main =
    Html.program { init = init, subscriptions = subscriptions, update = update, view = view }


type Model
    = Model String


type Msg
    = ButtonClicked
    | RequestDone String


init =
    ( Model "", Cmd.none )


subscriptions =
    always Sub.none


update msg (Model t) =
    case msg of
        ButtonClicked ->
            ( Model t, sendRequest )

        RequestDone newText ->
            ( Model newText, Cmd.none )


sendRequest =
    Http.send handleError
        (Http.post "http://localhost"
            (Http.jsonBody
                (Json.Encode.object
                    [ ( "name", Json.Encode.string "bla" )
                    , ( "id", Json.Encode.int 42 )
                    ]
                )
            )
            (Json.Decode.string)
        )


handleError error =
    RequestDone (toString error)


view (Model t) =
    div [] [ button [ onClick ButtonClicked ] [ text "Klick mich" ], text t ]
