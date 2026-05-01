use freecrime::resources::parsers;
use std::fs;

pub fn execute(map_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let map_data = fs::read(map_path)?;
    let map = parsers::cmp::parse_cmp(&map_data)?;
    
    println!("Map: {} block types", map.block_types.len());
    println!("Rendering map overview to 'map_overview.bmp'...");
    
    // Scale: 4x4 pixels per block
    let scale = 4;
    let width = 256 * scale;
    let height = 256 * scale;
    let mut image = vec![0u8; width * height * 4];
    
    for y in 0..256 {
        for x in 0..256 {
            let col = &map.grid[y * 256 + x];
            
            // Find highest non-zero lid
            let mut lid_color = [0, 0, 0];
            for z in 0..6 {
                let block_type_idx = col.levels[z];
                if block_type_idx != 0 {
                     let bt = &map.block_types[block_type_idx as usize];
                     // Note: We use the block type even if lid is 0 for schematic overview
                     let bt_type = bt.block_type();
                     
                     // Skip air
                     if bt_type == 0 && z < 5 { continue; }
                     
                     lid_color = match bt_type {
                         1 => [0, 100, 255],   // water (vibrant blue)
                         2 => [40, 40, 40],    // road (dark grey)
                         3 => [150, 150, 150], // pavement (light grey)
                         4 => [34, 139, 34],   // field (forest green)
                         5 => [178, 34, 34],   // building (firebrick red)
                         _ => [200, 200, 200],
                     };
                     
                     // Dim color based on height (Z 0 is highest, Z 5 is lowest)
                     // Level 0 (top): 1.0 factor
                     // Level 5 (bottom): 0.6 factor
                     let factor = 1.0 - (z as f32 * 0.08);
                     lid_color = [
                         (lid_color[0] as f32 * factor) as u8,
                         (lid_color[1] as f32 * factor) as u8,
                         (lid_color[2] as f32 * factor) as u8,
                     ];
                     break;
                }
            }
            
            // Fill scale*scale pixels
            for sy in 0..scale {
                for sx in 0..scale {
                    let out_idx = ((y * scale + sy) * width + (x * scale + sx)) * 4;
                    image[out_idx] = lid_color[0];
                    image[out_idx + 1] = lid_color[1];
                    image[out_idx + 2] = lid_color[2];
                    image[out_idx + 3] = 255;
                }
            }
        }
    }
    
    parsers::raw::write_rgba_bmp("map_overview.bmp", width as u32, height as u32, &image)?;
    println!("Overview rendered to map_overview.bmp");
    
    Ok(())
}
