use crate::resources::types::style::*;
use crate::resources::types::graphics::{IndexedImage, Palette};
use crate::resources::{Result, Error};
use std::io::{Read, Cursor, Seek, SeekFrom};

pub fn parse_gry(data: &[u8]) -> Result<Style> {
    let mut cursor = Cursor::new(data);
    
    let version = read_u32(&mut cursor)?;
    if version != 325 && version != 290 && version != 336 {
        return Err(Error::Parse(format!("Unsupported Style version: {}", version)));
    }
    
    let is_g24 = version == 336;
    
    let side_size = read_u32(&mut cursor)?;
    let lid_size = read_u32(&mut cursor)?;
    let aux_size = read_u32(&mut cursor)?;
    let anim_size = read_u32(&mut cursor)?;
    
    // Header differences
    let (palette_size, remap_size, remap_index_size, object_info_size, car_size, sprite_info_size, sprite_graphics_size, sprite_numbers_size) = if is_g24 {
        let clut_size = read_u32(&mut cursor)?;
        let _tileclut_size = read_u32(&mut cursor)?;
        let _spriteclut_size = read_u32(&mut cursor)?;
        let _newcarclut_size = read_u32(&mut cursor)?;
        let _fontclut_size = read_u32(&mut cursor)?;
        let palette_index_size = read_u32(&mut cursor)?;
        let object_info_size = read_u32(&mut cursor)?;
        let car_size = read_u32(&mut cursor)?;
        let sprite_info_size = read_u32(&mut cursor)?;
        let sprite_graphics_size = read_u32(&mut cursor)?;
        let sprite_numbers_size = read_u32(&mut cursor)?;
        (clut_size, palette_index_size, 0, object_info_size, car_size, sprite_info_size, sprite_graphics_size, sprite_numbers_size)
    } else {
        let p_size = read_u32(&mut cursor)?;
        let r_size = read_u32(&mut cursor)?;
        let ri_size = read_u32(&mut cursor)?;
        let o_size = read_u32(&mut cursor)?;
        let c_size = read_u32(&mut cursor)?;
        let si_size = read_u32(&mut cursor)?;
        let sg_size = read_u32(&mut cursor)?;
        let sn_size = read_u32(&mut cursor)?;
        (p_size, r_size, ri_size, o_size, c_size, si_size, sg_size, sn_size)
    };
    
    // 1. Block Data
    let total_block_size = side_size + lid_size + aux_size;
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
    
    let side_count = (side_size / 4096) as usize;
    let lid_count = (lid_size / 4096) as usize;
    let aux_count = (aux_size / 4096) as usize;
    
    // 2. Animations
    let mut animations = Vec::new();
    if anim_size > 0 {
        let mut anim_cursor = Cursor::new(&data[cursor.position() as usize..cursor.position() as usize + anim_size as usize]);
        cursor.seek(SeekFrom::Current(anim_size as i64))?;
        let num_anims = read_u8(&mut anim_cursor)?;
        for _ in 0..num_anims {
            let block = read_u8(&mut anim_cursor)?;
            let which = read_u8(&mut anim_cursor)?;
            let speed = read_u8(&mut anim_cursor)?;
            let frame_count = read_u8(&mut anim_cursor)?;
            let mut frames = Vec::with_capacity(frame_count as usize);
            for _ in 0..frame_count { frames.push(read_u8(&mut anim_cursor)?); }
            animations.push(Animation { block, which, speed, frames });
        }
    }
    
    // 3. Palette / CLUT
    let palette = if is_g24 {
        let mut paged_size = palette_size;
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
        let mut palette_data = vec![0u8; palette_size as usize];
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
        let num_remaps = remap_size / 256;
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
        for _ in 0..remap_size / 2 {
            let _val = read_u16(&mut cursor)?;
            remap_indices.push([0, 0, 0, 0]);
        }
    } else {
        for _ in 0..remap_index_size / 4 {
            let mut idx = [0u8; 4];
            cursor.read_exact(&mut idx)?;
            remap_indices.push(idx);
        }
    }
    
    // 6. Object Info
    let mut objects = Vec::new();
    let object_info_end = cursor.position() + object_info_size as u64;
    while cursor.position() < object_info_end {
        let width = read_i32(&mut cursor)? as u16;
        let height = read_i32(&mut cursor)? as u16;
        let depth = read_i32(&mut cursor)? as u16;
        let spr_num = read_u16(&mut cursor)?;
        let _weight = read_u16(&mut cursor)?;
        let _aux = read_u16(&mut cursor)?;
        let status = read_i8(&mut cursor)?;
        let num_into = read_u8(&mut cursor)?;
        let mut into = Vec::with_capacity(num_into as usize);
        for _ in 0..num_into { into.push(read_u16(&mut cursor)?); }
        objects.push(ObjectInfo { width, height, depth, spr_num, weight: _weight, aux: _aux, status, into });
    }
    
    // 7. Car Info
    let mut cars = Vec::new();
    let car_info_end = cursor.position() + car_size as u64;
    while cursor.position() < car_info_end {
        let width = read_i16(&mut cursor)? as u16;
        let height = read_i16(&mut cursor)? as u16;
        let depth = read_i16(&mut cursor)? as u16;
        let spr_num = read_u16(&mut cursor)?;
        let weight = read_u16(&mut cursor)?;
        let max_speed = read_i16(&mut cursor)?;
        let min_speed = read_i16(&mut cursor)?;
        let acceleration = read_i16(&mut cursor)?;
        let braking = read_i16(&mut cursor)?;
        let grip = read_i16(&mut cursor)?;
        let handling = read_i16(&mut cursor)?;
        cursor.seek(SeekFrom::Current(12 * 6))?;
        let mut remaps = [0u8; 12];
        cursor.read_exact(&mut remaps)?;
        let vtype = read_u8(&mut cursor)?;
        let model = read_u8(&mut cursor)?;
        let turning = read_u8(&mut cursor)?;
        let damageable = read_u8(&mut cursor)?;
        let mut value = [0u16; 4];
        for v in &mut value { *v = read_u16(&mut cursor)?; }
        let cx = read_i8(&mut cursor)?;
        let cy = read_i8(&mut cursor)?;
        let moment = read_i32(&mut cursor)?;
        let mass = read_f32_fix(&mut cursor)?;
        let thrust = read_f32_fix(&mut cursor)?;
        let adhesion_x = read_f32_fix(&mut cursor)?;
        let adhesion_y = read_f32_fix(&mut cursor)?;
        let handbrake_friction = read_f32_fix(&mut cursor)?;
        let footbrake_friction = read_f32_fix(&mut cursor)?;
        let brake_bias = read_f32_fix(&mut cursor)?;
        let turn_ratio = read_i16(&mut cursor)?;
        let drive_wheel_offset = read_i16(&mut cursor)?;
        let steering_wheel_offset = read_i16(&mut cursor)?;
        let slide_value = read_f32_fix(&mut cursor)?;
        let hb_slide_value = read_f32_fix(&mut cursor)?;
        let convertible = read_u8(&mut cursor)?;
        let engine = read_u8(&mut cursor)?;
        let radio = read_u8(&mut cursor)?;
        let horn = read_u8(&mut cursor)?;
        let sound_func = read_u8(&mut cursor)?;
        let fast_change = read_u8(&mut cursor)?;
        let num_doors = read_i16(&mut cursor)?;
        let mut doors = Vec::with_capacity(num_doors as usize);
        for _ in 0..num_doors {
            doors.push(DoorInfo { rpx: read_i16(&mut cursor)?, rpy: read_i16(&mut cursor)?, object: read_i16(&mut cursor)?, delta: read_i16(&mut cursor)? });
        }
        cars.push(CarInfo { width, height, depth, spr_num, weight, max_speed, min_speed, acceleration, braking, grip, handling, remaps, vtype, model, turning, damageable, value, cx, cy, moment, mass, thrust, adhesion_x, adhesion_y, handbrake_friction, footbrake_friction, brake_bias, turn_ratio, drive_wheel_offset, steering_wheel_offset, slide_value, hb_slide_value, convertible, engine, radio, horn, sound_func, fast_change, doors });
    }
    
    // 8. Sprite Info & 9. Sprite Graphics
    let sprite_info_start = cursor.position();
    let sprite_graphics_start = sprite_info_start + sprite_info_size as u64;
    cursor.seek(SeekFrom::Start(sprite_graphics_start))?;
    let mut sprite_graphics_data = vec![0u8; sprite_graphics_size as usize];
    cursor.read_exact(&mut sprite_graphics_data)?;
    
    cursor.seek(SeekFrom::Start(sprite_info_start))?;
    let mut sprites = Vec::new();
    let mut info_consumed = 0u64;
    while info_consumed < sprite_info_size as u64 {
        let w = read_u8(&mut cursor)?;
        let h = read_u8(&mut cursor)?;
        let dc = read_u8(&mut cursor)?;
        let _v = read_u8(&mut cursor)?;
        let _size: u64;
        let _ptr: u64;
        if is_g24 {
            let _sz = read_u16(&mut cursor)?;
            let _clut = read_u16(&mut cursor)?;
            let _xoff = read_u8(&mut cursor)?;
            let _yoff = read_u8(&mut cursor)?;
            let _page = read_u16(&mut cursor)?;
            _size = _sz as u64; // Placeholder
            _ptr = 0; // Placeholder
            info_consumed += 12;
            for _ in 0..dc { read_u32(&mut cursor)?; read_u16(&mut cursor)?; info_consumed += 6; }
        } else {
            let _sz = read_u16(&mut cursor)?;
            let _p = read_u32(&mut cursor)?;
            _size = _sz as u64;
            _ptr = _p as u64;
            info_consumed += 10;
            for _ in 0..dc { read_u32(&mut cursor)?; read_u16(&mut cursor)?; info_consumed += 6; }
        }
        // Simplified sprite loading
        sprites.push(Sprite { width: w, height: h, image: vec![], deltas: vec![] });
    }
    
    cursor.seek(SeekFrom::Start(sprite_graphics_start + sprite_graphics_size as u64))?;
    
    // 10. Sprite Numbers
    let mut sprite_numbers = SpriteNumbers::default();
    if sprite_numbers_size >= 42 {
        sprite_numbers.arrow = read_u16(&mut cursor)?;
        sprite_numbers.digits = read_u16(&mut cursor)?;
        sprite_numbers.boat = read_u16(&mut cursor)?;
        sprite_numbers.box_obj = read_u16(&mut cursor)?;
        sprite_numbers.bus = read_u16(&mut cursor)?;
        sprite_numbers.car = read_u16(&mut cursor)?;
        sprite_numbers.object = read_u16(&mut cursor)?;
        sprite_numbers.ped = read_u16(&mut cursor)?;
        sprite_numbers.speedo = read_u16(&mut cursor)?;
        sprite_numbers.tank = read_u16(&mut cursor)?;
        sprite_numbers.traffic_lights = read_u16(&mut cursor)?;
        sprite_numbers.train = read_u16(&mut cursor)?;
        sprite_numbers.trdoors = read_u16(&mut cursor)?;
        sprite_numbers.bike = read_u16(&mut cursor)?;
        sprite_numbers.tram = read_u16(&mut cursor)?;
        sprite_numbers.wbus = read_u16(&mut cursor)?;
        sprite_numbers.wcar = read_u16(&mut cursor)?;
        sprite_numbers.ex = read_u16(&mut cursor)?;
        sprite_numbers.tumcar = read_u16(&mut cursor)?;
        sprite_numbers.tumtruck = read_u16(&mut cursor)?;
        sprite_numbers.ferry = read_u16(&mut cursor)?;
    }
    
    if cursor.position() < data.len() as u64 {
         return Err(Error::Parse(format!("GRY file has {} trailing bytes", data.len() as u64 - cursor.position())));
    }

    Ok(Style { blocks, side_count, lid_count, aux_count, animations, palette, remap_tables, remap_indices, objects, cars, sprites, sprite_numbers })
}

fn read_u8(r: &mut impl Read) -> Result<u8> { let mut b = [0u8; 1]; r.read_exact(&mut b)?; Ok(b[0]) }
fn read_i8(r: &mut impl Read) -> Result<i8> { Ok(read_u8(r)? as i8) }
fn read_u16(r: &mut impl Read) -> Result<u16> { let mut b = [0u8; 2]; r.read_exact(&mut b)?; Ok(u16::from_le_bytes(b)) }
fn read_i16(r: &mut impl Read) -> Result<i16> { let mut b = [0u8; 2]; r.read_exact(&mut b)?; Ok(i16::from_le_bytes(b)) }
fn read_u32(r: &mut impl Read) -> Result<u32> { let mut b = [0u8; 4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn read_i32(r: &mut impl Read) -> Result<i32> { let mut b = [0u8; 4]; r.read_exact(&mut b)?; Ok(i32::from_le_bytes(b)) }
fn read_f32_fix(r: &mut impl Read) -> Result<f32> { let v = read_i32(r)?; Ok(v as f32 / 65536.0) }

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
