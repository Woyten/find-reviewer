module Main exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Response
import Post
import Request


main =
    Html.program { init = init, update = update, subscriptions = (always Sub.none), view = view }


type UserInput
    = Confirm
    | Deny
    | HaveTimeForReview
    | NeedReviewer
    | ReviewerInputUpdate String


type Msg
    = UserInput UserInput
    | HttpResponse Response.Status


type alias Model =
    { status : Response.Status, user : String, reviewId : Int }



-- INIT


init =
    ( { status = Response.Empty, user = "", id = 0 }, Cmd.none )



-- UPDATE


update msg oldModel =
    case msg of
        UserInput userinput ->
            handleUserInput userinput oldModel

        HttpResponse status ->
            { oldModel
                | status = status
                , id =
                    case status of
                        Response.NeedsReviewer coder id ->
                            id

                        _ ->
                            oldModel.id
            }
                ! []


handleUserInput userinput oldModel =
    case userinput of
        Confirm ->
            oldModel ! [ Post.sendRequest handleResult (Request.WillReview oldModel.id) ]

        Deny ->
            oldModel ! [ Post.sendRequest handleResult (Request.WontReview oldModel.id) ]

        HaveTimeForReview ->
            oldModel ! [ Post.sendRequest handleResult (Request.HaveTimeForReview oldModel.user) ]

        NeedReviewer ->
            oldModel ! [ Post.sendRequest handleResult (Request.NeedReviewer oldModel.user) ]

        ReviewerInputUpdate userinput ->
            { oldModel | user = userinput } ! []



-- VIEW


view model =
    div [] (createDynamicControls model.status)


createDynamicControls status =
    case status of
        Response.Empty ->
            defaultControls

        Response.Accepted ->
            defaultControls ++ label "green" "Request accepted"

        Response.AlreadyRegistered ->
            defaultControls ++ label "blue" "Coder already registered"

        Response.NoHelpNeeded ->
            defaultControls ++ label "blue" "No review open"

        Response.NeedsReviewer coder review_id ->
            askForConfirmation coder review_id

        Response.ReviewNotFound ->
            defaultControls ++ label "red" "The review could not be accepted.\nYou probably ran into a timeout."

        Response.Error input err ->
            defaultControls ++ label "red" "Invalid server response:" ++ label "black" input ++ label "red" err


defaultControls =
    wrapDivs
        [ text "Name:"
        , input [ onInput (UserInput << ReviewerInputUpdate) ] []
        , button [ onClick (UserInput NeedReviewer) ] [ text "I need a review" ]
        , button [ onClick (UserInput HaveTimeForReview) ] [ text "I have time for a review" ]
        ]


askForConfirmation coder review_id =
    wrapDivs
        [ b [] [ text coder ]
        , text " needs a review first. Will you do the review?"
        , button [ onClick (UserInput Confirm) ] [ text "Yes" ]
        , button [ onClick (UserInput Deny) ] [ text "No" ]
        ]


wrapDivs =
    List.map <| \x -> div [] [ x ]


label col multilineString =
    multilineString
        |> String.lines
        |> List.map text
        |> List.map (\x -> div [ style [ ( "color", col ) ] ] [ x ])


handleResult result =
    case result of
        Ok status ->
            HttpResponse status

        Err err ->
            HttpResponse (Response.Error "Request failed" (toString err))
