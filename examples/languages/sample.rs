// Rust sample demonstrating syntax highlighting
use std::collections::HashMap;

/// Calculate the factorial of a number
fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

#[derive(Debug, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn main() {
    let mut colors = HashMap::new();
    colors.insert("red", Color::new(255, 0, 0));
    colors.insert("green", Color::new(0, 255, 0));
    colors.insert("blue", Color::new(0, 0, 255));

    for (name, color) in &colors {
        println!("{}: {}", name, color.to_hex());
    }

    let result = factorial(5);
    println!("5! = {}", result);
}
