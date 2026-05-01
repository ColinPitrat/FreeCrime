use crate::resources::types::graphics::{IndexedImage, Palette};
use crate::resources::{Result, Error};

pub fn parse_act(data: &[u8]) -> Result<Palette> {
    if data.len() < 768 {
        return Err(Error::Parse(format!("ACT file too small: {} bytes", data.len())));
    }
    
    let mut colors = [[0u8; 3]; 256];
    for i in 0..256 {
        colors[i][0] = data[i * 3];
        colors[i][1] = data[i * 3 + 1];
        colors[i][2] = data[i * 3 + 2];
    }
    
    if data.len() > 768 {
        return Err(Error::Parse(format!("ACT file has {} trailing bytes", data.len() - 768)));
    }
    
    Ok(Palette { colors })
}

pub fn parse_rat(data: &[u8], width: u32, height: u32) -> Result<IndexedImage> {
    let expected_size = (width * height) as usize;
    if data.len() != expected_size {
        return Err(Error::Parse(format!(
            "RAT file size mismatch: expected {} bytes for {}x{}, got {}",
            expected_size, width, height, data.len()
        )));
    }
    Ok(IndexedImage::new(width, height, data.to_vec()))
}

pub fn write_rgba_bmp(path: &str, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;

    let file_header_size = 14;
    let info_header_size = 40;
    let pixel_data_offset = file_header_size + info_header_size;
    let image_size = width * height * 4;
    let file_size = pixel_data_offset + image_size;

    // File Header
    file.write_all(b"BM")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(&[0, 0, 0, 0])?; // Reserved
    file.write_all(&pixel_data_offset.to_le_bytes())?;

    // Info Header
    file.write_all(&info_header_size.to_le_bytes())?;
    file.write_all(&(width as i32).to_le_bytes())?;
    file.write_all(&(-(height as i32)).to_le_bytes())?; // Top-down
    file.write_all(&1u16.to_le_bytes())?; // Planes
    file.write_all(&32u16.to_le_bytes())?; // Bits per pixel
    file.write_all(&0u32.to_le_bytes())?; // Compression (none)
    file.write_all(&image_size.to_le_bytes())?;
    file.write_all(&0i32.to_le_bytes())?; // XPelsPerMeter
    file.write_all(&0i32.to_le_bytes())?; // YPelsPerMeter
    file.write_all(&0u32.to_le_bytes())?; // ClrUsed
    file.write_all(&0u32.to_le_bytes())?; // ClrImportant

    // Pixel Data (RGBA -> BGRA for BMP)
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let r = rgba[idx];
            let g = rgba[idx + 1];
            let b = rgba[idx + 2];
            let a = rgba[idx + 3];
            file.write_all(&[b, g, r, a])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_act() {
        let mut data = vec![0u8; 768];
        data[0] = 255;
        data[767] = 128;
        let palette = parse_act(&data).unwrap();
        assert_eq!(palette.colors[0][0], 255);
        assert_eq!(palette.colors[255][2], 128);
    }

    #[test]
    fn test_parse_rat() {
        let data = vec![42u8; 100];
        let img = parse_rat(&data, 10, 10).unwrap();
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
        assert_eq!(img.pixels[0], 42);
    }
}
