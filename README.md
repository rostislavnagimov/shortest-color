# shortest-color
[![Crates.io](https://img.shields.io/crates/v/shortest-color.svg)](https://crates.io/crates/shortest-color)

Ultra-fast CSS color parser - sub-200ns latency, under 5.5KB size

Performance: <200ns average, <5.5KB size, zero dependencies

Accepts hex, rgb/rgba, hsl/hsla, keywords
```rust
use shortest_color::shorten_css_color;

shorten_css_color("#ff0000")     // "red"
shorten_css_color("rgb(0,0,255)") // "blue" 
shorten_css_color("WHITE")       // "#fff"
```
Install:
```bash
cargo add shortest-color
```
```toml
[dependencies]
shortest-color = "0.1.2"
```