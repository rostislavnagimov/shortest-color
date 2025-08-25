# shortest-color

[![Crates.io](https://img.shields.io/crates/v/shortest-color.svg)](https://crates.io/crates/shortest-color)
[![Documentation](https://docs.rs/shortest-color/badge.svg)](https://docs.rs/shortest-color)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/yourusername/shortest-color)

A blazingly fast Rust library for parsing and shortening CSS colors to their most compact representation.

## ✨ Features

- 🚀 **High Performance**: Processes millions of colors per second
- 🎯 **Zero Allocations**: Most operations don't allocate memory
- 📦 **Comprehensive Parsing**: Supports all standard CSS color formats
- ✅ **Accurate Conversion**: Maintains color fidelity while minimizing output
- 🔧 **Simple API**: Easy to integrate into any project

### Supported Color Formats

| Input Format | Example | Output |
|--------------|---------|--------|
| Hex (3-digit) | `#f00` | `red` |
| Hex (6-digit) | `#ff0000` | `red` |
| Hex (4-digit with alpha) | `#f00a` | `#f00a` |
| Hex (8-digit with alpha) | `#ff0000ff` | `red` |
| RGB function | `rgb(255, 0, 0)` | `red` |
| RGBA function | `rgba(255, 0, 0, 0.5)` | `#ff000080` |
| HSL function | `hsl(0, 100%, 50%)` | `red` |
| HSLA function | `hsla(0, 100%, 50%, 1)` | `red` |
| Color keywords | `red`, `blue`, `transparent` | `red`, `blue`, `#0000` |

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
shortest-color = "0.1.0"
```

## 📚 Usage

### Basic Usage

```rust
use shortest_color::shorten_css_color;

// Hex colors
assert_eq!(shorten_css_color("#ff0000"), "red");
assert_eq!(shorten_css_color("#ffffff"), "#fff");
assert_eq!(shorten_css_color("#aabbcc"), "#abc");

// RGB/RGBA functions
assert_eq!(shorten_css_color("rgb(255, 0, 0)"), "red");
assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.5)"), "#ff000080");

// HSL/HSLA functions
assert_eq!(shorten_css_color("hsl(0, 100%, 50%)"), "red");
assert_eq!(shorten_css_color("hsla(240, 100%, 50%, 1)"), "#00f");

// Color keywords
assert_eq!(shorten_css_color("white"), "#fff");
assert_eq!(shorten_css_color("transparent"), "#0000");
```

### Advanced Usage

```rust
use shortest_color::{parse, shorten, Color};

// Parse a color string into a Color struct
if let Some(color) = parse("#ff0000") {
    println!("Red: {}, Green: {}, Blue: {}, Alpha: {}", 
             color.r, color.g, color.b, color.a);
}

// Create a color and get its shortest representation
let color = Color::rgb(170, 187, 204);
assert_eq!(shorten(&color), "#abc");

// Handle alpha transparency
let transparent_red = Color::new(255, 0, 0, 128);
assert_eq!(shorten(&transparent_red), "#ff000080");
```

### Flexible Input Handling

The library handles various input formats gracefully:

```rust
use shortest_color::shorten_css_color;

// Case insensitive
assert_eq!(shorten_css_color("RGB(255, 0, 0)"), "red");
assert_eq!(shorten_css_color("WHITE"), "#fff");

// Flexible spacing
assert_eq!(shorten_css_color("rgb( 255 , 0 , 0 )"), "red");
assert_eq!(shorten_css_color("rgb(255 0 0)"), "red");

// Whitespace handling
assert_eq!(shorten_css_color("  #ff0000  "), "red");

// Decimal values
assert_eq!(shorten_css_color("rgb(255.0, 0.0, 0.0)"), "red");
assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.5)"), "#ff000080");

// HSL with units
assert_eq!(shorten_css_color("hsl(360deg, 100%, 50%)"), "red");
assert_eq!(shorten_css_color("hsl(1turn, 100%, 50%)"), "red");
assert_eq!(shorten_css_color("hsl(400grad, 100%, 50%)"), "red");
```

## ⚡ Performance

This library is designed for high performance. Here are benchmark results running 100,000 iterations per test:

```
🚀 CSS Color Shortener Benchmark
Running 100000 iterations per test

Test                           Total     Avg/op         Ops/sec
──────────────────────────────────────────────────────────────────────
hex_basic                  28.35ms      283ns      3527088 ops/s
hex_shorthand              21.06ms      210ns      4748103 ops/s
hex_with_alpha             15.28ms      152ns      6542879 ops/s
rgb_basic                  27.46ms      274ns      3642086 ops/s
rgb_spaces                 27.42ms      274ns      3647200 ops/s
rgba_alpha                 18.63ms      186ns      5368010 ops/s
rgba_percentage            18.14ms      181ns      5512565 ops/s
hsl_basic                  29.00ms      289ns      3448721 ops/s
hsl_with_units             29.85ms      298ns      3349723 ops/s
hsla_alpha                 20.33ms      203ns      4919111 ops/s
keyword_short               2.46ms       24ns     40686942 ops/s
keyword_long               23.24ms      232ns      4303311 ops/s
keyword_transparent        14.17ms      141ns      7059529 ops/s
invalid_color               8.03ms       80ns     12461059 ops/s
empty_string              285.50µs        2ns    350262697 ops/s
whitespace                 27.22ms      272ns      3674067 ops/s
case_insensitive           28.07ms      280ns      3563151 ops/s
mixed_workload            218.74ms     2187ns       457154 ops/s
realistic_css             271.45ms     2714ns       368390 ops/s
──────────────────────────────────────────────────────────────────────

📊 Summary:
  Average time per operation: 435ns
  Fastest: empty_string (350262697 ops/s)
  Slowest: realistic_css (368390 ops/s)
  Total benchmark operations: 1900000
```

### Performance Highlights

- 🏃‍♂️ **350M+ operations per second** for simple cases
- ⚡ **3-40M operations per second** for complex parsing
- 📈 **Sub-microsecond latency** for most operations
- 🎯 **Zero allocation** path for many common cases

Run benchmarks yourself:

```bash
cargo bench
```

## 🏗️ Architecture

The library is built with performance in mind:

- **Lookup Tables**: Pre-computed hex parsing for maximum speed
- **Lazy Static**: Color keyword mapping computed once at startup  
- **Zero-Copy Parsing**: Minimal string allocations during parsing
- **Optimized Algorithms**: Fast HSL to RGB conversion
- **Branch Prediction**: Code structured for predictable branching

## 🔧 API Reference

### Main Functions

#### `shorten_css_color(color_str: &str) -> String`

The primary function that takes any CSS color string and returns its shortest representation.

#### `parse(color_str: &str) -> Option<Color>`

Parses a CSS color string into a `Color` struct. Returns `None` for invalid input.

#### `shorten(color: &Color) -> String`

Converts a `Color` struct to its shortest string representation.

### Color Struct

```rust
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Color {
    pub r: u8,  // Red component (0-255)
    pub g: u8,  // Green component (0-255) 
    pub b: u8,  // Blue component (0-255)
    pub a: u8,  // Alpha component (0-255)
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self  // Alpha = 255
}
```

## 🧪 Testing

The library includes comprehensive tests covering:

- ✅ All CSS color formats
- ✅ Edge cases and error conditions  
- ✅ Performance regressions
- ✅ Cross-platform compatibility

Run the test suite:

```bash
cargo test
```

## 🤝 Contributing

Contributions are welcome! Please feel free to:

- 🐛 Report bugs
- 💡 Suggest new features
- 🔧 Submit pull requests
- 📖 Improve documentation

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

This library supports all CSS Color Module Level 3 and Level 4 color formats and strives for maximum compatibility with browser implementations.

---

<div align="center">
  <b>Made with ❤️ and ⚡ in Rust</b>
</div>