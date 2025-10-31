use shortest_color::shorten_css_color;

#[test]
fn test_hex_colors() {
    assert_eq!(shorten_css_color("#ff0000"), "red");
    assert_eq!(shorten_css_color("#008000"), "green");
    assert_eq!(shorten_css_color("#000080"), "navy");
    assert_eq!(shorten_css_color("#800080"), "purple");
    assert_eq!(shorten_css_color("#808080"), "gray");

    assert_eq!(shorten_css_color("#000000"), "#000");
    assert_eq!(shorten_css_color("#ffffff"), "#fff");
    assert_eq!(shorten_css_color("#ff00ff"), "#f0f");
    assert_eq!(shorten_css_color("#00ffff"), "#0ff");
    assert_eq!(shorten_css_color("#ffff00"), "#ff0");
    assert_eq!(shorten_css_color("#aabbcc"), "#abc");

    assert_eq!(shorten_css_color("#FF0000"), "red");
    assert_eq!(shorten_css_color("#FFFFFF"), "#fff");
    assert_eq!(shorten_css_color("#d2b48c"), "tan");
    assert_eq!(shorten_css_color("#F00"), "red");
}

#[test]
fn test_hex_with_alpha() {
    assert_eq!(shorten_css_color("#ff000080"), "#ff000080");
    assert_eq!(shorten_css_color("#ffffff00"), "#fff0");
    assert_eq!(shorten_css_color("#000000ff"), "#000");
    assert_eq!(shorten_css_color("#aabbccdd"), "#abcd");
}

#[test]
fn test_rgb_colors() {
    assert_eq!(shorten_css_color("rgb(255, 0, 0)"), "red");
    assert_eq!(shorten_css_color("rgb(170, 187, 204)"), "#abc");
    assert_eq!(shorten_css_color("rgb(0, 255, 255)"), "#0ff");
    assert_eq!(shorten_css_color("rgb(0, 0, 0)"), "#000");
    assert_eq!(shorten_css_color("rgb(255, 255, 255)"), "#fff");

    assert_eq!(shorten_css_color("rgb(255,0,0)"), "red");
    assert_eq!(shorten_css_color("rgb( 255 , 0 , 0 )"), "red");
    assert_eq!(shorten_css_color("rgb(255 0 0)"), "red");
    assert_eq!(shorten_css_color("rgb(  255  0  0  )"), "red");

    assert_eq!(shorten_css_color("rgb(255.0, 0.0, 0.0)"), "red");
    assert_eq!(shorten_css_color("rgb(255.5, 0.5, 0.5)"), "#ff0101");
    assert_eq!(shorten_css_color("rgb(0.4, 0.4, 0.4)"), "#000");
    assert_eq!(shorten_css_color("rgb(0.6, 0.6, 0.6)"), "#010101");

    assert_eq!(shorten_css_color("RGB(255, 0, 0)"), "red");
}

#[test]
fn test_rgba_colors() {
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.5)"), "#ff000080");
    assert_eq!(shorten_css_color("rgba(170, 187, 204, 0.2)"), "#abc3");
    assert_eq!(shorten_css_color("rgba(255, 255, 255, 1)"), "#fff");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 1.0)"), "red");
    assert_eq!(shorten_css_color("rgba(0, 0, 0, 0)"), "#0000");

    assert_eq!(shorten_css_color("rgba(255, 0, 0, 50%)"), "#ff000080");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 100%)"), "red");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0%)"), "#f000");

    assert_eq!(shorten_css_color("rgba(255,0,0,0.5)"), "#ff000080");
    assert_eq!(shorten_css_color("rgba( 255 , 0 , 0 , 0.5 )"), "#ff000080");
    assert_eq!(shorten_css_color("rgba(255 0 0 0.5)"), "#ff000080");

    assert_eq!(shorten_css_color("rgba(255, 0, 0, .5)"), "#ff000080");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.)"), "#f000");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.001)"), "#f000");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.999)"), "red");

    assert_eq!(shorten_css_color("rgba(aqua, 0.5)"), "#00ffff80");
    assert_eq!(shorten_css_color("rgba(navy, 0.5)"), "#00008080");
}

#[test]
fn test_hsl_colors() {
    assert_eq!(shorten_css_color("hsl(0, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(120, 100%, 50%)"), "#0f0");
    assert_eq!(shorten_css_color("hsl(240, 100%, 50%)"), "#00f");
    assert_eq!(shorten_css_color("hsl(300, 100%, 50%)"), "#f0f");
    assert_eq!(shorten_css_color("hsl(60, 100%, 50%)"), "#ff0");
    assert_eq!(shorten_css_color("hsl(180, 100%, 50%)"), "#0ff");

    assert_eq!(shorten_css_color("hsl(0deg, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(360deg, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(1turn, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(6.283rad, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(400grad, 100%, 50%)"), "red");

    assert_eq!(shorten_css_color("hsl(0,100%,50%)"), "red");
    assert_eq!(shorten_css_color("hsl( 0 , 100% , 50% )"), "red");
    assert_eq!(shorten_css_color("hsl(0 100% 50%)"), "red");

    assert_eq!(shorten_css_color("hsl(720deg, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(-360deg, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(-90deg, 100%, 50%)"), "#8000ff");

    assert_eq!(shorten_css_color("hsl(0, 0%, 0%)"), "#000");
    assert_eq!(shorten_css_color("hsl(0, 0%, 100%)"), "#fff");
    assert_eq!(shorten_css_color("hsl(0, 0%, 50%)"), "gray");

    assert_eq!(shorten_css_color("HSL(0, 100%, 50%)"), "red");
}

