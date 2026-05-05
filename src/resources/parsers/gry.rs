use crate::resources::types::style::*;
use crate::resources::types::graphics::{IndexedImage, Palette};
use crate::resources::{Result, Error};
use std::io::{Cursor, Seek, SeekFrom, Read};
use binrw::{BinRead, BinReaderExt};

#[derive(BinRead)]
#[br(little)]
#[allow(dead_code)]
enum Header {
    #[br(magic = 290u32)]
    Grx {
        side_size: u32, lid_size: u32, aux_size: u32, anim_size: u32,
        palette_size: u32, remap_size: u32, remap_index_size: u32,
        object_info_size: u32, car_size: u32, sprite_info_size: u32,
        sprite_graphics_size: u32, sprite_numbers_size: u32,
    },
    #[br(magic = 325u32)]
    Gry {
        side_size: u32, lid_size: u32, aux_size: u32, anim_size: u32,
        palette_size: u32, remap_size: u32, remap_index_size: u32,
        object_info_size: u32, car_size: u32, sprite_info_size: u32,
        sprite_graphics_size: u32, sprite_numbers_size: u32,
    },
    #[br(magic = 336u32)]
    G24 {
        side_size: u32, lid_size: u32, aux_size: u32, anim_size: u32,
        clut_size: u32, tileclut_size: u32, spriteclut_size: u32,
        newcarclut_size: u32, fontclut_size: u32, palette_index_size: u32,
        object_info_size: u32, car_size: u32, sprite_info_size: u32,
        sprite_graphics_size: u32, sprite_numbers_size: u32,
    },
}

impl Header {
    fn side_size(&self) -> u32 { match self { Header::Grx { side_size, .. } | Header::Gry { side_size, .. } | Header::G24 { side_size, .. } => *side_size } }
    fn lid_size(&self) -> u32 { match self { Header::Grx { lid_size, .. } | Header::Gry { lid_size, .. } | Header::G24 { lid_size, .. } => *lid_size } }
    fn aux_size(&self) -> u32 { match self { Header::Grx { aux_size, .. } | Header::Gry { aux_size, .. } | Header::G24 { aux_size, .. } => *aux_size } }
    fn anim_size(&self) -> u32 { match self { Header::Grx { anim_size, .. } | Header::Gry { anim_size, .. } | Header::G24 { anim_size, .. } => *anim_size } }
    fn palette_size(&self) -> u32 { match self { Header::Grx { palette_size, .. } | Header::Gry { palette_size, .. } => *palette_size, Header::G24 { clut_size, .. } => *clut_size } }
    fn remap_size(&self) -> u32 { match self { Header::Grx { remap_size, .. } | Header::Gry { remap_size, .. } => *remap_size, Header::G24 { clut_size, .. } => *clut_size } }
    fn remap_index_size(&self) -> u32 { match self { Header::Grx { remap_index_size, .. } | Header::Gry { remap_index_size, .. } => *remap_index_size, Header::G24 { palette_index_size, .. } => *palette_index_size } }
    fn object_info_size(&self) -> u32 { match self { Header::Grx { object_info_size, .. } | Header::Gry { object_info_size, .. } | Header::G24 { object_info_size, .. } => *object_info_size } }
    fn car_size(&self) -> u32 { match self { Header::Grx { car_size, .. } | Header::Gry { car_size, .. } | Header::G24 { car_size, .. } => *car_size } }
    fn sprite_info_size(&self) -> u32 { match self { Header::Grx { sprite_info_size, .. } | Header::Gry { sprite_info_size, .. } | Header::G24 { sprite_info_size, .. } => *sprite_info_size } }
    fn sprite_graphics_size(&self) -> u32 { match self { Header::Grx { sprite_graphics_size, .. } | Header::Gry { sprite_graphics_size, .. } | Header::G24 { sprite_graphics_size, .. } => *sprite_graphics_size } }
    fn sprite_numbers_size(&self) -> u32 { match self { Header::Grx { sprite_numbers_size, .. } | Header::Gry { sprite_numbers_size, .. } | Header::G24 { sprite_numbers_size, .. } => *sprite_numbers_size } }
    fn is_g24(&self) -> bool { matches!(self, Header::G24 { .. }) }
}

#[derive(BinRead)]
#[br(little)]
#[allow(dead_code)]
struct ObjectInfoRaw {
    width: i32,
    height: i32,
    depth: i32,
    spr_num: u16,
    weight: u16,
    aux: u16,
    status: i8,
    num_into: u8,
    #[br(count = num_into)]
    into: Vec<u16>,
}

