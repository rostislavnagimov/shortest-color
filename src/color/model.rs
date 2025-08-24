#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline(always)]
    pub const fn is_opaque(self) -> bool {
        self.a == 255
    }

    #[inline(always)]
    pub const fn is_transparent(self) -> bool {
        self.a == 0
    }
}