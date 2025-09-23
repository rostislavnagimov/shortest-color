use shortest_color::shorten_css_color;
use std::time::{Duration, Instant};

const ITERATIONS: usize = 100_000;

struct BenchResult {
    name: String,
    total_time: Duration,
    avg_time_ns: u128,
    ops_per_sec: u64,
}

impl BenchResult {
    fn new(name: &str, total_time: Duration, iterations: usize) -> Self {
        let avg_time_ns = total_time.as_nanos() / iterations as u128;
        let ops_per_sec = if total_time.as_nanos() > 0 {
            (iterations as u128 * 1_000_000_000 / total_time.as_nanos()) as u64
        } else {
            0
        };

        Self {
            name: name.to_string(),
            total_time,
            avg_time_ns,
            ops_per_sec,
        }
    }

    fn print(&self) {
        println!(
            "{:<25} {:>8.2?} {:>8}ns {:>12} ops/s",
            self.name, self.total_time, self.avg_time_ns, self.ops_per_sec
        );
    }
}

fn benchmark_function<F>(name: &str, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..1000 {
        f();
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        f();
    }
    let elapsed = start.elapsed();

    BenchResult::new(name, elapsed, ITERATIONS)
}

fn main() {
    println!("🚀 CSS Color Shortener Benchmark");
    println!("Running {} iterations per test\n", ITERATIONS);
    println!(
        "{:<25} {:>10} {:>10} {:>15}",
        "Test", "Total", "Avg/op", "Ops/sec"
    );
    println!("{}", "─".repeat(70));

    let mut results = Vec::new();

    // Hex colors
    results.push(benchmark_function("hex_basic", || {
        shorten_css_color("#ff0000");
    }));

    results.push(benchmark_function("#f00", || {
        shorten_css_color("#f00");
    }));

    results.push(benchmark_function("hex_shorthand", || {
        shorten_css_color("#aabbcc");
    }));

    results.push(benchmark_function("hex_with_alpha", || {
        shorten_css_color("#ff000080");
    }));

    // RGB colors
    results.push(benchmark_function("rgb_basic", || {
        shorten_css_color("rgb(255, 0, 0)");
    }));

    results.push(benchmark_function("rgb_spaces", || {
        shorten_css_color("rgb( 255 , 0 , 0 )");
    }));

    results.push(benchmark_function("rgba_alpha", || {
        shorten_css_color("rgba(255, 0, 0, 0.5)");
    }));

    results.push(benchmark_function("rgba_percentage", || {
        shorten_css_color("rgba(255, 0, 0, 50%)");
    }));

    // HSL colors
    results.push(benchmark_function("hsl_basic", || {
        shorten_css_color("hsl(0, 100%, 50%)");
    }));

    results.push(benchmark_function("hsl_with_units", || {
        shorten_css_color("hsl(360deg, 100%, 50%)");
    }));

    results.push(benchmark_function("hsla_alpha", || {
        shorten_css_color("hsla(0, 100%, 50%, 0.5)");
    }));

    // Color keywords
    results.push(benchmark_function("keyword_short", || {
        shorten_css_color("red");
    }));

    results.push(benchmark_function("keyword_long", || {
        shorten_css_color("rebeccapurple");
    }));

    results.push(benchmark_function("keyword_transparent", || {
        shorten_css_color("transparent");
    }));

    // Edge cases
    results.push(benchmark_function("invalid_color", || {
        shorten_css_color("invalid");
    }));

    results.push(benchmark_function("empty_string", || {
        shorten_css_color("");
    }));

    results.push(benchmark_function("whitespace", || {
        shorten_css_color("  #ff0000  ");
    }));

    results.push(benchmark_function("case_insensitive", || {
        shorten_css_color("RGB(255, 0, 0)");
    }));

    // Mixed workload
    let test_colors = [
        "#ff0000",
        "rgb(255, 0, 0)",
        "hsl(0, 100%, 50%)",
        "rgba(255, 0, 0, 0.5)",
        "red",
        "white",
        "#aabbcc",
        "hsla(240, 100%, 50%, 1)",
        "transparent",
        "invalid",
        "#f00",
    ];

    results.push(benchmark_function("mixed_workload", || {
        for color in &test_colors {
            shorten_css_color(color);
        }
    }));

    // Realistic CSS colors
    let common_colors = [
        "#ffffff",
        "#000000",
        "#ff0000",
        "#00ff00",
        "#0000ff",
        "rgba(0, 0, 0, 0.5)",
        "rgba(255, 255, 255, 0.8)",
        "hsl(200, 50%, 50%)",
        "white",
        "black",
        "red",
        "blue",
        "#333",
        "#666",
        "#999",
        "#ccc",
        "#f5f5f5",
        "#f00",
    ];

    results.push(benchmark_function("realistic_css", || {
        for color in &common_colors {
            shorten_css_color(color);
        }
    }));

    // Print results
    for result in &results {
        result.print();
    }

    // Summary
    let avg_time: u128 =
        results.iter().map(|r| r.avg_time_ns).sum::<u128>() / results.len() as u128;
    let fastest = results.iter().max_by_key(|r| r.ops_per_sec).unwrap();
    let slowest = results.iter().min_by_key(|r| r.ops_per_sec).unwrap();

    println!("{}", "─".repeat(70));
    println!("\n📊 Summary:");
    println!("  Average time per operation: {}ns", avg_time);
    println!(
        "  Fastest: {} ({} ops/s)",
        fastest.name, fastest.ops_per_sec
    );
    println!(
        "  Slowest: {} ({} ops/s)",
        slowest.name, slowest.ops_per_sec
    );
    println!(
        "  Total benchmark operations: {}",
        ITERATIONS * results.len()
    );
}
