module Styles exposing (..)

import Color
import Style
import Style.Border as Border
import Style.Color as Color
import Style.Font as Font


type Styles
    = None
    | Main
    | RegularText
    | SuccessText
    | InfoText
    | ErrorText
    | Coder
    | TextBox
    | Button


stylesheet : Style.StyleSheet Styles variation
stylesheet =
    let
        roundedBorder =
            Border.rounded 15.0
    in
        Style.styleSheet
            [ Style.style Main
                [ Color.background Color.grey
                , Font.typeface [ Font.fantasy ]
                , Font.size 20.0
                , roundedBorder
                ]
            , Style.style RegularText []
            , Style.style SuccessText
                [ Color.text Color.darkGreen
                ]
            , Style.style InfoText
                [ Color.text Color.blue
                ]
            , Style.style ErrorText
                [ Color.text Color.red
                ]
            , Style.style Coder
                [ Font.typeface [ Font.sansSerif ]
                ]
            , Style.style TextBox
                [ Color.background Color.white
                , Font.typeface [ Font.serif ]
                , roundedBorder
                ]
            , Style.style Button
                [ Color.text Color.white
                , Color.background Color.darkYellow
                , Font.typeface [ Font.sansSerif ]
                , Font.bold
                , roundedBorder
                , Border.none
                ]
            ]
