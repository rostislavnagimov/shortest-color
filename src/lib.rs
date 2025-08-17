pub mod color;

pub use color::{convert_to_color, shorten_color, is_valid_color};

pub fn shorten_css_color(color_str: &str) -> String {
    let start = std::time::Instant::now();
    
    let validation_start = std::time::Instant::now();
    let is_valid = is_valid_color(color_str);
    let validation_time = validation_start.elapsed();
    
    if !is_valid {
        let total_time = start.elapsed();
        println!("shorten_css_color('{}') -> '{}' | validation: {:?}, total: {:?}", 
                 color_str, color_str, validation_time, total_time);
        return color_str.to_string();
    }
    
    let conversion_start = std::time::Instant::now();
    let result = if let Some(color) = convert_to_color(color_str) {
        let shortening_start = std::time::Instant::now();
        let shortened = shorten_color(&color);
        let shortening_time = shortening_start.elapsed();
        
        let conversion_time = conversion_start.elapsed();
        let total_time = start.elapsed();
        
        println!("shorten_css_color('{}') -> '{}' | validation: {:?}, conversion: {:?}, shortening: {:?}, total: {:?}", 
                 color_str, shortened, validation_time, conversion_time, shortening_time, total_time);
        shortened
    } else {
        let conversion_time = conversion_start.elapsed();
        let total_time = start.elapsed();
        
        println!("shorten_css_color('{}') -> '{}' | validation: {:?}, conversion: {:?} (failed), total: {:?}", 
                 color_str, color_str, validation_time, conversion_time, total_time);
        color_str.to_string()
    };
    
    result
}