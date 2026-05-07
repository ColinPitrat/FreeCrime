use super::graphics::{IndexedImage, Palette};
use serde::Serialize;
use std::collections::HashMap;

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
    pub tile_cl_count: usize,
    pub sprite_cl_count: usize,

    pub objects: Vec<ObjectInfo>,
    pub cars: Vec<CarInfo>,
    pub sprites: Vec<Sprite>,
    pub sprite_numbers: SpriteNumbers,

    /// Mapping from Aux block index to the (block_idx, which) that triggers it.
    /// which: 0 for Side, 1 for Lid.
    /// Used to inherit shading (remaps) for animation frames.
    pub aux_to_trigger: HashMap<usize, (usize, u8)>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum GtaVersion {
    Gta1,
    London,
    Gta2,
}

impl Style {
    pub fn is_block_animated(&self, map_idx: usize, which: u8) -> bool {
        self.animations.iter().any(|a| a.block as usize == map_idx && a.which == which)
    }

    pub fn get_animated_atlas_idx(&self, map_idx: usize, which: u8, remap: usize, ticks: u64, version: GtaVersion) -> usize {
        for anim in &self.animations {
            if anim.block as usize == map_idx && anim.which == which {
                // London Water Lid (fc=11) is an 11-frame loop including the base.
                // NYC Side 100 (fc=1) is a 2-frame loop (Base + 1 Aux).
                let total_frames = if version == GtaVersion::London && which == 1 {
                    anim.frames.len()
                } else {
                    anim.frames.len() + 1
                };

                if total_frames == 0 { break; }
                let frame_idx = (ticks / std::cmp::max(1, anim.speed as u64)) % total_frames as u64;

                if frame_idx == 0 {
                    break;
                } else {
                    let aux_idx = anim.frames[frame_idx as usize - 1] as usize;
                    // Lids and Aux both have 4 slots per block in the atlas
                    return self.side_count + self.lid_count * 4 + aux_idx * 4 + remap;
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
    pub fn get_face_rgba(&self, face_idx: usize, face_type: FaceType, remap: usize, version: GtaVersion) -> Vec<u8> {
        let block_idx = match face_type {
            FaceType::Side => face_idx,
            FaceType::Lid => self.side_count + face_idx,
            FaceType::Aux => self.side_count + self.lid_count + face_idx,
        };

        if block_idx >= self.blocks.len() { return vec![0; 64 * 64 * 4]; }
        let block = &self.blocks[block_idx];

        // London map lids/aux are opaque (index 0 is a visible color like black).
        // Standard GTA 1 and GTA 2 tiles support index 0 transparency.
        // Sides (walls) always support index 0 transparency for fences/decorations.
        let transparent = match face_type {
            FaceType::Side => true,
            FaceType::Lid | FaceType::Aux => version != GtaVersion::London,
        };

        if !self.cluts.is_empty() {
            // G24 logic: Global Indexing (Sides -> Lids -> Aux)
            let pal_idx_base = match face_type {
                FaceType::Side => 4 * face_idx,
                FaceType::Lid => 4 * (face_idx + self.side_count) + remap,
                FaceType::Aux => {
                    // London fix: Aux blocks have un-shaded entries in mapping table.
                    // Inherit triggering Lid's palette index to ensure shaded animations.
                    if version == GtaVersion::London {
                        let trigger_lid = self.aux_to_trigger.get(&face_idx).cloned();
                        if let Some((lid_idx, _)) = trigger_lid {
                            4 * (lid_idx + self.side_count) + remap
                        } else {
                            4 * (face_idx + self.side_count + self.lid_count) + remap
                        }
                    } else {
                        4 * (face_idx + self.side_count + self.lid_count) + remap
                    }
                }
            };

            let clut_idx = if pal_idx_base < self.palette_index.len() {
                self.palette_index[pal_idx_base] as usize
            } else { 0 };

            let palette = self.cluts.get(clut_idx).unwrap_or(&self.palette);
            block.to_rgba(palette, transparent)
        } else {
            // GRY logic: Local Indexing (Lids start at 0). Sides are usually not remapped.
            let table_idx = match face_type {
                FaceType::Side => 0,
                FaceType::Lid => self.remap_indices.get(face_idx).map(|r| r[remap] as usize).unwrap_or(0),
                FaceType::Aux => {
                    // London fix: Aux blocks have un-shaded entries in mapping table.
                    // Inherit triggering Lid's palette index to ensure shaded animations.
                    let trigger = self.aux_to_trigger.get(&face_idx).cloned();
                    if let Some((idx, which)) = trigger {
                         if which == 1 { // Lid
                             self.remap_indices.get(idx).map(|r| r[remap] as usize).unwrap_or(0)
                         } else { 0 }
                    } else { 0 }
                }
            };

            let table = self.remap_tables.get(table_idx).unwrap_or(&[0u8; 256]);
            block.to_rgba_remapped(&self.palette, table, transparent)
        }
    }

    pub fn get_sprite_offsets(&self) -> HashMap<&'static str, usize> {
        let mut offsets = HashMap::new();
        let mut current = 0;

        offsets.insert("SPR_ARROW", current); current += self.sprite_numbers.arrow as usize;
        offsets.insert("SPR_DIGITS", current); current += self.sprite_numbers.digits as usize;
        offsets.insert("SPR_BOAT", current); current += self.sprite_numbers.boat as usize;
        offsets.insert("SPR_BOX", current); current += self.sprite_numbers.box_obj as usize;
        offsets.insert("SPR_BUS", current); current += self.sprite_numbers.bus as usize;
        offsets.insert("SPR_CAR", current); current += self.sprite_numbers.car as usize;
        offsets.insert("SPR_OBJECT", current); current += self.sprite_numbers.object as usize;
        offsets.insert("SPR_PED", current); current += self.sprite_numbers.ped as usize;
        offsets.insert("SPR_SPEEDO", current); current += self.sprite_numbers.speedo as usize;
        offsets.insert("SPR_TANK", current); current += self.sprite_numbers.tank as usize;
        offsets.insert("SPR_TRAFFIC_LIGHTS", current); current += self.sprite_numbers.traffic_lights as usize;
        offsets.insert("SPR_TRAIN", current); current += self.sprite_numbers.train as usize;
        offsets.insert("SPR_TRDOORS", current); current += self.sprite_numbers.trdoors as usize;
        offsets.insert("SPR_BIKE", current); current += self.sprite_numbers.bike as usize;
        offsets.insert("SPR_TRAM", current); current += self.sprite_numbers.tram as usize;
        offsets.insert("SPR_WBUS", current); current += self.sprite_numbers.wbus as usize;
        offsets.insert("SPR_WCAR", current); current += self.sprite_numbers.wcar as usize;
        offsets.insert("SPR_EX", current); current += self.sprite_numbers.ex as usize;
        offsets.insert("SPR_TUMCAR", current); current += self.sprite_numbers.tumcar as usize;
        offsets.insert("SPR_TUMTRUCK", current); current += self.sprite_numbers.tumtruck as usize;
        offsets.insert("SPR_FERRY", current);

        offsets
    }
}

pub fn vehicle_type_const(vtype: u8) -> &'static str {
    match vtype {
        0 => "SPR_BUS",
        3 => "SPR_BIKE",
        4 => "SPR_CAR",
        8 => "SPR_TRAIN",
        9 => "SPR_TRAM",
        13 => "SPR_BOAT",
        14 => "SPR_TANK",
        _ => "SPR_CAR",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceType { Side, Lid, Aux }

#[derive(Debug, Clone, Default, Serialize)]
pub struct Animation {
    pub block: u8,
    pub which: u8, // 0 for side, 1 for lid
    pub speed: u8,
    pub frames: Vec<u8>, // Indices into aux_blocks
}

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
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
    pub remap24: Vec<[i16; 3]>,
    pub remap8: Vec<u8>,
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

#[derive(Debug, Clone, Default, Serialize)]
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
    pub ws: u8,
    pub size: u16,
    pub ptr: u32,
    pub clut: u16,
    pub pixels: Vec<u8>, // Indexed
    pub deltas: Vec<Delta>,
}

impl Sprite {
    pub fn apply_delta(&self, delta_idx: usize) -> Vec<u8> {
        if delta_idx >= self.deltas.len() { return self.pixels.clone(); }
        let mut pixels = self.pixels.clone();
        let delta = &self.deltas[delta_idx];
        let mut offset = 0;
        let mut curr_x = 0;
        let mut curr_y = 0;

        while offset < delta.data.len() {
            if offset + 3 > delta.data.len() { break; }
            let dx = delta.data[offset];
            let dy = delta.data[offset+1];
            offset += 2;

            curr_x += dx as i32;
            curr_y += dy as i32;

            while curr_x >= self.width as i32 {
                curr_x -= 256;
                curr_y += 1;
            }

            let curr_pos = curr_y * (self.width as i32) + curr_x;
            let length = delta.data[offset] as usize;
            offset += 1;

            if offset + length > delta.data.len() { break; }
            let data = &delta.data[offset..offset+length];
            offset += length;

            for (i, &item) in data.iter().enumerate().take(length) {
                let pos = (curr_pos + i as i32) as usize;
                if pos < pixels.len() {
                    pixels[pos] = item;
                }
            }
            curr_x += length as i32;
        }
        pixels
    }
}

#[derive(Debug, Clone, Default)]
pub struct Delta {
    pub size: u16,
    pub ptr: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize)]
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
        assert_eq!(style.get_animated_atlas_idx(1, 0, 0, 0, GtaVersion::Gta1), 1);

        // Lid index 1 -> side_count + 1*4 + remap = 10 + 4 + 0 = 14
        assert_eq!(style.get_animated_atlas_idx(1, 1, 0, 0, GtaVersion::Gta1), 14);
    }

    #[test]
    fn test_animation_indexing() {
        let mut style = Style::default();
        style.side_count = 100;
        style.lid_count = 50;
        style.animations.push(Animation {
            block: 10,
            which: 1, // Lid
            speed: 5,
            frames: vec![0, 1], // fc=2
        });

        // Gta1 Lid: total_frames = fc + 1 = 3 ([Base, Aux 0, Aux 1])
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 0, GtaVersion::Gta1), 100 + 10 * 4);
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 5, GtaVersion::Gta1), 100 + 50 * 4 + 0 * 4);
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 10, GtaVersion::Gta1), 100 + 50 * 4 + 1 * 4);

        // London Lid: total_frames = fc = 2 ([Base, Aux 0])
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 0, GtaVersion::London), 100 + 10 * 4);
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 5, GtaVersion::London), 100 + 50 * 4 + 0 * 4);
        assert_eq!(style.get_animated_atlas_idx(10, 1, 0, 10, GtaVersion::London), 100 + 10 * 4);
    }

}
