#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub colors: [[u8; 3]; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self { colors: [[0u8; 3]; 256] }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IndexedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl IndexedImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self { width, height, pixels }
    }

    pub fn to_rgba(&self, palette: &Palette) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &idx in &self.pixels {
            if idx == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette.colors[idx as usize];
                rgba.push(color[0]);
                rgba.push(color[1]);
                rgba.push(color[2]);
                rgba.push(255);
            }
        }
        rgba
    }

    pub fn to_rgba_remapped(&self, palette: &Palette, remap_table: &[u8; 256]) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &idx in &self.pixels {
            if idx == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let remapped_idx = remap_table[idx as usize];
                let color = palette.colors[remapped_idx as usize];
                rgba.push(color[0]);
                rgba.push(color[1]);
                rgba.push(color[2]);
                rgba.push(255);
            }
        }
        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_rgba_transparency() {
        let palette = Palette { colors: [[255, 0, 0]; 256] };
        let img = IndexedImage::new(1, 1, vec![0]); // Index 0
        let rgba = img.to_rgba(&palette);
        assert_eq!(rgba, vec![0, 0, 0, 0]); // Transparent
        
        let img2 = IndexedImage::new(1, 1, vec![1]); // Index 1
        let rgba2 = img2.to_rgba(&palette);
        assert_eq!(rgba2, vec![255, 0, 0, 255]); // Opaque
    }

    #[test]
    fn test_to_rgba_remapped() {
        let mut colors = [[0u8; 3]; 256];
        colors[1] = [0, 255, 0];
        colors[2] = [0, 0, 255];
        let palette = Palette { colors };
        let mut remap = [0u8; 256];
        remap[1] = 2; // Map index 1 to palette color 2 (Blue)
        let img = IndexedImage::new(1, 1, vec![1]);
        let rgba = img.to_rgba_remapped(&palette, &remap);
        assert_eq!(rgba, vec![0, 0, 255, 255]);
    }
}
