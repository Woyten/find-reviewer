module Main exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as Decode
import Dict


main =
    Html.program { init = init, update = update, subscriptions = (always Sub.none), view = view }


type Msg
    = Input String


type Status
    = Empty
    | Accepted
    | AlreadyRegistered
    | NoReviewOpen
    | NeedsReviewer String Int
    | ReviewNotFound
    | Error String String


requestsWithoutParams =
    [ Accepted, AlreadyRegistered, NoReviewOpen, ReviewNotFound ]


dict =
    requestsWithoutParams
        |> List.map (\x -> ( toString x, x ))
        |> Dict.fromList



-- INIT


init =
    ( Empty, Cmd.none )



-- UPDATE


update (Input input) =
    let
        newModel =
            if String.isEmpty input then
                Empty
            else
                evaluateStatus input
    in
        always ( newModel, Cmd.none )


evaluateStatus input =
    case (parseResponse input) of
        Ok status ->
            status

        Err err ->
            Error input err


parseResponse =
    let
        decoder =
            Decode.oneOf [ stringDecoder, objectDecoder ]
    in
        Decode.decodeString decoder


stringDecoder =
    Decode.string
        |> Decode.map resolveStatus
        |> Decode.andThen unwrap


resolveStatus status =
    dict |> Dict.get status


unwrap value =
    value
        |> Maybe.map Decode.succeed
        |> Maybe.withDefault (Decode.fail <| "Expecting one of " ++ (toString requestsWithoutParams))


objectDecoder =
    Decode.field "NeedsReviewer" needsReviewerDecoder


needsReviewerDecoder =
    Decode.map2 NeedsReviewer
        (Decode.field "coder" Decode.string)
        (Decode.field "review_id" Decode.int)



-- VIEW


view status =
    let
        temporaryControls =
            [ input [ onInput Input ] [], hr [] [] ]
    in
        div [] (temporaryControls ++ createDynamicControls status)


createDynamicControls status =
    case status of
        Empty ->
            defaultControls

        Accepted ->
            defaultControls ++ label "green" "Request accepted"

        AlreadyRegistered ->
            defaultControls ++ label "blue" "Coder already registered"

        NoReviewOpen ->
            defaultControls ++ label "blue" "No review open"

        NeedsReviewer coder review_id ->
            askForConfirmation coder review_id

        ReviewNotFound ->
            defaultControls ++ label "red" "The review could not be accepted.\nYou probably ran into a timeout."

        Error input err ->
            defaultControls ++ label "red" "Invalid server response:" ++ label "black" input ++ label "red" err


defaultControls =
    wrapDivs
        [ text "Name:"
        , input [] []
        , button [] [ text "I need a review" ]
        , button [] [ text "I have time for a review" ]
        ]


askForConfirmation coder review_id =
    wrapDivs
        [ b [] [ text coder ]
        , text " needs a review first. Will you do the review?"
        , button [] [ text "Yes" ]
        , button [] [ text "No" ]
        ]


wrapDivs =
    List.map <| \x -> div [] [ x ]


label col multilineString =
    multilineString
        |> String.lines
        |> List.map text
        |> List.map (\x -> div [ style [ ( "color", col ) ] ] [ x ])
