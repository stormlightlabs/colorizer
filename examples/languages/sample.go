// Go sample demonstrating syntax highlighting
package main

import (
	"fmt"
	"math"
)

// Color represents an RGB color
type Color struct {
	R, G, B uint8
}

// NewColor creates a new Color with clamped values
func NewColor(r, g, b int) Color {
	clamp := func(v int) uint8 {
		if v < 0 {
			return 0
		}
		if v > 255 {
			return 255
		}
		return uint8(v)
	}
	return Color{clamp(r), clamp(g), clamp(b)}
}

// ToHex converts the color to a hex string
func (c Color) ToHex() string {
	return fmt.Sprintf("#%02x%02x%02x", c.R, c.G, c.B)
}

// Lighten returns a lightened version of the color
func (c Color) Lighten(amount float64) Color {
	adjust := func(v uint8) uint8 {
		result := float64(v) + (255.0-float64(v))*amount
		return uint8(math.Min(255, result))
	}
	return Color{adjust(c.R), adjust(c.G), adjust(c.B)}
}

// GeneratePalette creates a palette of colors
func GeneratePalette(base Color, count int) []Color {
	colors := make([]Color, count)
	colors[0] = base

	for i := 1; i < count; i++ {
		angle := (360.0 / float64(count)) * float64(i)
		rad := angle * math.Pi / 180.0
		colors[i] = NewColor(
			int(math.Abs(math.Cos(rad))*255),
			int(math.Abs(math.Sin(rad))*255),
			int(base.B),
		)
	}

	return colors
}

func main() {
	base := NewColor(255, 128, 0)
	fmt.Printf("Base color: %s\n", base.ToHex())

	palette := GeneratePalette(base, 5)

	for i, color := range palette {
		lightened := color.Lighten(0.2)
		fmt.Printf("%d: %s -> %s\n", i, color.ToHex(), lightened.ToHex())
	}
}
