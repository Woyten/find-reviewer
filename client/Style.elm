module Style exposing (..)

import Html.Attributes as Attributes


mainDiv =
    Attributes.style <| [ ( "display", "flex" ), ( "justify-content", "center" ), ( "align-items", "center" ), ( "height", "100vh" ) ]


loginDiv =
    Attributes.style <| [ ( "width", "400px" ), ( "background-color", "lightgrey" ), ( "border-radius", "5px" ) ] ++ padding


text color =
    Attributes.style <| [ ( "color", color ) ] ++ padding


coder =
    Attributes.style <| [ ( "font-size", "large" ) ] ++ padding


textField =
    Attributes.style <| [ ( "width", "100%" ), ( "box-sizing", "border-box" ) ] ++ padding ++ margin


buttonBox =
    Attributes.style <| [ ( "display", "flex" ), ( "justify-content", "space-between" ) ] ++ margin


button =
    Attributes.style padding


padding =
    [ ( "padding", "10px" ) ]


margin =
    [ ( "margin-top", "5px" ), ( "margin-bottom", "5px" ) ]
