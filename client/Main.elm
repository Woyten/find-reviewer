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


init : ( Model, Cmd msg )
init =
    { status = Initial, user = "" } ! []



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        UserInput userInput ->
            handleUserInput userInput model

        HttpResult (Ok status) ->
            { model | status = HttpSuccess status } ! []

        HttpResult (Err err) ->
            { model | status = HttpFailure err } ! []


handleUserInput : UserInput -> Model -> ( Model, Cmd Msg )
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


defaultSpacing : Attribute variation msg
defaultSpacing =
    spacing 20.0


defaultPadding : Attribute variation msg
defaultPadding =
    padding 20.0


view : Model -> Html.Html Msg
view model =
    Element.viewport Styles.stylesheet <|
        el Styles.Main [ center, verticalCenter, defaultPadding ] <|
            createDynamicControls model


createDynamicControls : Model -> Element Styles.Styles variation Msg
createDynamicControls model =
    case model.status of
        Initial ->
            defaultControls model Styles.RegularText " "

        HttpSuccess Response.Accepted ->
            defaultControls model Styles.SuccessText "Request accepted"

        HttpSuccess Response.AlreadyRegistered ->
            defaultControls model Styles.InfoText "Coder already registered"

        HttpSuccess Response.NoReviewerNeeded ->
            defaultControls model Styles.InfoText "No review open"

        HttpSuccess (Response.NeedsReviewer coder review_id) ->
            askForConfirmation coder review_id

        HttpSuccess Response.ReviewNotFound ->
            defaultControls model Styles.ErrorText "Review not found. You probably ran into a timeout."

        HttpFailure (Http.BadPayload errorMessage _) ->
            defaultControls model Styles.ErrorText ("Invalid HTTP payload / " ++ errorMessage)

        HttpFailure otherError ->
            defaultControls model Styles.ErrorText <| toString otherError


defaultControls : Model -> Styles.Styles -> String -> Element Styles.Styles variation Msg
defaultControls model style message =
    column Styles.None [ defaultSpacing ] <|
        [ Input.text Styles.TextBox [ defaultSpacing, defaultPadding ] <|
            Input.Text (UserInput << ReviewerInputUpdate) model.user (Input.labelAbove <| text "Name:") []
        , row Styles.None
            [ spread, defaultSpacing ]
            [ button Styles.Button [ defaultPadding, Events.onClick <| UserInput NeedReviewer ] <| text "I need a reviewer"
            , button Styles.Button [ defaultPadding, Events.onClick <| UserInput HaveTimeForReview ] <| text "I have time for a review"
            ]
        , el style [] <| text message
        ]


askForConfirmation : String -> Int -> Element Styles.Styles variation Msg
askForConfirmation coder review_id =
    column Styles.None [ defaultSpacing ] <|
        [ el Styles.RegularText [] <| text "The following person needs a review:"
        , el Styles.Coder [] <| text coder
        , row Styles.None
            [ spread, defaultSpacing ]
            [ button Styles.Button [ defaultPadding, Events.onClick (UserInput <| WillReview review_id) ] (text "I will do it")
            , button Styles.Button [ defaultPadding, Events.onClick (UserInput <| WontReview review_id) ] (text "I won't do it")
            ]
        ]
