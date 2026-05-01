use super::graphics::{IndexedImage, Palette};

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub glyphs: Vec<IndexedImage>,
    pub palette: Palette,
}
