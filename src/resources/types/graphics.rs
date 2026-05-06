#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub colors: [[u8; 3]; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self { colors: [[0u8; 3]; 256] }
    }
}

impl Palette {
    pub fn apply_hls_offset(&self, h_off: i16, l_off: i16, s_off: i16) -> Self {
        let mut new_colors = [[0u8; 3]; 256];
        for (i, new_color) in new_colors.iter_mut().enumerate() {
            let r = self.colors[i][0] as f32 / 255.0;
            let g = self.colors[i][1] as f32 / 255.0;
            let b = self.colors[i][2] as f32 / 255.0;

            let (h, l, s) = rgb_to_hls(r, g, b);

            // GTA2 logic: Hue in degrees (0-360), Lightness/Saturation in percentage?
            // Python: h = (h + h_off / 360.0) % 1.0
            let nh = (h + h_off as f32 / 360.0).rem_euclid(1.0);
            let nl = (l + l_off as f32 / 100.0).clamp(0.0, 1.0);
            let ns = (s + s_off as f32 / 100.0).clamp(0.0, 1.0);

            let (nr, ng, nb) = hls_to_rgb(nh, nl, ns);
            new_color[0] = (nr * 255.0) as u8;
            new_color[1] = (ng * 255.0) as u8;
            new_color[2] = (nb * 255.0) as u8;
        }
        Self { colors: new_colors }
    }
}

// Minimal HLS/RGB conversion helpers
fn rgb_to_hls(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min { return (0.0, l, 0.0); }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == r {
        (g - b) / d + (if g < b { 6.0 } else { 0.0 })
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, l, s)
}

fn hls_to_rgb(h: f32, l: f32, s: f32) -> (f32, f32, f32) {
    if s == 0.0 { return (l, l, l); } // Greyscale
    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0/2.0 { return q; }
        if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
        p
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (hue_to_rgb(p, q, h + 1.0/3.0), hue_to_rgb(p, q, h), hue_to_rgb(p, q, h - 1.0/3.0))
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
