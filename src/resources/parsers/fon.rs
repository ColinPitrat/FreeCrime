use crate::resources::types::font::Font;
use crate::resources::types::graphics::{IndexedImage, Palette};
use crate::resources::{Result, Error};
use std::io::{Cursor, Read};
use binrw::{BinRead, BinReaderExt};

#[derive(BinRead)]
#[br(little)]
struct FonHeader {
    num_pictures: u8,
    height: u8,
}

#[derive(BinRead)]
#[br(little, import(height: u32))]
struct GlyphRaw {
    width: u8,
    #[br(count = width as u32 * height)]
    pixels: Vec<u8>,
}

/// Parses a FON font file.
/// These files contain a variable number of glyphs with a fixed height and a shared 8-bit palette.
pub fn parse_fon(data: &[u8]) -> Result<Font> {
    let mut cursor = Cursor::new(data);
    let header: FonHeader = cursor.read_le()?;

    let mut glyphs = Vec::with_capacity(header.num_pictures as usize);
    for _ in 0..header.num_pictures {
        let g: GlyphRaw = cursor.read_le_args((header.height as u32,))?;
        glyphs.push(IndexedImage::new(g.width as u32, header.height as u32, g.pixels));
    }

    let mut palette_data = [0u8; 768];
    cursor.read_exact(&mut palette_data)?;

    let mut colors = [[0u8; 3]; 256];
    for i in 0..256 {
        colors[i][0] = palette_data[i * 3];
        colors[i][1] = palette_data[i * 3 + 1];
        colors[i][2] = palette_data[i * 3 + 2];
    }

    if cursor.position() < data.len() as u64 {
        return Err(Error::Parse(format!("FON file has {} trailing bytes", data.len() as u64 - cursor.position())));
    }

    Ok(Font { glyphs, palette: Palette { colors } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fon() {
        let mut data = Vec::new();
        data.push(2); // num_pictures
        data.push(8); // height
        data.push(2); data.extend(vec![1u8; 16]); // Glyph 1
        data.push(1); data.extend(vec![2u8; 8]); // Glyph 2
        let mut palette = vec![0u8; 768];
        palette[0] = 63;
        data.extend(palette);

        let font = parse_fon(&data).unwrap();
        assert_eq!(font.glyphs.len(), 2);
        assert_eq!(font.glyphs[0].width, 2);
        assert_eq!(font.glyphs[0].height, 8);
        assert_eq!(font.glyphs[1].width, 1);
        assert_eq!(font.palette.colors[0][0], 63);
    }
}
