/// Represents a 3D GTA map with multiple layers of blocks, objects, and navigation data.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Map {
    pub style_index: u8,
    pub sample_index: u8,
    /// Width of the map in blocks (usually 256).
    pub width: usize,
    /// Height of the map in blocks (usually 256).
    pub height: usize,
    /// Number of vertical layers (usually 6).
    pub depth: usize,
    /// Flat 1D vector representing the 3D grid of blocks.
    pub blocks: Vec<Option<Block>>,
    /// List of static 3D objects placed in the map.
    pub objects: Vec<MapObject>,
    /// List of AI and player routes.
    pub routes: Vec<Route>,
    /// Pre-defined locations like police stations and hospitals.
    pub locations: MapLocations,
    /// Navigation zones for samples and area names.
    pub nav_zones: Vec<NavZone>,
}

impl Map {
    /// Creates a new map with the given dimensions, initializing all blocks to None.
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
            blocks: vec![None; width * height * depth],
            ..Default::default()
        }
    }

    /// Retrieves a reference to the block at the given coordinates.
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        if x >= self.width || y >= self.height || z >= self.depth { return None; }
        self.blocks[z * self.width * self.height + y * self.width + x].as_ref()
    }

    /// Sets the block at the given coordinates.
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < self.width && y < self.height && z < self.depth {
            self.blocks[z * self.width * self.height + y * self.width + x] = Some(block);
        }
    }

    /// Scans the entire map to identify which Lid IDs are used in blocks marked as "Flat".
    /// This is used by the transparency logic to determine if a Lid should be transparent.
    pub fn get_lid_flatness(&self) -> Vec<bool> {
        let mut flatness = vec![false; 256];
        for block in self.blocks.iter().flatten() {
            if block.lid == 0 {
                continue
            }
            if block.is_flat() {
                flatness[block.lid as usize] = true;
            }
            // TODO: Dig into this. Some lids are used on both flats and non-flats blocks.
            // This is fine as long as they don't use color 0. It would be nice to check that!
            // But this requires a back and forth between CMP and GRY. Or passing more info from
            // CMP to GRY.
            /*
            else if flatness[block.lid as usize] {
                eprintln!("Warning: {block:?} is not flat but has lid {} which is used in a flat block (so lid should be transparent sometimes, and sometimes not!)", block.lid)
            }
            */
        }
        flatness
    }
}

/// A single block in the 3D map grid, containing texture indices and physical properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    /// Contains slope, rotation, and block type bits.
    pub type_map: u16,
    /// Contains remap, flip, and transparency bits.
    pub type_map_ext: u8,
    /// Side texture index for the West face.
    pub left: u8,
    /// Side texture index for the East face.
    pub right: u8,
    /// Side texture index for the North face.
    pub top: u8,
    /// Side texture index for the South face.
    pub bottom: u8,
    /// Lid texture index for the top face.
    pub lid: u8,
}

impl Block {
    /// Returns the block type (0-7), defining its collision behavior.
    pub fn block_type(&self) -> u8 {
        ((self.type_map & 0x70) >> 4) as u8
    }

    /// Returns true if the block is "Flat" (used for signs, fences, etc).
    pub fn is_flat(&self) -> bool {
        (self.type_map & 0x80) != 0
    }

    /// Returns the slope type ID of the block.
    pub fn slope_type(&self) -> u8 {
        ((self.type_map & 0x3F00) >> 8) as u8
    }

    /// Returns the rotation of the Lid texture (0-3).
    pub fn lid_rotation(&self) -> u8 {
        ((self.type_map & 0xC000) >> 14) as u8
    }

    /// Returns the remap index (0-3) for the Lid texture.
    pub fn lid_remap(&self) -> u8 {
        (self.type_map_ext & 0x18) >> 3
    }

    /// Returns true if the wall textures should be flipped vertically.
    pub fn flip_top_bottom(&self) -> bool {
        (self.type_map_ext & 0x20) == 0
    }

    /// Returns true if the wall textures should be flipped horizontally.
    pub fn flip_left_right(&self) -> bool {
        (self.type_map_ext & 0x40) == 0
    }

    /// Calculates the vertical offsets (0.0 to 1.0) for each of the 4 Lid corners based on slope.
    /// Returns (Top-Left, Top-Right, Bottom-Right, Bottom-Left).
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

/// Metadata for a dynamic 3D object placed in the world.
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

/// A sequence of points defining a path for AI or trains.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Route {
    pub route_type: u8,
    pub points: Vec<(u8, u8, u8)>,
}

/// Map-wide collections of critical gameplay locations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapLocations {
    pub police: Vec<(u8, u8, u8)>,
    pub hospital: Vec<(u8, u8, u8)>,
    pub fire: Vec<(u8, u8, u8)>,
    pub unused: Vec<Vec<(u8, u8, u8)>>,
}

/// A named zone in the map for sound triggers or UI names.
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

    #[test]
    fn test_seven_degree_slopes() {
        let mut b = Block::default();
        // Type 9: North, Step 0. start=1.0, end=0.875
        b.type_map = 9 << 8;
        assert_eq!(b.get_slope_deltas(), (0.875, 0.875, 1.0, 1.0));

        // Type 17: South, Step 0. start=1.0, end=0.875
        b.type_map = 17 << 8;
        assert_eq!(b.get_slope_deltas(), (1.0, 1.0, 0.875, 0.875));
    }

    #[test]
    fn test_lid_flatness() {
        let mut map = Map::new(1, 1, 1);
        let mut b = Block::default();
        b.lid = 10;
        b.type_map = 0x80; // Flat
        map.set_block(0, 0, 0, b);

        let flatness = map.get_lid_flatness();
        assert!(flatness[10]);
        assert!(!flatness[11]);
    }
}
