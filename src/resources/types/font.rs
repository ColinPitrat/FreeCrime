use super::graphics::{IndexedImage, Palette};

/// Represents a game font with a set of indexed glyphs and a shared palette.
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    /// List of glyph images.
    pub glyphs: Vec<IndexedImage>,
    /// The palette used by all glyphs in this font.
    pub palette: Palette,
}
