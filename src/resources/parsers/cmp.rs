use crate::resources::types::map::*;
use crate::resources::{Result, Error};
use std::io::{Cursor, Read};
use binrw::{BinRead, BinReaderExt};

#[derive(BinRead)]
#[br(little)]
#[allow(dead_code)]
struct CmpHeader {
    #[br(assert(version == 331, "Unsupported CMP version: {}", version))]
    version: u32,
    style_index: u8,
    sample_index: u8,
    _reserved: u16,
    route_size: u32,
    object_pos_size: u32,
    column_size: u32,
    block_size: u32,
    nav_data_size: u32,
}

#[derive(BinRead)]
#[br(little)]
struct BlockTypeRaw {
    type_map: u16,
    type_map_ext: u8,
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
    lid: u8,
}

#[derive(BinRead)]
#[br(little)]
struct MapObjectRaw {
    x: u16,
    y: u16,
    z: u16,
    obj_type: u8,
    remap: u8,
    rotation: u16,
    pitch: u16,
    roll: u16,
}

#[derive(BinRead)]
#[br(little)]
struct NavZoneRaw {
    x: u8,
    y: u8,
    width: u8,
    height: u8,
    sample: u8,
    #[br(map = |s: [u8; 30]| String::from_utf8_lossy(&s).trim_matches(char::from(0)).to_string())]
    name: String,
}

pub fn parse_cmp(data: &[u8]) -> Result<Map> {
    let mut cursor = Cursor::new(data);

    let header: CmpHeader = cursor.read_le()?;

    // Base (Offsets into column data)
    let mut base_offsets = Vec::with_capacity(256 * 256);
    for _ in 0..256 * 256 {
        base_offsets.push(cursor.read_le::<u32>()?);
    }

    // Column Data
    let mut column_data = vec![0u8; header.column_size as usize];
    cursor.read_exact(&mut column_data)?;

    // Block Types
    let num_block_types = header.block_size / 8;
    let mut block_types_raw: Vec<BlockTypeRaw> = Vec::with_capacity(num_block_types as usize);
    for _ in 0..num_block_types {
        block_types_raw.push(cursor.read_le()?);
    }

    let block_types: Vec<Block> = block_types_raw.into_iter().map(|b| Block {
        type_map: b.type_map,
        type_map_ext: b.type_map_ext,
        left: b.left,
        right: b.right,
        top: b.top,
        bottom: b.bottom,
        lid: b.lid,
    }).collect();

    // Initialize 3D Map
    let mut map = Map::new(256, 256, 6);
    map.style_index = header.style_index;
    map.sample_index = header.sample_index;

    // Resolve Columns into 3D Grid
    for (idx, &offset) in base_offsets.iter().enumerate() {
        if offset as usize >= column_data.len() {
             return Err(Error::Parse(format!("Column offset out of bounds: {}", offset)));
        }
        let x = idx % 256;
        let y = idx / 256;

        let mut col_cursor = Cursor::new(&column_data[..]);
        col_cursor.set_position(offset as u64);

        let height: u16 = col_cursor.read_le()?;
        if height > 6 {
            return Err(Error::Parse(format!("Invalid column height: {}", height)));
        }

        let num_blocks = 6 - height as usize;
        for i in 0..num_blocks {
            let bt_idx: u16 = col_cursor.read_le()?;
            if let Some(bt) = (bt_idx > 0).then(|| block_types.get(bt_idx as usize)).flatten() {
                map.set_block(x, y, height as usize + i, bt.clone());
            }
        }
    }

    // Object Pos
    let object_pos_end = cursor.position() + header.object_pos_size as u64;
    let mut objects = Vec::new();
    while cursor.position() < object_pos_end {
        let b: MapObjectRaw = cursor.read_le()?;
        objects.push(MapObject {
            x: b.x, y: b.y, z: b.z,
            obj_type: b.obj_type,
            remap: b.remap,
            rotation: b.rotation,
            pitch: b.pitch,
            roll: b.roll,
        });
    }
    map.objects = objects;

    // Routes
    let route_end = cursor.position() + header.route_size as u64;
    let mut routes = Vec::new();
    while cursor.position() < route_end {
        let num_points: u8 = cursor.read_le()?;
        let route_type: u8 = cursor.read_le()?;
        let mut points = Vec::with_capacity(num_points as usize);
        for _ in 0..num_points {
            points.push((cursor.read_le()?, cursor.read_le()?, cursor.read_le()?));
        }
        routes.push(Route { route_type, points });
    }
    map.routes = routes;

    // Locations
    let mut locations_raw = [0u8; 108];
    cursor.read_exact(&mut locations_raw)?;
    let mut loc_cursor = Cursor::new(&locations_raw);

    let mut read_locs = || -> Result<Vec<(u8, u8, u8)>> {
        let mut v = Vec::with_capacity(6);
        for _ in 0..6 { v.push((loc_cursor.read_le()?, loc_cursor.read_le()?, loc_cursor.read_le()?)); }
        Ok(v)
    };

    let police = read_locs()?;
    let hospital = read_locs()?;
    let unused1 = read_locs()?;
    let unused2 = read_locs()?;
    let fire = read_locs()?;
    let unused3 = read_locs()?;

    map.locations = MapLocations {
        police,
        hospital,
        fire,
        unused: vec![unused1, unused2, unused3],
    };

    // Nav Data
    let nav_end = cursor.position() + header.nav_data_size as u64;
    let mut nav_zones = Vec::new();
    while cursor.position() < nav_end {
        let b: NavZoneRaw = cursor.read_le()?;
        nav_zones.push(NavZone { x: b.x, y: b.y, width: b.width, height: b.height, sample: b.sample, name: b.name });
    }
    map.nav_zones = nav_zones;

    // Strictness check
    if cursor.position() < data.len() as u64 {
        return Err(Error::Parse(format!("CMP file has {} trailing bytes", data.len() as u64 - cursor.position())));
    }

    Ok(map)
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
        data.extend_from_slice(&6u16.to_le_bytes()); // height 6 -> 0 blocks

        // Block Types
        data.extend_from_slice(&0u16.to_le_bytes()); // tm
        data.push(0); // tme
        data.push(1); data.push(2); data.push(3); data.push(4); data.push(5); // faces

        // Locations: 108 bytes
        data.extend(vec![0u8; 108]);

        let map = parse_cmp(&data).unwrap();
        assert_eq!(map.style_index, 1);
        assert_eq!(map.width, 256);
        assert_eq!(map.get_block(0,0,0), None);
    }
}
