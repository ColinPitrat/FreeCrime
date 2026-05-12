pub mod types;
pub mod parsers;

pub use types::text::TextBundle;
pub use types::map::Map;
pub use types::style::Style;
pub use types::font::Font;
pub use types::graphics::{IndexedImage, Palette};
pub use parsers::ini::Mission;

/// Error types for the FreeCrime resource library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors occurring during file format parsing.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Low-level binary reading errors from binrw.
    #[error("Binary read error: {0}")]
    BinRead(#[from] binrw::Error),

    /// Miscellaneous errors.
    #[error("Other error: {0}")]
    Other(String),
}

/// Library-specific Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A coherent bundle of resources representing a full game city/level.
/// Combines map geometry, visual style, mission scripts, and localized text.
#[derive(Debug, Clone)]
pub struct CityBundle {
    pub map: Map,
    pub style: Style,
    pub mission: Mission,
    pub text: TextBundle,
}

impl CityBundle {
    /// Utility to load all primary components for a GTA 1 level from memory.
    pub fn load_gta1(
        map_data: &[u8],
        style_data: &[u8],
        mission_data: &str,
        text_data: &[u8],
    ) -> Result<Self> {
        let map = parsers::cmp::parse_cmp(map_data)?;
        let lid_flatness = map.get_lid_flatness();
        let style = parsers::gry::parse_gry(style_data, Some(&lid_flatness))?;
        let mission = parsers::ini::parse_mission(mission_data)?;
        let text = parsers::fxt::parse_fxt(text_data)?;

        Ok(Self { map, style, mission, text })
    }
}
