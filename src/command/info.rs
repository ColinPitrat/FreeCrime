use freecrime::resources::parsers;
use std::fs;
use std::path::Path;

pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_uppercase();

    match ext.as_str() {
        "FXT" => {
            let bundle = parsers::fxt::parse_fxt(&data)?;
            println!("FXT Text File: {} entries", bundle.entries.len());
        }
        "CMP" => {
            let map = parsers::cmp::parse_cmp(&data)?;
            println!("CMP Map File:");
            println!("  Style Index: {}", map.style_index);
            println!("  Block Types: {}", map.block_types.len());
            println!("  Objects: {}", map.objects.len());
            println!("  Routes: {}", map.routes.len());
            let mut max_side = 0;
            let mut max_lid = 0;
            for bt in &map.block_types {
                max_side = max_side.max(bt.left).max(bt.right).max(bt.top).max(bt.bottom);
                max_lid = max_lid.max(bt.lid);
            }
            println!("  Max Tile Indices: side={}, lid={}", max_side, max_lid);
            }

        "GRY" | "G24" => {
            let style = parsers::gry::parse_gry(&data)?;
            println!("Style File ({}):", ext);
            println!("  Blocks: {} ({} side, {} lid, {} aux)", 
                style.blocks.len(), style.side_count, style.lid_count, style.aux_count);
            println!("  Animations: {}", style.animations.len());
            println!("  Cars: {}", style.cars.len());
            println!("  Objects: {}", style.objects.len());
            println!("  Sprites: {}", style.sprites.len());
        }
        "FON" => {
            let font = parsers::fon::parse_fon(&data)?;
            println!("FON Font File:");
            println!("  Glyphs: {}", font.glyphs.len());
            if let Some(first) = font.glyphs.first() {
                println!("  Glyph Height: {}", first.height);
            }
        }
        "SDT" => {
            let indices = parsers::sdt::parse_sdt(&data)?;
            println!("SDT Sound Index: {} records", indices.len());
        }
        "INI" => {
            let content = String::from_utf8_lossy(&data);
            let mission = parsers::ini::parse_mission(&content)?;
            println!("MISSION.INI File: {} entries", mission.entries.len());
        }
        _ => println!("Unsupported file extension: {}", ext),
    }
    Ok(())
}
