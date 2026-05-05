use freecrime::resources::parsers;
use std::fs;
use std::path::Path;

pub fn execute(path: &str, out: &str) -> anyhow::Result<()> {
    let data = fs::read(path)?;
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_uppercase();

    match ext.as_str() {
        "GRY" | "G24" => {
            let style = parsers::gry::parse_gry(&data)?;
            fs::create_dir_all(out)?;
            for (i, block) in style.blocks.iter().enumerate() {
                let rgba = block.to_rgba(&style.palette);
                let img = image::RgbaImage::from_raw(64, 64, rgba).unwrap();
                img.save(Path::new(out).join(format!("block_{:03}.png", i)))?;
            }
            println!("Extracted {} blocks to {}", style.blocks.len(), out);
        }
        "FON" => {
            let font = parsers::fon::parse_fon(&data)?;
            fs::create_dir_all(out)?;
            for (i, glyph) in font.glyphs.iter().enumerate() {
                let mut rgba = Vec::with_capacity(glyph.width as usize * glyph.height as usize * 4);
                for &p in &glyph.pixels {
                    if p == 0 {
                        rgba.extend_from_slice(&[0, 0, 0, 0]);
                    } else {
                        rgba.extend_from_slice(&[255, 255, 255, 255]);
                    }
                }
                if let Some(img) = image::RgbaImage::from_raw(glyph.width as u32, glyph.height as u32, rgba) {
                    img.save(Path::new(out).join(format!("glyph_{:03}.png", i)))?;
                }
            }
            println!("Extracted {} glyphs to {}", font.glyphs.len(), out);
        }
        "SDT" => {
            let raw_path = Path::new(path).with_extension("RAW");
            if !raw_path.exists() {
                anyhow::bail!("Associated RAW file not found: {:?}", raw_path);
            }
            let raw_data = fs::read(raw_path)?;
            let indices = parsers::sdt::parse_sdt(&data)?;
            fs::create_dir_all(out)?;
            for (i, record) in indices.iter().enumerate() {
                if record.size > 0 {
                    let sample = &raw_data[record.offset as usize..(record.offset + record.size) as usize];
                    fs::write(Path::new(out).join(format!("sample_{:03}.raw", i)), sample)?;
                }
            }
            println!("Extracted {} samples to {}", indices.len(), out);
        }
        _ => println!("Extraction not supported for extension: {}", ext),
    }
    Ok(())
}
