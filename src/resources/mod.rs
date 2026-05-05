pub mod types;
pub mod parsers;

pub use types::text::TextBundle;
pub use types::map::Map;
pub use types::style::Style;
pub use types::font::Font;
pub use types::graphics::{IndexedImage, Palette};
pub use parsers::ini::Mission;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Binary read error: {0}")]
    BinRead(#[from] binrw::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// A coherent bundle of resources representing a full game city/level.
#[derive(Debug, Clone)]
pub struct CityBundle {
    pub map: Map,
    pub style: Style,
    pub mission: Mission,
    pub text: TextBundle,
}

impl CityBundle {
    pub fn load_gta1(
        map_data: &[u8],
        style_data: &[u8],
        mission_data: &str,
        text_data: &[u8],
    ) -> Result<Self> {
        let map = parsers::cmp::parse_cmp(map_data)?;
        let style = parsers::gry::parse_gry(style_data)?;
        let mission = parsers::ini::parse_mission(mission_data)?;
        let text = parsers::fxt::parse_fxt(text_data)?;

        Ok(Self { map, style, mission, text })
    }
}
