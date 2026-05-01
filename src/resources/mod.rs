pub mod types;
pub mod parsers;

pub use types::text::TextBundle;
pub use types::map::Map;
pub use types::style::Style;
pub use types::font::Font;
pub use types::graphics::{IndexedImage, Palette};
pub use parsers::ini::Mission;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Parse(s) => write!(f, "Parse error: {}", s),
            Error::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
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
