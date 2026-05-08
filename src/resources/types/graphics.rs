/// Represents a 256-color palette used for indexed images.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    /// The RGB colors in the palette.
    pub colors: [[u8; 3]; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self { colors: [[0u8; 3]; 256] }
    }
}

impl Palette {
    /// Applies Hue, Lightness, and Saturation offsets to the entire palette.
    /// Used for dynamic car color variations in GTA 2.
    pub fn apply_hls_offset(&self, h_off: i16, l_off: i16, s_off: i16) -> Self {
        let mut new_colors = [[0u8; 3]; 256];
        for (i, new_color) in new_colors.iter_mut().enumerate() {
            let r = self.colors[i][0] as f32 / 255.0;
            let g = self.colors[i][1] as f32 / 255.0;
            let b = self.colors[i][2] as f32 / 255.0;

            let (h, l, s) = rgb_to_hls(r, g, b);

            // GTA2 logic: Hue in degrees (0-360), Lightness/Saturation in percentage.
            let nh = (h + h_off as f32 / 360.0).rem_euclid(1.0);
            let nl = (l + l_off as f32 / 100.0).clamp(0.0, 1.0);
            let ns = (s + s_off as f32 / 100.0).clamp(0.0, 1.0);

            let (nr, ng, nb) = hls_to_rgb(nh, nl, ns);
            new_color[0] = (nr * 255.0).round() as u8;
            new_color[1] = (ng * 255.0).round() as u8;
            new_color[2] = (nb * 255.0).round() as u8;
        }
        Self { colors: new_colors }
    }
}

/// Converts RGB (0.0 - 1.0) to HLS (0.0 - 1.0).
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

/// Converts HLS (0.0 - 1.0) back to RGB (0.0 - 1.0).
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

/// A 2D image using 8-bit palette indices.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IndexedImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The raw palette indices.
    pub pixels: Vec<u8>,
}

impl IndexedImage {
    /// Creates a new indexed image with the given dimensions and pixel data.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self { width, height, pixels }
    }

    /// Converts the indexed image to 32-bit RGBA pixels using the provided palette.
    /// If `transparent` is true, index 0 is rendered with 0 alpha.
    pub fn to_rgba(&self, palette: &Palette, transparent: bool) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &idx in &self.pixels {
            if transparent && idx == 0 {
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

    /// Converts the indexed image to 32-bit RGBA pixels using a remap table.
    /// Each pixel index is first passed through the remap table before palette lookup.
    pub fn to_rgba_remapped(&self, palette: &Palette, remap_table: &[u8; 256], transparent: bool) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &idx in &self.pixels {
            if transparent && idx == 0 {
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
    fn test_hls_conversion() {
        // Red
        let (h, l, s) = rgb_to_hls(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 1e-6 || (h - 1.0).abs() < 1e-6);
        assert!((l - 0.5).abs() < 1e-6);
        assert!((s - 1.0).abs() < 1e-6);

        let (r, g, b) = hls_to_rgb(0.0, 0.5, 1.0);
        assert!((r - 1.0).abs() < 1e-6);
        assert!((g - 0.0).abs() < 1e-6);
        assert!((b - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_palette_hls_offset() {
        let mut colors = [[0u8; 3]; 256];
        colors[1] = [255, 0, 0]; // Red
        let pal = Palette { colors };

        // Offset lightness by +20%
        let pal2 = pal.apply_hls_offset(0, 20, 0);
        // Lightness 0.5 -> 0.7. RGB should become roughly (255, 102, 102)
        // q = 0.7 * (1 + 0.3) = 0.91? No, s is 1.0.
        // q = 0.7 + 1.0 - 0.7*1.0 = 1.0. p = 2*0.7 - 1.0 = 0.4.
        // Red at l=0.7: r=1.0, g=0.4, b=0.4 -> (255, 102, 102)
        assert_eq!(pal2.colors[1], [255, 102, 102]);
    }

    #[test]
    fn test_to_rgba_transparency() {
        let palette = Palette { colors: [[255, 0, 0]; 256] };
        let img = IndexedImage::new(1, 1, vec![0]); // Index 0
        let rgba = img.to_rgba(&palette, true);
        assert_eq!(rgba, vec![0, 0, 0, 0]); // Transparent

        let img2 = IndexedImage::new(1, 1, vec![1]); // Index 1
        let rgba2 = img2.to_rgba(&palette, true);
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
        let rgba = img.to_rgba_remapped(&palette, &remap, true);
        assert_eq!(rgba, vec![0, 0, 255, 255]);
    }
}

