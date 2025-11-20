// TypeScript sample demonstrating syntax highlighting
interface Color {
  r: number;
  g: number;
  b: number;
}

class Palette {
  private colors: Map<string, Color>;

  constructor() {
    this.colors = new Map();
  }

  add(name: string, color: Color): void {
    this.colors.set(name, color);
  }

  toHex(color: Color): string {
    const r = color.r.toString(16).padStart(2, '0');
    const g = color.g.toString(16).padStart(2, '0');
    const b = color.b.toString(16).padStart(2, '0');
    return `#${r}${g}${b}`;
  }

  display(): void {
    this.colors.forEach((color, name) => {
      console.log(`${name}: ${this.toHex(color)}`);
    });
  }
}

// Main execution
const palette = new Palette();
palette.add("red", { r: 255, g: 0, b: 0 });
palette.add("green", { r: 0, g: 255, b: 0 });
palette.add("blue", { r: 0, g: 0, b: 255 });
palette.display();