#[test]
fn test_hsla_colors() {
    assert_eq!(shorten_css_color("hsla(0, 100%, 50%, 0.5)"), "#ff000080");
    assert_eq!(shorten_css_color("hsla(240, 100%, 50%, 1)"), "#00f");
    assert_eq!(shorten_css_color("hsla(0, 100%, 50%, 100%)"), "red");
    assert_eq!(shorten_css_color("hsla(0, 100%, 50%, 50%)"), "#ff000080");

    assert_eq!(shorten_css_color("hsla(0,100%,50%,0.5)"), "#ff000080");
    assert_eq!(
        shorten_css_color("hsla( 0 , 100% , 50% , 0.5 )"),
        "#ff000080"
    );
    assert_eq!(shorten_css_color("hsla(0 100% 50% 0.5)"), "#ff000080");

    assert_eq!(shorten_css_color("rgba(lime, 0.5)"), "#00ff0080");
    assert_eq!(shorten_css_color("rgba(teal, 0.5)"), "#00808080");
}

#[test]
fn test_color_keywords() {
    assert_eq!(shorten_css_color("red"), "red");
    assert_eq!(shorten_css_color("blue"), "blue");
    assert_eq!(shorten_css_color("white"), "#fff");
    assert_eq!(shorten_css_color("black"), "#000");
    assert_eq!(shorten_css_color("yellow"), "#ff0");
    assert_eq!(shorten_css_color("magenta"), "#f0f");
    assert_eq!(shorten_css_color("cyan"), "cyan");

    assert_eq!(shorten_css_color("rebeccapurple"), "#639");
    assert_eq!(shorten_css_color("transparent"), "#0000");
    assert_eq!(shorten_css_color("silver"), "silver");
    assert_eq!(shorten_css_color("gray"), "gray");
    assert_eq!(shorten_css_color("maroon"), "maroon");
    assert_eq!(shorten_css_color("olive"), "olive");
    assert_eq!(shorten_css_color("fuchsia"), "#f0f");
    assert_eq!(shorten_css_color("lime"), "lime");
    assert_eq!(shorten_css_color("aqua"), "aqua");
    assert_eq!(shorten_css_color("navy"), "navy");
    assert_eq!(shorten_css_color("teal"), "teal");
    assert_eq!(shorten_css_color("purple"), "purple");

    assert_eq!(shorten_css_color("WHITE"), "#fff");
    assert_eq!(shorten_css_color("Red"), "red");
    assert_eq!(shorten_css_color("BLUE"), "blue");
}

#[test]
fn test_whitespace_handling() {
    assert_eq!(shorten_css_color("  #ff0000  "), "red");
    assert_eq!(shorten_css_color(" rgb(255, 0, 0) "), "red");
    assert_eq!(shorten_css_color("  white  "), "#fff");
    assert_eq!(shorten_css_color("   red   "), "red");
    assert_eq!(shorten_css_color("       #ffffff "), "#fff");
    assert_eq!(shorten_css_color("\n rgb(0, 255, 0) \n"), "#0f0");
    assert_eq!(shorten_css_color("  transparent  "), "#0000");
}

#[test]
fn test_invalid_colors() {
    assert_eq!(shorten_css_color(""), "");
    assert_eq!(shorten_css_color("invalid"), "invalid");
    assert_eq!(shorten_css_color("rgb(300, 0, 0)"), "rgb(300, 0, 0)");
    assert_eq!(shorten_css_color("#gggggg"), "#gggggg");
    assert_eq!(
        shorten_css_color("hsl(400, 200%, 150%)"),
        "hsl(400, 200%, 150%)"
    );
    assert_eq!(shorten_css_color("rgb(255, 0)"), "rgb(255, 0)");
    assert_eq!(shorten_css_color("rgba(255, 0, 0)"), "rgba(255, 0, 0)");
    assert_eq!(shorten_css_color("hsl(0, 100%)"), "hsl(0, 100%)");
    assert_eq!(
        shorten_css_color("hsla(0, 100%, 50%)"),
        "hsla(0, 100%, 50%)"
    );
    assert_eq!(
        shorten_css_color("rgb(-10, -10, -10)"),
        "rgb(-10, -10, -10)"
    );
    assert_eq!(
        shorten_css_color("rgba(255, 0, 0, 1.5)"),
        "rgba(255, 0, 0, 1.5)"
    );
    assert_eq!(
        shorten_css_color("rgba(255, 0, 0, -0.5)"),
        "rgba(255, 0, 0, -0.5)"
    );
}

#[test]
fn test_edge_cases() {
    assert_eq!(shorten_css_color("a"), "a");
    assert_eq!(shorten_css_color("ab"), "ab");
    assert_eq!(shorten_css_color("abc"), "abc");
    assert_eq!(shorten_css_color("abcd"), "abcd");

    assert_eq!(shorten_css_color("#fff"), "#fff");
    assert_eq!(shorten_css_color("#abc"), "#abc");
    assert_eq!(shorten_css_color("#f0a2"), "#f0a2");

    assert_eq!(shorten_css_color("rgb(255.4, 255.4, 255.4)"), "#fff");
    assert_eq!(shorten_css_color("rgb(255.6, 255.6, 255.6)"), "#fff");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.004)"), "#ff000001");
    assert_eq!(shorten_css_color("rgba(255, 0, 0, 0.996)"), "#ff0000fe");

    assert_eq!(shorten_css_color("rgba(128, 128, 128, 0.5)"), "#80808080");
    assert_eq!(shorten_css_color("rgba(64, 128, 192, 0.25)"), "#4080c040");

    assert_eq!(shorten_css_color("hsl(0.001deg, 100%, 50%)"), "red");
    assert_eq!(shorten_css_color("hsl(359.999deg, 100%, 50%)"), "red");
}
