use freecrime::resources::parsers;
use std::fs;
use std::path::Path;

pub fn execute(cmp_path: &str, out_path_override: Option<&str>) -> anyhow::Result<()> {
    let map_data = fs::read(cmp_path)?;
    let map = parsers::cmp::parse_cmp(&map_data)?;

    let mut pixels = vec![0u8; map.width * map.height * 3];
    for y in 0..map.height {
        for x in 0..map.width {
            // Sample highest block
            let mut top_block = None;
            for z in 0..map.depth {
                if let Some(block) = map.get_block(x, y, z) {
                    top_block = Some(block);
                    break;
                }
            }

            let color = if let Some(block) = top_block {
                match block.block_type() {
                    1 => [0, 0, 255],   // Water
                    2 => [100, 100, 100], // Road
                    3 => [200, 200, 0], // Pavement
                    4 => [0, 200, 0],   // Field
                    5 => [200, 100, 0], // Building
                    _ => [50, 50, 50],
                }
            } else {
                [0, 0, 0] // Air
            };

            let idx = (y * map.width + x) * 3;
            pixels[idx..idx+3].copy_from_slice(&color);
        }
    }

    let out_path = if let Some(p) = out_path_override {
        Path::new(p).to_path_buf()
    } else {
        Path::new(cmp_path).with_extension("bmp")
    };

    let img = image::RgbImage::from_raw(map.width as u32, map.height as u32, pixels).unwrap();
    img.save(&out_path)?;

    println!("Overview generated: {:?}", out_path);
    Ok(())
}