#[derive(BinRead)]
#[br(little)]
struct SpriteNumbersRaw {
    arrow: u16, digits: u16, boat: u16, box_obj: u16, bus: u16, car: u16,
    object: u16, ped: u16, speedo: u16, tank: u16, traffic_lights: u16,
    train: u16, trdoors: u16, bike: u16, tram: u16, wbus: u16, wcar: u16,
    ex: u16, tumcar: u16, tumtruck: u16, ferry: u16,
}

pub fn parse_gry(data: &[u8]) -> Result<Style> {
    let mut cursor = Cursor::new(data);
    let header: Header = cursor.read_le()?;
    let is_g24 = header.is_g24();

    // 1. Block Data
    let total_block_size = header.side_size() + header.lid_size() + header.aux_size();
    let block_count = total_block_size / 4096;
    let actual_block_data_size = if is_g24 {
        let rem = block_count % 4;
        let padding = if rem == 0 { 0 } else { (4 - rem) * 4096 };
        total_block_size + padding
    } else {
        total_block_size
    };

    let mut block_data = vec![0u8; actual_block_data_size as usize];
    cursor.read_exact(&mut block_data)?;

    let mut blocks = Vec::with_capacity(block_count as usize);
    let rows = (block_count + 3) / 4;
    for row in 0..rows {
        for col in 0..4 {
            let block_idx = row * 4 + col;
            if block_idx >= block_count { break; }
            let mut pixels = vec![0u8; 64 * 64];
            for y in 0..64 {
                let src_offset = (row * 64 + y) as usize * 256 + col as usize * 64;
                pixels[y as usize * 64..(y as usize + 1) * 64].copy_from_slice(&block_data[src_offset..src_offset + 64]);
            }
            blocks.push(IndexedImage::new(64, 64, pixels));
        }
    }

    let side_count = (header.side_size() / 4096) as usize;
    let lid_count = (header.lid_size() / 4096) as usize;
    let aux_count = (header.aux_size() / 4096) as usize;

    // 2. Animations
    let mut animations = Vec::new();
    if header.anim_size() > 0 {
        let anim_data_slice = &data[cursor.position() as usize..cursor.position() as usize + header.anim_size() as usize];
        cursor.seek(SeekFrom::Current(header.anim_size() as i64))?;
        let mut anim_cursor = Cursor::new(anim_data_slice);
        let num_anims: u8 = anim_cursor.read_le()?;
        for _ in 0..num_anims {
            let block: u8 = anim_cursor.read_le()?;
            let which: u8 = anim_cursor.read_le()?;
            let speed: u8 = anim_cursor.read_le()?;
            let frame_count: u8 = anim_cursor.read_le()?;
            let mut frames = Vec::with_capacity(frame_count as usize);
            for _ in 0..frame_count { frames.push(anim_cursor.read_le::<u8>()?); }
            animations.push(Animation { block, which, speed, frames });
        }
    }

    // 3. Palette / CLUT
    let palette = if is_g24 {
        let mut paged_size = header.palette_size();
        if paged_size % 65536 != 0 { paged_size += 65536 - (paged_size % 65536); }
        let mut clut_data = vec![0u8; paged_size as usize];
        cursor.read_exact(&mut clut_data)?;
        let mut colors = [[0u8; 3]; 256];
        if !clut_data.is_empty() {
            for i in 0..256 {
                colors[i][2] = clut_data[i * 4];
                colors[i][1] = clut_data[i * 4 + 1];
                colors[i][0] = clut_data[i * 4 + 2];
            }
        }
        Palette { colors }
    } else {
        let mut palette_data = vec![0u8; header.palette_size() as usize];
        cursor.read_exact(&mut palette_data)?;
        let mut colors = [[0u8; 3]; 256];
        for i in 0..256 {
            if i * 3 + 2 < palette_data.len() {
                colors[i][0] = (palette_data[i * 3] << 2) | (palette_data[i * 3] >> 4);
                colors[i][1] = (palette_data[i * 3 + 1] << 2) | (palette_data[i * 3 + 1] >> 4);
                colors[i][2] = (palette_data[i * 3 + 2] << 2) | (palette_data[i * 3 + 2] >> 4);
            }
        }
        Palette { colors }
    };

    // 4. Remap Tables
    let mut remap_tables = Vec::new();
    if !is_g24 {
        let num_remaps = header.remap_size() / 256;
        for _ in 0..num_remaps {
            let mut table = [0u8; 256];
            cursor.read_exact(&mut table)?;
            remap_tables.push(table);
        }
    } else {
        let mut table = [0u8; 256];
        for i in 0..256 { table[i] = i as u8; }
        remap_tables.push(table);
    }

    // 5. Remap Index
    let mut remap_indices = Vec::new();
    if is_g24 {
        for _ in 0..header.remap_size() / 2 {
            let _: u16 = cursor.read_le()?;
            remap_indices.push([0, 0, 0, 0]);
        }
    } else {
        for _ in 0..header.remap_index_size() / 4 {
            let mut idx = [0u8; 4];
            cursor.read_exact(&mut idx)?;
            remap_indices.push(idx);
        }
    }

    // 6. Object Info
    let mut objects = Vec::new();
    let object_info_end = cursor.position() + header.object_info_size() as u64;
    while cursor.position() < object_info_end {
        let b: ObjectInfoRaw = cursor.read_le()?;
        objects.push(ObjectInfo {
            width: b.width as u16, height: b.height as u16, depth: b.depth as u16,
            spr_num: b.spr_num, weight: b.weight, aux: b.aux, status: b.status, into: b.into
        });
    }

    // 7. Car Info
    let mut cars = Vec::new();
    let car_info_end = cursor.position() + header.car_size() as u64;
    while cursor.position() < car_info_end {
        let width: i16 = cursor.read_le()?;
        let height: i16 = cursor.read_le()?;
        let depth: i16 = cursor.read_le()?;
        let spr_num: u16 = cursor.read_le()?;
        let weight: u16 = cursor.read_le()?;
        let max_speed: i16 = cursor.read_le()?;
        let min_speed: i16 = cursor.read_le()?;
        let acceleration: i16 = cursor.read_le()?;
        let braking: i16 = cursor.read_le()?;
        let grip: i16 = cursor.read_le()?;
        let handling: i16 = cursor.read_le()?;
        cursor.seek(SeekFrom::Current(12 * 6))?;
        let mut remaps = [0u8; 12];
        cursor.read_exact(&mut remaps)?;
        let vtype: u8 = cursor.read_le()?;
        let model: u8 = cursor.read_le()?;
        let turning: u8 = cursor.read_le()?;
        let damageable: u8 = cursor.read_le()?;
        let mut value = [0u16; 4];
        for v in &mut value { *v = cursor.read_le()?; }
        let cx: i8 = cursor.read_le()?;
        let cy: i8 = cursor.read_le()?;
        let moment: i32 = cursor.read_le()?;
        let mass = read_f32_fix(&mut cursor)?;
        let thrust = read_f32_fix(&mut cursor)?;
        let adhesion_x = read_f32_fix(&mut cursor)?;
        let adhesion_y = read_f32_fix(&mut cursor)?;
        let handbrake_friction = read_f32_fix(&mut cursor)?;
        let footbrake_friction = read_f32_fix(&mut cursor)?;
        let brake_bias = read_f32_fix(&mut cursor)?;
        let turn_ratio: i16 = cursor.read_le()?;
        let drive_wheel_offset: i16 = cursor.read_le()?;
        let steering_wheel_offset: i16 = cursor.read_le()?;
        let slide_value = read_f32_fix(&mut cursor)?;
        let hb_slide_value = read_f32_fix(&mut cursor)?;
        let convertible: u8 = cursor.read_le()?;
        let engine: u8 = cursor.read_le()?;
        let radio: u8 = cursor.read_le()?;
        let horn: u8 = cursor.read_le()?;
        let sound_func: u8 = cursor.read_le()?;
        let fast_change: u8 = cursor.read_le()?;
        let num_doors: i16 = cursor.read_le()?;
        let mut doors = Vec::with_capacity(num_doors as usize);
        for _ in 0..num_doors {
            doors.push(DoorInfo {
                rpx: cursor.read_le()?, rpy: cursor.read_le()?,
                object: cursor.read_le()?, delta: cursor.read_le()?
            });
        }
        cars.push(CarInfo { width: width as u16, height: height as u16, depth: depth as u16, spr_num, weight, max_speed, min_speed, acceleration, braking, grip, handling, remaps, vtype, model, turning, damageable, value, cx, cy, moment, mass, thrust, adhesion_x, adhesion_y, handbrake_friction, footbrake_friction, brake_bias, turn_ratio, drive_wheel_offset, steering_wheel_offset, slide_value, hb_slide_value, convertible, engine, radio, horn, sound_func, fast_change, doors });
    }

    // 8. Sprite Info & 9. Sprite Graphics
    let sprite_info_start = cursor.position();
    let sprite_graphics_start = sprite_info_start + header.sprite_info_size() as u64;
    cursor.seek(SeekFrom::Start(sprite_graphics_start))?;
    let _sprite_graphics_data = vec![0u8; header.sprite_graphics_size() as usize];

    cursor.seek(SeekFrom::Start(sprite_info_start))?;
    let mut sprites = Vec::new();
    let mut info_consumed = 0u64;
    while info_consumed < header.sprite_info_size() as u64 {
        let w: u8 = cursor.read_le()?;
        let h: u8 = cursor.read_le()?;
        let dc: u8 = cursor.read_le()?;
        let _v: u8 = cursor.read_le()?;
        if is_g24 {
            let _: u16 = cursor.read_le()?;
            let _: u16 = cursor.read_le()?;
            let _: u8 = cursor.read_le()?;
            let _: u8 = cursor.read_le()?;
            let _: u16 = cursor.read_le()?;
            info_consumed += 12;
            for _ in 0..dc { let _: u32 = cursor.read_le()?; let _: u16 = cursor.read_le()?; info_consumed += 6; }
        } else {
            let _: u16 = cursor.read_le()?;
            let _: u32 = cursor.read_le()?;
            info_consumed += 10;
            for _ in 0..dc { let _: u32 = cursor.read_le()?; let _: u16 = cursor.read_le()?; info_consumed += 6; }
        }
        sprites.push(Sprite { width: w, height: h, image: vec![], deltas: vec![] });
    }

    cursor.seek(SeekFrom::Start(sprite_graphics_start + header.sprite_graphics_size() as u64))?;

    // 10. Sprite Numbers
    let mut sprite_numbers = SpriteNumbers::default();
    if header.sprite_numbers_size() >= 42 {
        let sn: SpriteNumbersRaw = cursor.read_le()?;
        sprite_numbers = SpriteNumbers {
            arrow: sn.arrow, digits: sn.digits, boat: sn.boat, box_obj: sn.box_obj,
            bus: sn.bus, car: sn.car, object: sn.object, ped: sn.ped, speedo: sn.speedo,
            tank: sn.tank, traffic_lights: sn.traffic_lights, train: sn.train,
            trdoors: sn.trdoors, bike: sn.bike, tram: sn.tram, wbus: sn.wbus,
            wcar: sn.wcar, ex: sn.ex, tumcar: sn.tumcar, tumtruck: sn.tumtruck, ferry: sn.ferry,
        };
    }

    if cursor.position() < data.len() as u64 {
         return Err(Error::Parse(format!("GRY file has {} trailing bytes", data.len() as u64 - cursor.position())));
    }

    Ok(Style { blocks, side_count, lid_count, aux_count, animations, palette, remap_tables, remap_indices, objects, cars, sprites, sprite_numbers })
}

