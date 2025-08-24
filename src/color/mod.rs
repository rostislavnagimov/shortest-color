pub mod converter;
pub mod keywords;
pub mod model;

pub use converter::{convert_to_color, try_convert_color, is_valid_color, shorten_color};
pub use model::Color;