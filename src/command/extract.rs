use freecrime::resources::parsers;
use freecrime::resources::types::style::{FaceType, Style, vehicle_type_const};
use freecrime::resources::types::graphics::IndexedImage;
use std::fs;
use std::path::Path;
use serde::Serialize;

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

            // 1. Blocks
            println!("Extracting blocks...");
            let blocks_dir = Path::new(out).join("blocks");
            fs::create_dir_all(&blocks_dir)?;

            for i in 0..style.side_count {
                let rgba = style.get_face_rgba(i, FaceType::Side, 0);
                save_png(&blocks_dir.join(format!("side_{:03}.png", i)), 64, 64, &rgba)?;
            }
            for i in 0..style.lid_count {
                for r in 0..4 {
                    let rgba = style.get_face_rgba(i, FaceType::Lid, r);
                    save_png(&blocks_dir.join(format!("lid_{:03}_remap_{}.png", i, r)), 64, 64, &rgba)?;
                }
            }
            for i in 0..style.aux_count {
                let rgba = style.get_face_rgba(i, FaceType::Aux, 0);
                save_png(&blocks_dir.join(format!("aux_{:03}.png", i)), 64, 64, &rgba)?;
            }

            // 2. Sprites
            println!("Extracting sprites...");
            let sprites_dir = Path::new(out).join("sprites");
            fs::create_dir_all(&sprites_dir)?;
            for (i, spr) in style.sprites.iter().enumerate() {
                if spr.width == 0 || spr.height == 0 { continue; }

                let palette = if !style.cluts.is_empty() {
                    let pal_idx_in_map = spr.clut as usize + style.tile_cl_count;
                    let clut_idx = style.palette_index.get(pal_idx_in_map).cloned().unwrap_or(0);
                    style.cluts.get(clut_idx as usize).unwrap_or(&style.palette)
                } else {
                    &style.palette
                };

                let rgba = IndexedImage::new(spr.width as u32, spr.height as u32, spr.pixels.clone()).to_rgba(palette);
                save_png(&sprites_dir.join(format!("sprite_{:04}.png", i)), spr.width as u32, spr.height as u32, &rgba)?;

                for (j, _delta) in spr.deltas.iter().enumerate() {
                    let d_pixels = spr.apply_delta(j);
                    let d_rgba = IndexedImage::new(spr.width as u32, spr.height as u32, d_pixels).to_rgba(palette);
                    save_png(&sprites_dir.join(format!("sprite_{:04}_delta_{:03}.png", i, j)), spr.width as u32, spr.height as u32, &d_rgba)?;
                }
            }

            // 3. Car Remaps
            println!("Extracting car remaps...");
            let car_remaps_dir = Path::new(out).join("car_remaps");
            fs::create_dir_all(&car_remaps_dir)?;
            let sprite_offsets = style.get_sprite_offsets();

            for (c, car) in style.cars.iter().enumerate() {
                let base_sprite_key = vehicle_type_const(car.vtype);
                if let Some(&base_offset) = sprite_offsets.get(base_sprite_key) {
                    let sprite_num = car.spr_num as usize + base_offset;
                    if let Some(spr) = style.sprites.get(sprite_num) {
                        if spr.width == 0 || spr.height == 0 { continue; }

                        // Base palette
                        let base_palette = if !style.cluts.is_empty() {
                            let pal_idx_in_map = spr.clut as usize + style.tile_cl_count;
                            let clut_idx = style.palette_index.get(pal_idx_in_map).cloned().unwrap_or(0);
                            style.cluts.get(clut_idx as usize).cloned().unwrap_or(style.palette.clone())
                        } else {
                            style.palette.clone()
                        };

                        for r_idx in 0..12 {
                            let remap_palette = if !style.cluts.is_empty() {
                                let offsets = car.remap24[r_idx];
                                if offsets == [0, 0, 0] { continue; }
                                base_palette.apply_hls_offset(offsets[0], offsets[1], offsets[2])
                            } else {
                                let remap_val = car.remap8[r_idx];
                                if remap_val == 0 { continue; }
                                let table = style.remap_tables.get(remap_val as usize).unwrap_or(&[0u8; 256]);
                                // Build a remapped palette
                                let mut colors = [[0u8; 3]; 256];
                                for i in 0..256 {
                                    colors[i] = style.palette.colors[table[i] as usize];
                                }
                                freecrime::resources::types::graphics::Palette { colors }
                            };

                            let rgba = IndexedImage::new(spr.width as u32, spr.height as u32, spr.pixels.clone()).to_rgba(&remap_palette);
                            save_png(&car_remaps_dir.join(format!("car_{:03}_remap_{:02}.png", c, r_idx)), spr.width as u32, spr.height as u32, &rgba)?;
                        }
                    }
                }
            }

            // 4. Metadata (JSON)
            println!("Saving metadata...");
            let json = serde_json::to_string_pretty(&style_metadata(&style))?;
            fs::write(Path::new(out).join("style.json"), json)?;

            println!("Extracted all data from {} to {}", path, out);
        }
        "FON" => {
            let font = parsers::fon::parse_fon(&data)?;
            fs::create_dir_all(out)?;
            for (i, glyph) in font.glyphs.iter().enumerate() {
                let rgba = glyph.to_rgba(&font.palette);
                save_png(&Path::new(out).join(format!("glyph_{:03}.png", i)), glyph.width, glyph.height, &rgba)?;
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

fn save_png(path: &std::path::Path, w: u32, h: u32, rgba: &[u8]) -> anyhow::Result<()> {
    if let Some(img) = image::RgbaImage::from_raw(w, h, rgba.to_vec()) {
        img.save(path)?;
    }
    Ok(())
}

use freecrime::resources::types::style::{Animation, ObjectInfo, CarInfo, SpriteNumbers};

#[derive(Serialize)]
struct StyleMetadata {
    side_count: usize,
    lid_count: usize,
    aux_count: usize,
    animations: Vec<Animation>,
    objects: Vec<ObjectInfo>,
    cars: Vec<CarInfo>,
    sprite_numbers: SpriteNumbers,
}

fn style_metadata(style: &Style) -> StyleMetadata {
    StyleMetadata {
        side_count: style.side_count,
        lid_count: style.lid_count,
        aux_count: style.aux_count,
        animations: style.animations.clone(),
        objects: style.objects.clone(),
        cars: style.cars.clone(),
        sprite_numbers: style.sprite_numbers.clone(),
    }
}
