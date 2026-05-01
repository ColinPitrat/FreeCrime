use crate::resources::types::map::*;
use crate::resources::{Result, Error};
use std::io::{Read, Cursor};

pub fn parse_cmp(data: &[u8]) -> Result<Map> {
    let mut cursor = Cursor::new(data);
    
    // Header
    let version = read_u32(&mut cursor)?;
    if version != 331 {
        return Err(Error::Parse(format!("Unsupported CMP version: {}", version)));
    }
    
    let style_index = read_u8(&mut cursor)?;
    let sample_index = read_u8(&mut cursor)?;
    let _reserved = read_u16(&mut cursor)?;
    
    let route_size = read_u32(&mut cursor)?;
    let object_pos_size = read_u32(&mut cursor)?;
    let column_size = read_u32(&mut cursor)?;
    let block_size = read_u32(&mut cursor)?;
    let nav_data_size = read_u32(&mut cursor)?;
    
    // Base (Offsets into column data)
    let mut base_offsets = Vec::with_capacity(256 * 256);
    for _ in 0..256 * 256 {
        base_offsets.push(read_u32(&mut cursor)?);
    }
    
    // Column Data
    let mut column_data = vec![0u8; column_size as usize];
    cursor.read_exact(&mut column_data)?;
    
    // Block Types
    let num_block_types = block_size / 8;
    let mut block_types = Vec::with_capacity(num_block_types as usize);
    for _ in 0..num_block_types {
        let type_map = read_u16(&mut cursor)?;
        let type_map_ext = read_u8(&mut cursor)?;
        let left = read_u8(&mut cursor)?;
        let right = read_u8(&mut cursor)?;
        let top = read_u8(&mut cursor)?;
        let bottom = read_u8(&mut cursor)?;
        let lid = read_u8(&mut cursor)?;
        block_types.push(BlockType {
            type_map,
            type_map_ext,
            left,
            right,
            top,
            bottom,
            lid,
        });
    }
    
    // Resolve Columns
    let mut grid = Vec::with_capacity(256 * 256);
    for &offset in &base_offsets {
        if offset as usize >= column_data.len() {
             return Err(Error::Parse(format!("Column offset out of bounds: {}", offset)));
        }
        let mut col_cursor = Cursor::new(&column_data[..]);
        col_cursor.set_position(offset as u64);
        
        let height = read_u16(&mut col_cursor)?;
        if height > 6 {
            return Err(Error::Parse(format!("Invalid column height: {}", height)));
        }
        
        let mut levels = [0u16; 6];
        let num_blocks = 6 - height as usize;
        for i in 0..num_blocks {
            levels[height as usize + i] = read_u16(&mut col_cursor)?;
        }
        
        grid.push(Column { levels });
    }
    
    // Object Pos
    let object_pos_end = cursor.position() + object_pos_size as u64;
    let mut objects = Vec::new();
    while cursor.position() < object_pos_end {
        objects.push(MapObject {
            x: read_u16(&mut cursor)?,
            y: read_u16(&mut cursor)?,
            z: read_u16(&mut cursor)?,
            obj_type: read_u8(&mut cursor)?,
            remap: read_u8(&mut cursor)?,
            rotation: read_u16(&mut cursor)?,
            pitch: read_u16(&mut cursor)?,
            roll: read_u16(&mut cursor)?,
        });
    }
    
    // Routes
    let route_end = cursor.position() + route_size as u64;
    let mut routes = Vec::new();
    while cursor.position() < route_end {
        let num_points = read_u8(&mut cursor)? as usize;
        let route_type = read_u8(&mut cursor)?;
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            points.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?));
        }
        routes.push(Route { route_type, points });
    }
    
    // Locations
    let mut police = Vec::new();
    for _ in 0..6 { police.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    let mut hospital = Vec::new();
    for _ in 0..6 { hospital.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    let mut unused1 = Vec::new();
    for _ in 0..6 { unused1.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    let mut unused2 = Vec::new();
    for _ in 0..6 { unused2.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    let mut fire = Vec::new();
    for _ in 0..6 { fire.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    let mut unused3 = Vec::new();
    for _ in 0..6 { unused3.push((read_u8(&mut cursor)?, read_u8(&mut cursor)?, read_u8(&mut cursor)?)); }
    
    let locations = MapLocations {
        police,
        hospital,
        fire,
        unused: vec![unused1, unused2, unused3],
    };
    
    // Nav Data
    let nav_end = cursor.position() + nav_data_size as u64;
    let mut nav_zones = Vec::new();
    while cursor.position() < nav_end {
        let x = read_u8(&mut cursor)?;
        let y = read_u8(&mut cursor)?;
        let width = read_u8(&mut cursor)?;
        let height = read_u8(&mut cursor)?;
        let sample = read_u8(&mut cursor)?;
        
        let mut name_buf = [0u8; 30];
        cursor.read_exact(&mut name_buf)?;
        let name = String::from_utf8_lossy(&name_buf)
            .trim_matches(char::from(0))
            .to_string();
            
        nav_zones.push(NavZone { x, y, width, height, sample, name });
    }
    
    // Strictness check
    if cursor.position() < data.len() as u64 {
        return Err(Error::Parse(format!("CMP file has {} trailing bytes", data.len() as u64 - cursor.position())));
    }

    Ok(Map {
        style_index,
        sample_index,
        grid,
        block_types,
        objects,
        routes,
        locations,
        nav_zones,
    })
}

// Helpers
fn read_u8(r: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16(r: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(r: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cmp_minimal() {
        let mut data = Vec::new();
        // Version
        data.extend_from_slice(&331u32.to_le_bytes());
        // style, sample, reserved
        data.push(1); data.push(2); data.extend_from_slice(&0u16.to_le_bytes());
        
        // sizes: route, obj, col, block, nav
        data.extend_from_slice(&0u32.to_le_bytes()); // route
        data.extend_from_slice(&0u32.to_le_bytes()); // obj
        data.extend_from_slice(&2u32.to_le_bytes()); // col (1 col with height 6)
        data.extend_from_slice(&8u32.to_le_bytes()); // block (1 block info)
        data.extend_from_slice(&0u32.to_le_bytes()); // nav
        
        // Base: 256*256 offsets
        for _ in 0..256*256 {
            data.extend_from_slice(&0u32.to_le_bytes());
        }
        
        // Column data
        data.extend_from_slice(&6u16.to_le_bytes()); // height 6 -> 0 blocks? 
        // Wait, if height=6, 6-6=0 blocks.
        
        // Block Types
        data.extend_from_slice(&0u16.to_le_bytes()); // tm
        data.push(0); // tme
        data.push(1); data.push(2); data.push(3); data.push(4); data.push(5); // faces
        
        // Locations: 108 bytes
        data.extend(vec![0u8; 108]);
        
        let map = parse_cmp(&data).unwrap();
        assert_eq!(map.style_index, 1);
        assert_eq!(map.grid.len(), 256*256);
        assert_eq!(map.block_types.len(), 1);
    }
}
