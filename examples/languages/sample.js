// JavaScript sample demonstrating syntax highlighting

class Color {
  constructor(r, g, b) {
    this.r = r;
    this.g = g;
    this.b = b;
  }

  toHex() {
    const toHexComponent = (c) => {
      const hex = c.toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    };

    return `#${toHexComponent(this.r)}${toHexComponent(this.g)}${toHexComponent(this.b)}`;
  }

  lighten(amount = 0.1) {
    const adjust = (c) => Math.min(255, Math.floor(c + (255 - c) * amount));
    return new Color(adjust(this.r), adjust(this.g), adjust(this.b));
  }
}

// Array methods and arrow functions
const colors = [
  new Color(255, 0, 0),
  new Color(0, 255, 0),
  new Color(0, 0, 255),
];

const hexColors = colors.map(color => color.toHex());
console.log('Original colors:', hexColors);

const lightenedColors = colors.map(color => color.lighten(0.2).toHex());
console.log('Lightened colors:', lightenedColors);

// Template literals and object destructuring
const palette = { primary: '#ff5500', secondary: '#00aaff' };
const { primary, secondary } = palette;
console.log(`Primary: ${primary}, Secondary: ${secondary}`);