fn read_f32_fix(r: &mut (impl Read + Seek)) -> Result<f32> {
    let v: i32 = r.read_le()?;
    Ok(v as f32 / 65536.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gry_header() {
        let mut data = Vec::new();
        data.extend_from_slice(&325u32.to_le_bytes()); // version
        for _ in 0..12 { data.extend_from_slice(&0u32.to_le_bytes()); } // 12 sizes
        let style = parse_gry(&data).unwrap();
        assert_eq!(style.blocks.len(), 0);
    }

    #[test]
    fn test_parse_g24_header() {
        let mut data = Vec::new();
        data.extend_from_slice(&336u32.to_le_bytes()); // version
        for _ in 0..15 { data.extend_from_slice(&0u32.to_le_bytes()); } // 15 sizes
        let style = parse_gry(&data).unwrap();
        assert_eq!(style.blocks.len(), 0);
    }

    #[test]
    fn test_gry_palette_scaling() {
        let mut data = Vec::new();
        data.extend_from_slice(&325u32.to_le_bytes());
        for _ in 0..3 { data.extend_from_slice(&0u32.to_le_bytes()); } // block sizes
        data.extend_from_slice(&0u32.to_le_bytes()); // anim size
        data.extend_from_slice(&768u32.to_le_bytes()); // palette size
        for _ in 0..7 { data.extend_from_slice(&0u32.to_le_bytes()); } // other sizes
        let mut palette = vec![0u8; 768];
        palette[0] = 63; // Max GTA1 color (6-bit)
        data.extend(palette);
        let style = parse_gry(&data).unwrap();
        assert_eq!(style.palette.colors[0][0], 255); // Scaled to 8-bit
    }
}
