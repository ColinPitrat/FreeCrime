use crate::resources::types::font::Font;
use crate::resources::types::graphics::{IndexedImage, Palette};
use crate::resources::{Result, Error};
use std::io::{Read, Cursor};

pub fn parse_fon(data: &[u8]) -> Result<Font> {
    let mut cursor = Cursor::new(data);

    let mut num_pictures_buf = [0u8; 1];
    cursor.read_exact(&mut num_pictures_buf)?;
    let num_pictures = num_pictures_buf[0] as usize;

    let mut height_buf = [0u8; 1];
    cursor.read_exact(&mut height_buf)?;
    let height = height_buf[0] as u32;

    let mut glyphs = Vec::with_capacity(num_pictures);
    for _ in 0..num_pictures {
        let mut width_buf = [0u8; 1];
        cursor.read_exact(&mut width_buf)?;
        let width = width_buf[0] as u32;

        let mut pixels = vec![0u8; (width * height) as usize];
        cursor.read_exact(&mut pixels)?;

        glyphs.push(IndexedImage::new(width, height, pixels));
    }

    let mut palette_data = [0u8; 768];
    cursor.read_exact(&mut palette_data)?;

    let mut colors = [[0u8; 3]; 256];
    for i in 0..256 {
        colors[i][0] = palette_data[i * 3];
        colors[i][1] = palette_data[i * 3 + 1];
        colors[i][2] = palette_data[i * 3 + 2];
    }

    // Strictness check: check if all data was consumed
    let current_pos = cursor.position();
    if current_pos < data.len() as u64 {
        return Err(Error::Parse(format!(
            "FON file has {} trailing bytes",
            data.len() as u64 - current_pos
        )));
    }

    Ok(Font {
        glyphs,
        palette: Palette { colors },
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fon() {
        let mut data = Vec::new();
        data.push(2); // num_pictures
        data.push(8); // height
        
        // Picture 1: width 2, 16 pixels
        data.push(2);
        data.extend(vec![1u8; 16]);
        
        // Picture 2: width 1, 8 pixels
        data.push(1);
        data.extend(vec![2u8; 8]);
        
        // Palette: 768 bytes
        let mut palette = vec![0u8; 768];
        palette[0] = 63; // Red of color 0
        data.extend(palette);
        
        let font = parse_fon(&data).unwrap();
        assert_eq!(font.glyphs.len(), 2);
        assert_eq!(font.glyphs[0].width, 2);
        assert_eq!(font.glyphs[0].height, 8);
        assert_eq!(font.glyphs[1].width, 1);
        assert_eq!(font.palette.colors[0][0], 63);
    }
}
