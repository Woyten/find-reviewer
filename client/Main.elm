module Main exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Response
import Post
import Request


main =
    Html.program { init = init, update = update, subscriptions = (always Sub.none), view = view }


type UserInput
    = ReviewerInputUpdate String
    | NeedReviewer
    | HaveTimeForReview
    | WillReview Int
    | WontReview Int


type Msg
    = UserInput UserInput
    | HttpResult (Result Http.Error Response.Response)


type ViewStatus
    = Initial
    | HttpSuccess Response.Response
    | HttpFailure Http.Error


type alias Model =
    { status : ViewStatus, user : String }



-- INIT


init =
    ( { status = Initial, user = "" }, Cmd.none )



-- UPDATE


update msg model =
    case msg of
        UserInput userInput ->
            handleUserInput userInput model

        HttpResult (Ok status) ->
            { model | status = HttpSuccess status } ! []

        HttpResult (Err err) ->
            { model | status = HttpFailure err } ! []


handleUserInput userInput model =
    case userInput of
        ReviewerInputUpdate userInput ->
            { model | user = userInput } ! []

        NeedReviewer ->
            model ! [ Post.sendRequest HttpResult (Request.NeedReviewer model.user) ]

        HaveTimeForReview ->
            model ! [ Post.sendRequest HttpResult (Request.HaveTimeForReview model.user) ]

        WillReview review_id ->
            model ! [ Post.sendRequest HttpResult (Request.WillReview review_id) ]

        WontReview review_id ->
            model ! [ Post.sendRequest HttpResult (Request.WontReview review_id) ]



-- VIEW


view model =
    div [] (createDynamicControls model)


createDynamicControls model =
    case model.status of
        Initial ->
            defaultControls model

        HttpSuccess Response.Accepted ->
            defaultControls model ++ label "green" "Request accepted"

        HttpSuccess Response.AlreadyRegistered ->
            defaultControls model ++ label "blue" "Coder already registered"

        HttpSuccess Response.NoReviewerNeeded ->
            defaultControls model ++ label "blue" "No review open"

        HttpSuccess (Response.NeedsReviewer coder review_id) ->
            askForConfirmation coder review_id

        HttpSuccess Response.ReviewNotFound ->
            defaultControls model ++ label "red" "The review could not be accepted.\nYou probably ran into a timeout."

        HttpFailure (Http.BadPayload errMsg _) ->
            defaultControls model
                ++ label "red" "Invalid server payload:"
                ++ label "black" errMsg

        HttpFailure otherError ->
            defaultControls model
                ++ label "red" "HTTP error:"
                ++ label "black" (toString otherError)


defaultControls model =
    wrapDivs
        [ text "Name:"
        , input [ value model.user, onInput (UserInput << ReviewerInputUpdate) ] []
        , button [ onClick (UserInput NeedReviewer) ] [ text "I need a review" ]
        , button [ onClick (UserInput HaveTimeForReview) ] [ text "I have time for a review" ]
        ]


askForConfirmation coder review_id =
    wrapDivs
        [ b [] [ text coder ]
        , text " needs a review first. Will you do the review?"
        , button [ onClick (UserInput <| WillReview review_id) ] [ text "Yes" ]
        , button [ onClick (UserInput <| WontReview review_id) ] [ text "No" ]
        ]


wrapDivs =
    List.map <| \x -> div [] [ x ]


label color multilineString =
    multilineString
        |> String.lines
        |> List.map text
        |> List.map (\x -> div [ style [ ( "color", color ) ] ] [ x ])
