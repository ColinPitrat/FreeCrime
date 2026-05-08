#[derive(Debug, Clone, Default, PartialEq)]
pub struct Map {
    pub style_index: u8,
    pub sample_index: u8,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub blocks: Vec<Option<Block>>,
    pub objects: Vec<MapObject>,
    pub routes: Vec<Route>,
    pub locations: MapLocations,
    pub nav_zones: Vec<NavZone>,
}

impl Map {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
            blocks: vec![None; width * height * depth],
            ..Default::default()
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x >= self.width || y >= self.height || z >= self.depth { return None; }
        self.blocks[z * self.width * self.height + y * self.width + x].as_ref()
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < self.width && y < self.height && z < self.depth {
            self.blocks[z * self.width * self.height + y * self.width + x] = Some(block);
        }
    }

    pub fn get_lid_flatness(&self) -> Vec<bool> {
        let mut flatness = vec![false; 256];
        for block in self.blocks.iter().flatten() {
            if block.lid != 0 && block.is_flat() {
                flatness[block.lid as usize] = true;
            }
        }
        flatness
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub type_map: u16,
    pub type_map_ext: u8,
    pub left: u8,
    pub right: u8,
    pub top: u8,
    pub bottom: u8,
    pub lid: u8,
}

impl Block {
    pub fn block_type(&self) -> u8 {
        ((self.type_map & 0x70) >> 4) as u8
    }

    pub fn is_flat(&self) -> bool {
        (self.type_map & 0x80) != 0
    }

    pub fn slope_type(&self) -> u8 {
        ((self.type_map & 0x3F00) >> 8) as u8
    }

    pub fn lid_rotation(&self) -> u8 {
        ((self.type_map & 0xC000) >> 14) as u8
    }

    pub fn lid_remap(&self) -> u8 {
        (self.type_map_ext & 0x18) >> 3
    }

    pub fn flip_top_bottom(&self) -> bool {
        (self.type_map_ext & 0x20) == 0
    }

    pub fn flip_left_right(&self) -> bool {
        (self.type_map_ext & 0x40) == 0
    }

    pub fn get_slope_deltas(&self) -> (f32, f32, f32, f32) {
        let slope_type = self.slope_type();
        match slope_type {
            0 => (0.0, 0.0, 0.0, 0.0),
            1 => (0.5, 0.5, 1.0, 1.0), 2 => (0.0, 0.0, 0.5, 0.5),
            3 => (1.0, 1.0, 0.5, 0.5), 4 => (0.5, 0.5, 0.0, 0.0),
            5 => (0.5, 1.0, 1.0, 0.5), 6 => (0.0, 0.5, 0.5, 0.0),
            7 => (1.0, 0.5, 0.5, 1.0), 8 => (0.5, 0.0, 0.0, 0.5),
            41 => (0.0, 0.0, 1.0, 1.0), 42 => (1.0, 1.0, 0.0, 0.0),
            43 => (0.0, 1.0, 1.0, 0.0), 44 => (1.0, 0.0, 0.0, 1.0),
            _ => {
                if (9..=40).contains(&slope_type) {
                    let (dir, step) = ((slope_type - 9) / 8, (slope_type - 9) % 8);
                    let start = 1.0 - (step as f32 * 0.125);
                    let end = start - 0.125;
                    match dir {
                        0 => (end, end, start, start), // North
                        1 => (start, start, end, end), // South
                        2 => (end, start, start, end), // West
                        3 => (start, end, end, start), // East
                        _ => (0.0, 0.0, 0.0, 0.0),
                    }
                } else { (0.0, 0.0, 0.0, 0.0) }
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapObject {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub obj_type: u8,
    pub remap: u8,
    pub rotation: u16,
    pub pitch: u16,
    pub roll: u16,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Route {
    pub route_type: u8,
    pub points: Vec<(u8, u8, u8)>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapLocations {
    pub police: Vec<(u8, u8, u8)>,
    pub hospital: Vec<(u8, u8, u8)>,
    pub fire: Vec<(u8, u8, u8)>,
    pub unused: Vec<Vec<(u8, u8, u8)>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NavZone {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub sample: u8,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_slope_deltas() {
        let mut b = Block::default();
        b.type_map = 0 << 8; // No slope
        assert_eq!(b.get_slope_deltas(), (0.0, 0.0, 0.0, 0.0));

        b.type_map = 1 << 8; // 26 deg North
        assert_eq!(b.get_slope_deltas(), (0.5, 0.5, 1.0, 1.0));
    }
}
