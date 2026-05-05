use super::graphics::{IndexedImage, Palette};

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub blocks: Vec<IndexedImage>,
    pub side_count: usize,
    pub lid_count: usize,
    pub aux_count: usize,
    
    pub animations: Vec<Animation>,
    pub palette: Palette, // Primary palette for GRY/G24
    pub remap_tables: Vec<[u8; 256]>, // For GRY
    pub remap_indices: Vec<[u8; 4]>, // For GRY (lid remap tables)
    
    // G24 specific
    pub cluts: Vec<Palette>,
    pub palette_index: Vec<u16>,
    pub tile_clut_offset: usize,
    pub sprite_clut_offset: usize,

    pub objects: Vec<ObjectInfo>,
    pub cars: Vec<CarInfo>,
    pub sprites: Vec<Sprite>,
    pub sprite_numbers: SpriteNumbers,
}

impl Style {
    pub fn is_block_animated(&self, map_idx: usize, which: u8) -> bool {
        self.animations.iter().any(|a| a.block as usize == map_idx && a.which == which)
    }

    pub fn get_animated_atlas_idx(&self, map_idx: usize, which: u8, remap: usize, ticks: u64) -> usize {
        for anim in &self.animations {
            if anim.block as usize == map_idx && anim.which == which {
                let total_frames = anim.frames.len() + 1;
                let frame_idx = (ticks / std::cmp::max(1, anim.speed as u64)) % total_frames as u64;
                
                if frame_idx == 0 {
                    break; 
                } else {
                    let aux_idx = anim.frames[frame_idx as usize - 1] as usize;
                    return self.side_count + self.lid_count * 4 + aux_idx;
                }
            }
        }
        
        if which == 0 { // Side
            map_idx
        } else { // Lid
            self.side_count + map_idx * 4 + remap
        }
    }

    /// Gets the RGBA pixels for a block face, handling both GRY and G24 palette systems.
    pub fn get_face_rgba(&self, face_idx: usize, face_type: FaceType, remap: usize) -> Vec<u8> {
        let block_idx = match face_type {
            FaceType::Side => face_idx,
            FaceType::Lid => self.side_count + face_idx,
            FaceType::Aux => self.side_count + self.lid_count + face_idx,
        };
        
        if block_idx >= self.blocks.len() { return vec![0; 64 * 64 * 4]; }
        let block = &self.blocks[block_idx];
        
        if !self.cluts.is_empty() {
            // G24 logic
            let pal_idx_base = match face_type {
                FaceType::Side => 4 * face_idx,
                FaceType::Lid => 4 * (face_idx + self.side_count) + remap,
                FaceType::Aux => 4 * (face_idx + self.side_count + self.lid_count),
            };
            
            let clut_idx = if pal_idx_base < self.palette_index.len() {
                self.palette_index[pal_idx_base] as usize
            } else { 0 };
            
            let palette = self.cluts.get(clut_idx).unwrap_or(&self.palette);
            block.to_rgba(palette)
        } else {
            // GRY logic
            let table_idx = if face_type == FaceType::Lid {
                if face_idx < self.remap_indices.len() {
                    self.remap_indices[face_idx][remap] as usize
                } else { 0 }
            } else { 0 };
            
            let table = self.remap_tables.get(table_idx).unwrap_or(&[0u8; 256]);
            block.to_rgba_remapped(&self.palette, table)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceType { Side, Lid, Aux }

#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub block: u8,
    pub which: u8, // 0 for side, 1 for lid
    pub speed: u8,
    pub frames: Vec<u8>, // Indices into aux_blocks
}

#[derive(Debug, Clone, Default)]
pub struct ObjectInfo {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub spr_num: u16,
    pub weight: u16,
    pub aux: u16,
    pub status: i8,
    pub into: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct CarInfo {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub spr_num: u16,
    pub weight: u16,
    pub max_speed: i16,
    pub min_speed: i16,
    pub acceleration: i16,
    pub braking: i16,
    pub grip: i16,
    pub handling: i16,
    pub remaps: [u8; 12],
    pub vtype: u8,
    pub model: u8,
    pub turning: u8,
    pub damageable: u8,
    pub value: [u16; 4],
    pub cx: i8,
    pub cy: i8,
    pub moment: i32,
    pub mass: f32,
    pub thrust: f32,
    pub adhesion_x: f32,
    pub adhesion_y: f32,
    pub handbrake_friction: f32,
    pub footbrake_friction: f32,
    pub brake_bias: f32,
    pub turn_ratio: i16,
    pub drive_wheel_offset: i16,
    pub steering_wheel_offset: i16,
    pub slide_value: f32,
    pub hb_slide_value: f32,
    pub convertible: u8,
    pub engine: u8,
    pub radio: u8,
    pub horn: u8,
    pub sound_func: u8,
    pub fast_change: u8,
    pub doors: Vec<DoorInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct DoorInfo {
    pub rpx: i16,
    pub rpy: i16,
    pub object: i16,
    pub delta: i16,
}

#[derive(Debug, Clone, Default)]
pub struct Sprite {
    pub width: u8,
    pub height: u8,
    pub image: Vec<u8>,
    pub deltas: Vec<Delta>,
}

#[derive(Debug, Clone, Default)]
pub struct Delta {
    pub commands: Vec<DeltaCommand>,
}

#[derive(Debug, Clone, Default)]
pub struct DeltaCommand {
    pub offset: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct SpriteNumbers {
    pub arrow: u16,
    pub digits: u16,
    pub boat: u16,
    pub box_obj: u16,
    pub bus: u16,
    pub car: u16,
    pub object: u16,
    pub ped: u16,
    pub speedo: u16,
    pub tank: u16,
    pub traffic_lights: u16,
    pub train: u16,
    pub trdoors: u16,
    pub bike: u16,
    pub tram: u16,
    pub wbus: u16,
    pub wcar: u16,
    pub ex: u16,
    pub tumcar: u16,
    pub tumtruck: u16,
    pub ferry: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_indexing_logic() {
        let mut style = Style::default();
        style.side_count = 10;
        style.lid_count = 5;
        
        // Map index 1 -> Atlas index 1
        assert_eq!(style.get_animated_atlas_idx(1, 0, 0, 0), 1);
        
        // Lid index 1 -> side_count + 1*4 + remap = 10 + 4 + 0 = 14
        assert_eq!(style.get_animated_atlas_idx(1, 1, 0, 0), 14);
    }

    #[test]
    fn test_animation_indexing() {
        let mut style = Style::default();
        style.side_count = 100;
        style.lid_count = 50;
        style.animations.push(Animation {
            block: 10,
            which: 0,
            speed: 5,
            frames: vec![0],
        });
        
        // Frame 0
        assert_eq!(style.get_animated_atlas_idx(10, 0, 0, 0), 10);
        // Frame 1 -> side_count + lid_count*4 + aux_idx = 100 + 200 + 0 = 300
        assert_eq!(style.get_animated_atlas_idx(10, 0, 0, 5), 300);
    }
}
