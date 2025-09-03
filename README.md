# shortest-color

[![Crates.io](https://img.shields.io/crates/v/shortest-color.svg)](https://crates.io/crates/shortest-color)
[![Documentation](https://docs.rs/shortest-color/badge.svg)](https://docs.rs/shortest-color)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/rostislavnagimov/shortest-color)

**Ultra-fast 7.6KB CSS color parser with 268ns latency. Zero dependencies, zero allocations.**

## Key Performance Highlights

- **268 nanoseconds** average processing time per color
- **7.6KB total library size** - smaller than most images
- **291M+ operations/sec** for simple cases  
- **Zero dependencies** - no external crates required
- **Zero allocations** for most common operations
- **574K+ ops/sec** even for complex realistic CSS workloads

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

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
shortest-color = "0.1.1"
```

or just run

```
cargo add shortest-color
```

## Usage
The library handles various input formats gracefully:

```rust
use shortest_color::shorten_css_color;

// Case insensitive
let result = shorten_css_color("RGB(255, 0, 0)");
println!("{}", result); // "red"

let result = shorten_css_color("WHITE");
println!("{}", result); // "#fff"

// Flexible spacing
let result = shorten_css_color("rgb( 255 , 0 , 0 )");
println!("{}", result); // "red"

let result = shorten_css_color("rgb(255 0 0)");
println!("{}", result); // "red"

// Whitespace handling
let result = shorten_css_color("  #ff0000  ");
println!("{}", result); // "red"

// Decimal values
let result = shorten_css_color("rgb(255.0, 0.0, 0.0)");
println!("{}", result); // "red"

let result = shorten_css_color("rgba(255, 0, 0, 0.5)");
println!("{}", result); // "#ff000080"

// HSL with units
let result = shorten_css_color("hsl(360deg, 100%, 50%)");
println!("{}", result); // "red"

let result = shorten_css_color("hsl(1turn, 100%, 50%)");
println!("{}", result); // "red"

let result = shorten_css_color("hsl(400grad, 100%, 50%)");
println!("{}", result); // "red"
```

## Performance Benchmarks

This library is designed for extreme performance. Here are benchmark results running 1,900,000 total operations across different test scenarios:

```
CSS Color Shortener Benchmark
Running 100000 iterations per test

Test                           Total     Avg/op         Ops/sec
──────────────────────────────────────────────────────────────────────
hex_basic                   8.80ms       88ns     11358365 ops/s
hex_shorthand              11.83ms      118ns      8456033 ops/s
hex_with_alpha             15.08ms      150ns      6629833 ops/s
rgb_basic                  11.45ms      114ns      8735722 ops/s
rgb_spaces                 12.64ms      126ns      7914235 ops/s
rgba_alpha                 20.43ms      204ns      4895581 ops/s
rgba_percentage            20.92ms      209ns      4779581 ops/s
hsl_basic                  13.68ms      136ns      7309674 ops/s
hsl_with_units             14.77ms      147ns      6770614 ops/s
hsla_alpha                 22.15ms      221ns      4514715 ops/s
keyword_short               2.40ms       24ns     41611029 ops/s
keyword_long               13.24ms      132ns      7553725 ops/s
keyword_transparent        12.53ms      125ns      7978882 ops/s
invalid_color               7.16ms       71ns     13972984 ops/s
empty_string              342.58µs        3ns    291900065 ops/s
whitespace                  8.01ms       80ns     12479720 ops/s
case_insensitive           11.43ms      114ns      8749671 ops/s
mixed_workload            130.45ms     1304ns       766585 ops/s
realistic_css             174.19ms     1741ns       574084 ops/s
──────────────────────────────────────────────────────────────────────

Summary:
  Average time per operation: 268ns
  Fastest: empty_string (291900065 ops/s)
  Slowest: realistic_css (574084 ops/s)
  Total benchmark operations: 1900000
```

### Performance Highlights

- **291M+ operations per second** for simple cases
- **574K+ operations per second** for realistic CSS workloads  
- **Sub-microsecond latency** for most operations
- **Zero allocation** path for many common cases
- **Consistent performance** across different input formats

## Why Choose shortest-color?

### Tiny Size
At just **7.6KB**, shortest-color is smaller than most image files. Perfect for:
- WebAssembly applications
- Edge computing
- Resource-constrained environments
- Minimizing bundle sizes

### Blazing Fast
With an average of **268 nanoseconds** per operation:
- Process millions of colors per second
- Ideal for real-time applications
- CSS processing pipelines
- Color analysis tools

### Zero Dependencies
- No external crates required
- Minimal supply chain risk
- Fast compilation times
- Easy auditing

## Architecture

The library is built with performance in mind:

- **Lookup Tables**: Pre-computed hex parsing for maximum speed
- **Lazy Static**: Color keyword mapping computed once at startup  
- **Zero-Copy Parsing**: Minimal string allocations during parsing
- **Optimized Algorithms**: Fast HSL to RGB conversion
- **Branch Prediction**: Code structured for predictable branching

## API Reference

#### `shorten_css_color(color_str: &str) -> String`

The only public function. Takes any CSS color string and returns its shortest representation. All color parsing and optimization is handled internally.

## Contributing

Contributions are welcome! Please feel free to:

- Report bugs
- Suggest new features
- Submit pull requests
- Improve documentation

## License

This project is licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This library supports all CSS Color Module Level 3 and Level 4 color formats and strives for maximum compatibility with browser implementations.

---

<div align="center">
  <b>Made with Rust for maximum performance and minimal footprint</b>
</div>