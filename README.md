# shortest-color
[![Crates.io](https://bit.ly/4guN0OR)](https://bit.ly/42BNBbx)

Ultra-fast CSS color parser - <120ns, <6KB, zero dependencies. For hex, rgb/rgba, hsl/hsla, keywords

```rust
use shortest_color::shorten_css_color;

shorten_css_color("#ff0000") // red
shorten_css_color("rgb(0,0,255)") // blue
shorten_css_color("WHITE") // #fff
```
Install:
```bash
cargo add shortest-color
```
```toml
[dependencies]
shortest-color = "0.1.4"
```