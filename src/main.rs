use shortest_color::print_benchmark_summary;
use shortest_color::shorten_css_color;

fn main() {
    let test_cases = &[
        ("#ff0000", "red"),
        ("#008000", "green"),
        ("#000080", "navy"),
        ("#800080", "purple"),
        ("#808080", "gray"),
        ("#000000", "#000"),
        ("#ffffff", "#fff"),
        ("#ff00ff", "#f0f"),
        ("#00ffff", "#0ff"),
        ("#ffff00", "#ff0"),
        ("#aabbcc", "#abc"),
        ("#112233", "#123"),
        ("#ddeeff", "#def"),
        ("#123456", "#123456"),
        ("#abcdef", "#abcdef"),
        ("#ff5733", "#ff5733"),
        ("rgb(255, 0, 0)", "red"),
        ("rgb(170, 187, 204)", "#abc"),
        ("rgb(18, 52, 86)", "#123456"),
        ("rgb(0, 255, 255)", "#0ff"),
        ("rgba(255, 0, 0, 0.5)", "#ff000080"),
        ("rgba(170, 187, 204, 0.2)", "#abc3"),
        ("rgba(255, 255, 255, 1)", "#fff"),
        ("hsl(0, 100%, 50%)", "red"),
        ("hsl(120, 100%, 50%)", "#0f0"),
        ("hsl(240, 100%, 50%)", "#00f"),
        ("hsl(300, 100%, 50%)", "#f0f"),
        ("hsla(0, 100%, 50%, 0.5)", "#ff000080"),
        ("hsla(240, 100%, 50%, 1)", "#00f"),
        ("red", "red"),
        ("blue", "#00f"),
        ("white", "#fff"),
        ("rebeccapurple", "#639"),
        ("transparent", "#0000"),
        ("", ""),
        ("invalid", "invalid"),
        ("rgb(300, 0, 0)", "rgb(300, 0, 0)"),
        ("#gggggg", "#gggggg"),
        ("hsl(400, 200%, 150%)", "hsl(400, 200%, 150%)"),
        ("  #ff0000  ", "red"),
        (" rgb(255, 0, 0) ", "red"),
        ("  white  ", "#fff"),
        ("#FF0000", "red"),
        ("RGB(255, 0, 0)", "red"),
        ("WHITE", "#fff"),
        ("Red", "red"),
    ];

    println!("🧪 Тестируем функцию сокращения цветов:\n");

    let mut passed = 0;
    let mut failed = 0;

    for &(input, expected) in test_cases {
        let result = shorten_css_color(input);
        let status = if result == expected {
            passed += 1;
            "✅"
        } else {
            failed += 1;
            "❌"
        };

        println!(
            "{:<25} -> {:<12} {} {}",
            format!("\"{}\"", input),
            format!("\"{}\"", result),
            status,
            if result != expected {
                format!("(ожидался \"{}\")", expected)
            } else {
                String::new()
            }
        );
    }

    println!(
        "\n📊 Результаты: {} ✅ прошло, {} ❌ не прошло",
        passed, failed
    );

    if failed == 0 {
        println!("🎉 Все тесты прошли успешно!");
    }

    print_benchmark_summary();
}
