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
        
        ("rgb(255,0,0)", "red"),
        ("rgb( 255 , 0 , 0 )", "red"),
        ("rgb(255 0 0)", "red"),
        ("rgb(255  0  0)", "red"),
        ("rgb(  255   0   0  )", "red"),
        
        ("rgba(255, 0, 0, 0.5)", "#ff000080"),
        ("rgba(170, 187, 204, 0.2)", "#abc3"),
        ("rgba(255, 255, 255, 1)", "#fff"),
        ("rgba(255, 0, 0, 1.0)", "red"),
        
        ("rgba(255, 0, 0, 50%)", "#ff000080"),
        ("rgba(255, 0, 0, 100%)", "red"),
        ("rgba(255, 0, 0, 0%)", "#f000"),
        
        ("rgba(255,0,0,0.5)", "#ff000080"),
        ("rgba( 255 , 0 , 0 , 0.5 )", "#ff000080"),
        ("rgba(255 0 0 0.5)", "#ff000080"),
        ("rgba(  255  0  0  0.5  )", "#ff000080"),
        
        ("hsl(0, 100%, 50%)", "red"),
        ("hsl(120, 100%, 50%)", "#0f0"),
        ("hsl(240, 100%, 50%)", "#00f"),
        ("hsl(300, 100%, 50%)", "#f0f"),
        
        ("hsl(0deg, 100%, 50%)", "red"),
        ("hsl(360deg, 100%, 50%)", "red"),
        ("hsl(1turn, 100%, 50%)", "red"),
        ("hsl(6.283rad, 100%, 50%)", "red"),
        ("hsl(400grad, 100%, 50%)", "red"),
        
        ("hsl(0,100%,50%)", "red"),
        ("hsl( 0 , 100% , 50% )", "red"),
        ("hsl(0 100% 50%)", "red"),
        ("hsl(  0  100%  50%  )", "red"),
        
        ("hsla(0, 100%, 50%, 0.5)", "#ff000080"),
        ("hsla(240, 100%, 50%, 1)", "#00f"),
        ("hsla(0, 100%, 50%, 100%)", "red"),
        ("hsla(0, 100%, 50%, 50%)", "#ff000080"),
        
        ("hsla(0,100%,50%,0.5)", "#ff000080"),
        ("hsla( 0 , 100% , 50% , 0.5 )", "#ff000080"),
        ("hsla(0 100% 50% 0.5)", "#ff000080"),
        
        ("red", "red"),
        ("blue", "blue"),
        ("white", "#fff"),
        ("rebeccapurple", "#639"),
        ("transparent", "#0000"),
        
        ("  #ff0000  ", "red"),
        (" rgb(255, 0, 0) ", "red"),
        ("  white  ", "#fff"),
        ("   red   ", "red"),
        ("       #ffffff ", "#fff"),
        ("\n rgb(0, 255, 0) \n", "#0f0"),
        
        ("#FF0000", "red"),
        ("RGB(255, 0, 0)", "red"),
        ("WHITE", "#fff"),
        ("Red", "red"),
        ("BLUE", "blue"),
        ("hSl(0, 100%, 50%)", "red"),
        ("RGBA(255, 0, 0, 1)", "red"),
        
        ("rgb(255 , 0, 0)", "red"),
        ("rgb(255, 0 , 0)", "red"),
        ("rgba(255 , 0 , 0, 1)", "red"),
        ("hsl(0 , 100%, 50%)", "red"),
        ("hsla(0 , 100% , 50%, 1)", "red"),
        
        ("rgb(255.0, 0.0, 0.0)", "red"),
        ("rgba(255.5, 0.5, 0.5, 1.0)", "#ff0101"),
        ("hsl(0.0, 100.0%, 50.0%)", "red"),
        
        ("rgb(0, 0, 0)", "#000"),
        ("rgb(255, 255, 255)", "#fff"),
        ("rgba(0, 0, 0, 0)", "#0000"),
        ("rgba(255, 255, 255, 0)", "#fff0"),
        ("hsl(0, 0%, 0%)", "#000"),
        ("hsl(0, 0%, 100%)", "#fff"),
        
        ("#fff", "#fff"),
        ("#abc", "#abc"),
        ("#f0a2", "#f0a2"),
        
        ("red", "red"),
        ("blue", "blue"),
        ("teal", "teal"),
        ("lime", "lime"),
        ("navy", "navy"),
        ("aqua", "aqua"),
        
        ("", ""),
        ("invalid", "invalid"),
        ("rgb(300, 0, 0)", "rgb(300, 0, 0)"),
        ("#gggggg", "#gggggg"),
        ("hsl(400, 200%, 150%)", "hsl(400, 200%, 150%)"),
        ("rgb(255, 0)", "rgb(255, 0)"),
        ("rgba(255, 0, 0)", "rgba(255, 0, 0)"),
        ("hsl(0, 100%)", "hsl(0, 100%)"),
        ("hsla(0, 100%, 50%)", "hsla(0, 100%, 50%)"),
        
        ("   rgb(   255   ,   0   ,   0   )   ", "red"),
        ("   rgba(   255   ,   0   ,   0   ,   1   )   ", "red"),
        ("   hsl(   0   ,   100%   ,   50%   )   ", "red"),
        
        ("black", "#000"),
        ("silver", "silver"),
        ("gray", "gray"),
        ("maroon", "maroon"),
        ("olive", "olive"),
        ("fuchsia", "#f0f"),
        
        ("#ff000080", "#ff000080"),
        ("#00ff0080", "#00ff0080"),
        ("#0000ff80", "#0000ff80"),
        ("#ffffff00", "#fff0"),
        ("#000000ff", "#000"),
        
        ("#ff00ff80", "#f0f8"),
        ("#00ffff00", "#0ff0"),
        ("#ffff0080", "#ff08"),
        ("#aabbccdd", "#abcd"),
        
        ("rgb(1, 1, 1)", "#010101"),
        ("rgb(254, 254, 254)", "#fefefe"),
        ("rgb(128, 128, 128)", "gray"),
        
        ("hsl(60, 100%, 50%)", "#ff0"),
        ("hsl(180, 100%, 50%)", "#0ff"),
        ("hsl(0, 100%, 25%)", "maroon"),
        ("hsl(0, 0%, 50%)", "gray"),
        ("hsl(0, 0%, 75%)", "silver"),
        
        ("rgb(255,000,000)", "red"),
        ("rgb(000,255,000)", "#0f0"),
        ("rgb(000,000,255)", "#00f"),
        ("hsl(000, 100%, 050%)", "red"),
        ("hsla(000, 100%, 050%, 001)", "red"),
        
        ("a", "a"),
        ("ab", "ab"),
        ("abc", "abc"),
        ("abcd", "abcd"),
        ("white", "#fff"),
        ("yellow", "#ff0"),
        ("magenta", "#f0f"),
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
            "{:<35} -> {:<15} {} {}",
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
    } else {
        println!("⚠️  Есть проблемы, требующие внимания");
    }

    println!("\n🔍 Тестируем производительность early return:");
    
    let performance_tests = vec![
        "#fff", "#abc", "#1234", "#f0a2",
        "red", "blue", "teal", "aqua", "white", "black",
    ];
    
    for test in &performance_tests {
        let _result = shorten_css_color(test);
    }

    print_benchmark_summary();
}