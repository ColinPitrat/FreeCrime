use freecrime::resources::parsers;
use std::fs;
use std::path::Path;

pub fn execute(path: &str, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_uppercase();

    match ext.as_str() {
        "FXT" => {
            let bundle = parsers::fxt::parse_fxt(&data)?;
            let mut content = String::new();
            let mut keys: Vec<_> = bundle.entries.keys().collect();
            keys.sort();
            for key in keys {
                content.push_str(&format!("[{}] {}\n", key, bundle.entries[key]));
            }
            fs::write(out, content)?;
            println!("Extracted FXT to {}", out);
        }
        "FON" => {
            let font = parsers::fon::parse_fon(&data)?;
            fs::create_dir_all(out)?;
            for (i, glyph) in font.glyphs.iter().enumerate() {
                let rgba = glyph.to_rgba(&font.palette);
                let out_file = format!("{}/glyph_{:03}.bmp", out, i);
                parsers::raw::write_rgba_bmp(&out_file, glyph.width, glyph.height, &rgba)?;
            }
            println!("Extracted {} glyphs to {}", font.glyphs.len(), out);
        }
        "GRY" | "G24" => {
             let style = parsers::gry::parse_gry(&data)?;
             fs::create_dir_all(out)?;
             for i in 0..std::cmp::min(100, style.blocks.len()) {
                 let block = &style.blocks[i];
                 let rgba = block.to_rgba(&style.palette);
                 let out_file = format!("{}/block_{:03}.bmp", out, i);
                 parsers::raw::write_rgba_bmp(&out_file, block.width, block.height, &rgba)?;
             }
             println!("Extracted first 100 blocks to {}", out);
        }
        "SDT" => {
            let indices = parsers::sdt::parse_sdt(&data)?;
            let mut content = String::new();
            content.push_str("Index,Offset,Size,Frequency\n");
            for (i, idx) in indices.iter().enumerate() {
                content.push_str(&format!("{},{},{},{}\n", i, idx.offset, idx.size, idx.frequency));
            }
            fs::write(out, content)?;
            println!("Extracted SDT info to {}", out);
        }
        "INI" => {
            let content = String::from_utf8_lossy(&data);
            let mission = parsers::ini::parse_mission(&content)?;
            let mut cleaned = String::new();
            for entry in mission.entries {
                match entry {
                    freecrime::resources::parsers::ini::MissionEntry::Header(name, params) => {
                        cleaned.push_str(&format!("{} {}\n", name, params.join(" ")));
                    }
                    freecrime::resources::parsers::ini::MissionEntry::Command { id, name, .. } => {
                        cleaned.push_str(&format!("{} {}\n", id, name));
                    }
                }
            }
            fs::write(out, cleaned)?;
            println!("Extracted cleaned MISSION.INI to {}", out);
        }
        _ => println!("Extraction not implemented for {}", ext),
    }
    Ok(())
}
