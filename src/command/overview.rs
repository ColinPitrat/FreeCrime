use freecrime::resources::parsers;
use std::fs;
use std::path::Path;

pub fn execute(cmp_path: &str) -> anyhow::Result<()> {
    let map_data = fs::read(cmp_path)?;
    let map = parsers::cmp::parse_cmp(&map_data)?;

    let mut pixels = vec![0u8; 256 * 256 * 3];
    for y in 0..256 {
        for x in 0..256 {
            let col = &map.grid[y * 256 + x];
            // Sample highest block type
            let mut bt_idx = 0;
            for z in (0..6).rev() {
                if col.levels[z] != 0 {
                    bt_idx = col.levels[z];
                    break;
                }
            }

            let color = if bt_idx == 0 {
                [0, 0, 0] // Air
            } else {
                let bt = &map.block_types[bt_idx as usize];
                match bt.block_type() {
                    1 => [0, 0, 255],   // Water
                    2 => [100, 100, 100], // Road
                    3 => [200, 200, 0], // Pavement
                    4 => [0, 200, 0],   // Field
                    5 => [200, 100, 0], // Building
                    _ => [50, 50, 50],
                }
            };

            let idx = (y * 256 + x) * 3;
            pixels[idx..idx+3].copy_from_slice(&color);
        }
    }

    let out_path = Path::new(cmp_path).with_extension("bmp");
    let img = image::RgbImage::from_raw(256, 256, pixels).unwrap();
    img.save(&out_path)?;

    println!("Overview generated: {:?}", out_path);
    Ok(())
}
