module Main exposing (main)

import Element exposing (..)
import Element.Attributes exposing (..)
import Element.Events as Events
import Element.Input as Input
import Html
import Http
import Post
import Request
import Response
import Styles


main : Program Never Model Msg
main =
    Html.program { init = init, update = update, subscriptions = always Sub.none, view = view }


type alias Model =
    { status : Response.Response, httpError : Maybe Http.Error, token : String, username : String }


type Msg
    = UserInteraction UserInteraction
    | HttpResult (Result Http.Error Response.Response)


type UserInteraction
    = TokenInput String
    | TokenSubmission
    | NeedReviewer
    | HaveTimeForReview
    | WillReview Int
    | WontReview Int



-- INIT


init : ( Model, Cmd Msg )
init =
    { status = Response.UnknownIdentity, httpError = Nothing, token = "", username = "" }
        ! [ Post.findReviewer HttpResult Request.LoadIdentity ]



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        UserInteraction userInteraction ->
            handleUserInteraction userInteraction model

        HttpResult httpResult ->
            handleHttpResult httpResult model


handleUserInteraction : UserInteraction -> Model -> ( Model, Cmd Msg )
handleUserInteraction userInteraction model =
    case userInteraction of
        TokenInput token ->
            { model | token = token } ! []

        TokenSubmission ->
            model ! [ Post.findReviewer HttpResult <| Request.SendIdentity model.token ]

        NeedReviewer ->
            model ! [ Post.findReviewer HttpResult <| Request.NeedReviewer ]

        HaveTimeForReview ->
            model ! [ Post.findReviewer HttpResult <| Request.HaveTimeForReview ]

        WillReview review_id ->
            model ! [ Post.findReviewer HttpResult <| Request.WillReview review_id ]

        WontReview review_id ->
            model ! [ Post.findReviewer HttpResult <| Request.WontReview review_id ]


handleHttpResult : Result Http.Error Response.Response -> Model -> ( Model, Cmd Msg )
handleHttpResult httpResult model =
    let
        getNewUsername okResult =
            case okResult of
                Response.KnownIdentity username ->
                    username

                _ ->
                    model.username
    in
        case httpResult of
            Ok okResult ->
                { model | status = okResult, httpError = Nothing, username = getNewUsername okResult } ! []

            Err err ->
                { model | httpError = Just err } ! []



-- VIEW


defaultSpacing : Attribute variation msg
defaultSpacing =
    spacing 20.0


defaultPadding : Attribute variation msg
defaultPadding =
    padding 20.0


view : Model -> Html.Html Msg
view model =
    viewport Styles.stylesheet <|
        Element.map UserInteraction <|
            el Styles.Main [ center, verticalCenter, defaultPadding ] <|
                column Styles.None
                    [ defaultSpacing ]
                    [ showDynamicControls model.status model.token model.username
                    , showHttpError model.httpError
                    ]


showDynamicControls : Response.Response -> String -> String -> Element Styles.Styles variation UserInteraction
showDynamicControls status token username =
    let
        defaultControls =
            showDefaultControls username
    in
        case status of
            Response.KnownIdentity username ->
                defaultControls Styles.SuccessText "Login successful"

            Response.UnknownIdentity ->
                askForTokenInput token

            Response.Accepted ->
                defaultControls Styles.SuccessText "Action successful"

            Response.AlreadyRegistered ->
                defaultControls Styles.InfoText "Coder already registered"

            Response.NoReviewerNeeded ->
                defaultControls Styles.InfoText "No review open"

            Response.NeedsReviewer coder review_id ->
                askForConfirmation coder review_id

            Response.ReviewNotFound ->
                defaultControls Styles.ErrorText "Review not found. You probably ran into a timeout."


askForTokenInput : String -> Element Styles.Styles variation UserInteraction
askForTokenInput token =
    column Styles.None [ defaultSpacing ] <|
        [ el Styles.RegularText [] <|
            Input.text Styles.TextBox [ defaultSpacing, defaultPadding ] <|
                Input.Text TokenInput token (Input.labelAbove <| text "Please enter your token") []
        , button Styles.Button [ defaultPadding, Events.onClick TokenSubmission ] <| text "Login"
        ]


showDefaultControls : String -> Styles.Styles -> String -> Element Styles.Styles variation UserInteraction
showDefaultControls username style message =
    column Styles.None [ defaultSpacing ] <|
        [ el Styles.RegularText [] <| text <| "Logged in as " ++ username
        , row Styles.None
            [ spread, defaultSpacing ]
            [ button Styles.Button [ defaultPadding, Events.onClick NeedReviewer ] <| text "I need a reviewer"
            , button Styles.Button [ defaultPadding, Events.onClick HaveTimeForReview ] <| text "I have time for a review"
            ]
        , el style [] <| text message
        ]


askForConfirmation : String -> Int -> Element Styles.Styles variation UserInteraction
askForConfirmation coder review_id =
    column Styles.None [ defaultSpacing ] <|
        [ el Styles.RegularText [] <| text "The following person needs a review:"
        , el Styles.Coder [] <| text coder
        , row Styles.None
            [ spread, defaultSpacing ]
            [ button Styles.Button [ defaultPadding, Events.onClick <| WillReview review_id ] <| text "I will do it"
            , button Styles.Button [ defaultPadding, Events.onClick <| WontReview review_id ] <| text "I won't do it"
            ]
        ]


showHttpError : Maybe Http.Error -> Element Styles.Styles variation msg
showHttpError maybeError =
    let
        getErrorText error =
            case error of
                Http.BadStatus { body } ->
                    "Bad request / " ++ body

                Http.BadPayload errorMessage _ ->
                    "Bad response / " ++ errorMessage

                otherError ->
                    toString otherError
    in
        el Styles.ErrorText [] <| text (maybeError |> Maybe.map getErrorText |> Maybe.withDefault "")
