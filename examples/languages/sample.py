#!/usr/bin/env python3
"""Python sample demonstrating syntax highlighting"""

from typing import List, Tuple
import math


class Color:
    """Represents an RGB color"""

    def __init__(self, r: int, g: int, b: int):
        self.r = max(0, min(255, r))
        self.g = max(0, min(255, g))
        self.b = max(0, min(255, b))

    def to_hex(self) -> str:
        """Convert color to hex string"""
        return f"#{self.r:02x}{self.g:02x}{self.b:02x}"

    def lighten(self, amount: float = 0.1) -> 'Color':
        """Return a lightened version of this color"""
        adjust = lambda c: min(255, int(c + (255 - c) * amount))
        return Color(adjust(self.r), adjust(self.g), adjust(self.b))

    def __repr__(self) -> str:
        return f"Color({self.r}, {self.g}, {self.b})"


def generate_palette(base: Color, count: int) -> List[Color]:
    """Generate a palette by varying the hue"""
    colors = [base]
    for i in range(1, count):
        # Simple hue rotation
        angle = (360 / count) * i
        rad = math.radians(angle)
        # This is a simplified color rotation
        colors.append(Color(
            int(abs(math.cos(rad)) * 255),
            int(abs(math.sin(rad)) * 255),
            base.b
        ))
    return colors


def main():
    # Create base color
    base = Color(255, 128, 0)
    print(f"Base color: {base.to_hex()}")

    # Generate palette
    palette = generate_palette(base, 5)

    # Display results
    for i, color in enumerate(palette):
        lightened = color.lighten(0.2)
        print(f"{i}: {color.to_hex()} -> {lightened.to_hex()}")


if __name__ == "__main__":
    main()
