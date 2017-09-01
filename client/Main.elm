module Main exposing (main)

import Html exposing (..)
import Html.Attributes as Attributes
import Html.Events as Events
import Http
import Post
import Request
import Response
import Style


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
    { status = Initial, user = "" } ! []



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
    div [ Style.mainDiv ] [ div [ Style.loginDiv ] (createDynamicControls model) ]


createDynamicControls model =
    case model.status of
        Initial ->
            defaultControls model ++ [ multiline "" " " ]

        HttpSuccess Response.Accepted ->
            defaultControls model
                ++ [ multiline "green" "Request accepted" ]

        HttpSuccess Response.AlreadyRegistered ->
            defaultControls model
                ++ [ multiline "blue" "Coder already registered" ]

        HttpSuccess Response.NoReviewerNeeded ->
            defaultControls model
                ++ [ multiline "blue" "No review open" ]

        HttpSuccess (Response.NeedsReviewer coder review_id) ->
            askForConfirmation coder review_id

        HttpSuccess Response.ReviewNotFound ->
            defaultControls model
                ++ [ multiline "red" "Review not found. You probably ran into a timeout." ]

        HttpFailure (Http.BadPayload errMsg _) ->
            defaultControls model
                ++ [ multiline "red" "Invalid server payload:"
                   , multiline "black" errMsg
                   ]

        HttpFailure otherError ->
            defaultControls model


defaultControls model =
    [ label [ Style.text "black", Attributes.for "text_input" ] [ text "Name:" ]
    , input [ Style.textField, Attributes.id "text_input", Attributes.value model.user, Events.onInput (UserInput << ReviewerInputUpdate) ] []
    , div [ Style.buttonBox ]
        [ button [ Style.button, Events.onClick (UserInput NeedReviewer) ] [ text "I need a reviewer" ]
        , button [ Style.button, Events.onClick (UserInput HaveTimeForReview) ] [ text "I have time for a review" ]
        ]
    ]


askForConfirmation coder review_id =
    [ div [ Style.text "black" ] [ text "The following person needs a review:" ]
    , div [ Style.coder ] [ text coder ]
    , div [ Style.buttonBox ]
        [ button [ Style.button, Events.onClick (UserInput <| WillReview review_id) ] [ text "I will do it" ]
        , button [ Style.button, Events.onClick (UserInput <| WontReview review_id) ] [ text "I won't do it" ]
        ]
    ]


multiline color string =
    string
        |> String.lines
        |> List.map (text >> List.singleton >> div [])
        |> div [ Style.text color ]
