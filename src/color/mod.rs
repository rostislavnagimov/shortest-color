pub mod converter;
pub mod keywords;
pub mod model;
pub mod shortener;
pub mod validator;

pub use converter::convert_to_color;
pub use shortener::shorten_color;
pub use validator::is_valid_color;
pub use model::Color;