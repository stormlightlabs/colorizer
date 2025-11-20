-- Elm sample demonstrating syntax highlighting
module Color exposing (Color, toHex, lighten, generatePalette)

import String


-- Type definition
type alias Color =
    { r : Int
    , g : Int
    , b : Int
    }


-- Helper function to clamp values
clamp : Int -> Int
clamp value =
    if value < 0 then
        0
    else if value > 255 then
        255
    else
        value


-- Create a new color with clamped values
newColor : Int -> Int -> Int -> Color
newColor r g b =
    { r = clamp r
    , g = clamp g
    , b = clamp b
    }


-- Convert a number to a two-digit hex string
toHexComponent : Int -> String
toHexComponent n =
    let
        hex =
            String.fromInt n
                |> String.padLeft 2 '0'
    in
    hex


-- Convert color to hex string
toHex : Color -> String
toHex color =
    "#"
        ++ toHexComponent color.r
        ++ toHexComponent color.g
        ++ toHexComponent color.b


-- Lighten a color by a given amount
lighten : Float -> Color -> Color
lighten amount color =
    let
        adjust c =
            let
                result =
                    toFloat c + (255 - toFloat c) * amount
            in
            clamp (round result)
    in
    { r = adjust color.r
    , g = adjust color.g
    , b = adjust color.b
    }


-- Generate a palette of colors
generatePalette : Color -> Int -> List Color
generatePalette base count =
    List.range 0 (count - 1)
        |> List.map
            (\i ->
                let
                    angle =
                        (360.0 / toFloat count) * toFloat i

                    rad =
                        angle * pi / 180.0
                in
                newColor
                    (round (abs (cos rad) * 255))
                    (round (abs (sin rad) * 255))
                    base.b
            )


-- Example usage
main : String
main =
    let
        base =
            newColor 255 128 0

        palette =
            generatePalette base 5

        hexColors =
            List.map toHex palette
    in
    String.join ", " hexColors
