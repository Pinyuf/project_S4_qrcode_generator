use crate::Args;
use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgba, imageops::FilterType};
use std::{
    fmt::Write as _,
    fs,
    io::Cursor,
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

pub struct GeneratedQr {
    pub path: String,
    pub preview: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub fn generate_qr(link: &str, args: &Args) -> Result<GeneratedQr, String> {
    generate_qr_internal(link, args, true)
}

pub fn generate_qr_preview(link: &str, args: &Args) -> Result<GeneratedQr, String> {
    generate_qr_internal(link, args, false)
}

fn generate_qr_internal(link: &str, args: &Args, save_svg: bool) -> Result<GeneratedQr, String> {
    if link.trim().is_empty() {
        return Err("Le lien ne peut pas être vide.".to_string());
    }

    let fg =
        parse_hex_color(&args.fg).map_err(|e| format!("Couleur de premier plan invalide: {e}"))?;
    let bg = parse_hex_color(&args.bg).map_err(|e| format!("Couleur de fond invalide: {e}"))?;

    let code = ManualQrCode::encode(link.as_bytes())
        .map_err(|e| format!("Impossible de générer le QR code: {e}"))?;
    let modules = code.size() as u32;

    let quiet_zone = 4u32;
    let scale = ((args.size + modules - 1) / modules).max(1);
    let dim = (modules + quiet_zone * 2) * scale;

    let background = load_background(args, dim, dim)?;
    let fg_color = Rgba([fg.0, fg.1, fg.2, 255]);
    let fallback_bg = Rgba([bg.0, bg.1, bg.2, 255]);
    let background = background.map(|mut img| {
        // Adoucit l'image de fond pour garantir un contraste suffisant.
        apply_overlay(&mut img, fallback_bg, 0.55);
        fill_quiet_zone(&mut img, quiet_zone * scale, fallback_bg);
        img
    });

    let mut colored: ImageBuffer<Rgba<u8>, Vec<u8>> = if let Some(ref bg_img) = background {
        bg_img.clone()
    } else {
        ImageBuffer::from_pixel(dim, dim, fallback_bg)
    };

    for y in 0..modules {
        for x in 0..modules {
            let is_dark = code.get_module(x as usize, y as usize);
            if is_dark {
                let base_x = (x + quiet_zone) * scale;
                let base_y = (y + quiet_zone) * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        colored.put_pixel(base_x + dx, base_y + dy, fg_color);
                    }
                }
            } else if background.is_some() {
                let base_x = (x + quiet_zone) * scale;
                let base_y = (y + quiet_zone) * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        let mut px = *colored.get_pixel(base_x + dx, base_y + dy);
                        px = blend_with_color(px, fallback_bg, 0.65);
                        colored.put_pixel(base_x + dx, base_y + dy, px);
                    }
                }
            }
        }
    }

    if save_svg {
        let svg_content = build_svg(&code, dim, scale, quiet_zone, fg, bg, background.as_ref())?;
        fs::write(&args.output, svg_content)
            .map_err(|e| format!("Impossible de sauvegarder le fichier: {e}"))?;
    }

    Ok(GeneratedQr {
        path: args.output.clone(),
        preview: colored,
    })
}

fn build_svg(
    code: &ManualQrCode,
    dim: u32,
    scale: u32,
    quiet_zone: u32,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    background: Option<&ImageBuffer<Rgba<u8>, Vec<u8>>>,
) -> Result<String, String> {
    let fg_hex = format!("#{:02x}{:02x}{:02x}", fg.0, fg.1, fg.2);
    let bg_hex = format!("#{:02x}{:02x}{:02x}", bg.0, bg.1, bg.2);

    let mut svg = String::with_capacity(16_000);
    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{dim}" height="{dim}" viewBox="0 0 {dim} {dim}" shape-rendering="crispEdges">"#,
        dim=dim
    ).map_err(|_| "Erreur interne lors de la construction du SVG")?;

    if let Some(img) = background {
        let data_uri = encode_background(img)?;
        writeln!(
            svg,
            r#"<image href="{data_uri}" x="0" y="0" width="{dim}" height="{dim}" preserveAspectRatio="none" />"#,
            dim=dim
        ).map_err(|_| "Erreur interne lors de la construction du SVG")?;
    } else {
        writeln!(
            svg,
            r#"<rect width="100%" height="100%" fill="{bg_hex}" />"#
        )
        .map_err(|_| "Erreur interne lors de la construction du SVG")?;
    }

    svg.push_str(r#"<g fill=""#);
    svg.push_str(&fg_hex);
    svg.push_str(r#"">"#);

    let modules = code.size() as u32;
    for y in 0..modules {
        for x in 0..modules {
            if code.get_module(x as usize, y as usize) {
                let px = (x + quiet_zone) * scale;
                let py = (y + quiet_zone) * scale;
                writeln!(
                    svg,
                    r#"<rect x="{px}" y="{py}" width="{scale}" height="{scale}" />"#
                )
                .map_err(|_| "Erreur interne lors de la construction du SVG")?;
            }
        }
    }

    svg.push_str("</g></svg>");
    Ok(svg)
}

fn encode_background(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String, String> {
    let mut png_bytes = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_bytes);
        DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut cursor, ImageOutputFormat::Png)
            .map_err(|e| format!("Impossible d'encoder l'image de fond: {e}"))?;
    }

    let encoded = encode_base64(&png_bytes);
    Ok(format!("data:image/png;base64,{encoded}"))
}

fn encode_base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let chunk = &data[i..i + 3];
        let n = u32::from(chunk[0]) << 16 | u32::from(chunk[1]) << 8 | u32::from(chunk[2]);
        out.push(TABLE[(n >> 18) as usize] as char);
        out.push(TABLE[(n >> 12 & 0x3F) as usize] as char);
        out.push(TABLE[(n >> 6 & 0x3F) as usize] as char);
        out.push(TABLE[(n & 0x3F) as usize] as char);
        i += 3;
    }

    match data.len() - i {
        1 => {
            let n = u32::from(data[i]) << 16;
            out.push(TABLE[(n >> 18) as usize] as char);
            out.push(TABLE[(n >> 12 & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = u32::from(data[i]) << 16 | u32::from(data[i + 1]) << 8;
            out.push(TABLE[(n >> 18) as usize] as char);
            out.push(TABLE[(n >> 12 & 0x3F) as usize] as char);
            out.push(TABLE[(n >> 6 & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => {}
    }

    out
}

#[derive(Clone)]
struct CachedBackground {
    path: String,
    modified: Option<SystemTime>,
    width: u32,
    height: u32,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

fn background_cache() -> &'static Mutex<Vec<CachedBackground>> {
    static CACHE: OnceLock<Mutex<Vec<CachedBackground>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

fn load_background(
    args: &Args,
    width: u32,
    height: u32,
) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>, String> {
    let Some(path) = args.bg_image.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Ok(None);
    };

    let metadata = fs::metadata(path).ok();
    let modified = metadata.as_ref().and_then(|m| m.modified().ok());

    if let Ok(cache) = background_cache().lock() {
        if let Some(entry) = cache
            .iter()
            .find(|c| c.path == path.as_str() && c.width == width && c.height == height && c.modified == modified)
        {
            return Ok(Some(entry.image.clone()));
        }
    }

    let img = image::open(path)
        .map_err(|e| format!("Impossible de charger l'image de fond \"{path}\": {e}"))?
        .to_rgba8();

    let resized = image::imageops::resize(&img, width, height, FilterType::Triangle);

    if let Ok(mut cache) = background_cache().lock() {
        let entry = CachedBackground {
            path: path.to_string(),
            modified,
            width,
            height,
            image: resized.clone(),
        };
        if cache.len() >= 3 {
            cache.remove(0);
        }
        cache.push(entry);
    }

    Ok(Some(resized))
}

fn apply_overlay(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, color: Rgba<u8>, alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    let inv = 1.0 - alpha;

    for pixel in img.pixels_mut() {
        for i in 0..3 {
            let base = pixel[i] as f32 * inv;
            let overlay = color[i] as f32 * alpha;
            pixel[i] = (base + overlay).round().clamp(0.0, 255.0) as u8;
        }
    }
}

fn fill_quiet_zone(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, quiet_px: u32, color: Rgba<u8>) {
    let width = img.width();
    let height = img.height();

    for y in 0..height {
        for x in 0..width {
            if x < quiet_px || y < quiet_px || x >= width - quiet_px || y >= height - quiet_px {
                img.put_pixel(x, y, color);
            }
        }
    }
}

fn blend_with_color(pixel: Rgba<u8>, overlay: Rgba<u8>, alpha: f32) -> Rgba<u8> {
    let alpha = alpha.clamp(0.0, 1.0);
    let inv = 1.0 - alpha;
    let mut out = pixel;
    for i in 0..3 {
        let blended = pixel[i] as f32 * inv + overlay[i] as f32 * alpha;
        out[i] = blended.round().clamp(0.0, 255.0) as u8;
    }
    out
}

fn parse_hex_color(input: &str) -> Result<(u8, u8, u8), &'static str> {
    let trimmed = input.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return Err("attendu format hex sur 6 caractères, ex: #aabbcc");
    }

    let r = u8::from_str_radix(&trimmed[0..2], 16).map_err(|_| "valeur rouge invalide")?;
    let g = u8::from_str_radix(&trimmed[2..4], 16).map_err(|_| "valeur verte invalide")?;
    let b = u8::from_str_radix(&trimmed[4..6], 16).map_err(|_| "valeur bleue invalide")?;
    Ok((r, g, b))
}

struct ManualQrCode {
    version: usize,
    size: usize,
    modules: Vec<bool>,
    is_function: Vec<bool>,
}

impl ManualQrCode {
    fn encode(data: &[u8]) -> Result<Self, String> {
        let version = pick_version(data.len())?;
        let capacity_bits = num_data_codewords(version) * 8;
        let char_count_bits = if version <= 9 { 8 } else { 16 };

        let mut buffer = BitBuffer::new();
        buffer.push_bits(0b0100, 4); // Byte mode
        buffer.push_bits(data.len() as u32, char_count_bits as u8);
        for &b in data {
            buffer.push_bits(b as u32, 8);
        }

        let terminator = (capacity_bits - buffer.len()).min(4);
        buffer.push_bits(0, terminator as u8);
        while buffer.len() % 8 != 0 {
            buffer.push_bit(false);
        }

        let mut pad_toggle = true;
        while buffer.len() / 8 < num_data_codewords(version) {
            buffer.push_bits(if pad_toggle { 0xEC } else { 0x11 }, 8);
            pad_toggle = !pad_toggle;
        }

        let data_codewords = buffer.to_bytes();
        let mut qr = ManualQrCode::new(version);
        let full_codewords = qr.add_ecc_and_interleave(&data_codewords);

        qr.draw_function_patterns();
        qr.draw_codewords(&full_codewords);
        qr.apply_mask();
        qr.draw_format_bits();

        Ok(qr)
    }

    fn new(version: usize) -> Self {
        let size = version * 4 + 17;
        Self {
            version,
            size,
            modules: vec![false; size * size],
            is_function: vec![false; size * size],
        }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get_module(&self, x: usize, y: usize) -> bool {
        if x < self.size && y < self.size {
            self.modules[y * self.size + x]
        } else {
            false
        }
    }

    fn add_ecc_and_interleave(&self, data: &[u8]) -> Vec<u8> {
        debug_assert_eq!(data.len(), num_data_codewords(self.version));
        let numblocks = num_error_correction_blocks(self.version);
        let blockecclen = ecc_codewords_per_block(self.version);
        let rawcodewords = num_raw_data_modules(self.version) / 8;
        let numshortblocks = numblocks - rawcodewords % numblocks;
        let shortblocklen = rawcodewords / numblocks;

        let mut blocks: Vec<Vec<u8>> = Vec::with_capacity(numblocks);
        let rsdiv = reed_solomon_compute_divisor(blockecclen);
        let mut k = 0;

        for i in 0..numblocks {
            let datlen = shortblocklen - blockecclen + usize::from(i >= numshortblocks);
            let mut dat = data[k..k + datlen].to_vec();
            k += datlen;
            let ecc = reed_solomon_compute_remainder(&dat, &rsdiv);
            if i < numshortblocks {
                dat.push(0);
            }
            dat.extend_from_slice(&ecc);
            blocks.push(dat);
        }

        let mut result = Vec::with_capacity(rawcodewords);
        for i in 0..=shortblocklen {
            for (j, block) in blocks.iter().enumerate() {
                if i != shortblocklen - blockecclen || j >= numshortblocks {
                    result.push(block[i]);
                }
            }
        }
        result
    }

    fn draw_function_patterns(&mut self) {
        let size = self.size as i32;
        for i in 0..size {
            self.set_function_module(6, i, i % 2 == 0);
            self.set_function_module(i, 6, i % 2 == 0);
        }

        self.draw_finder_pattern(3, 3);
        self.draw_finder_pattern(size - 4, 3);
        self.draw_finder_pattern(3, size - 4);

        let align_positions = alignment_pattern_positions(self.version, self.size);
        let numalign = align_positions.len();
        for (i, &x) in align_positions.iter().enumerate() {
            for (j, &y) in align_positions.iter().enumerate() {
                if !(i == 0 && j == 0 || i == 0 && j == numalign - 1 || i == numalign - 1 && j == 0)
                {
                    self.draw_alignment_pattern(x as i32, y as i32);
                }
            }
        }

        self.draw_format_bits();
        self.draw_version();
    }

    fn draw_format_bits(&mut self) {
        let bits = compute_format_bits();

        for i in 0..6 {
            self.set_function_module(8, i, get_bit(bits, i as u32));
        }
        self.set_function_module(8, 7, get_bit(bits, 6));
        self.set_function_module(8, 8, get_bit(bits, 7));
        self.set_function_module(7, 8, get_bit(bits, 8));
        for i in 9..15 {
            self.set_function_module(14 - i, 8, get_bit(bits, i as u32));
        }

        let size = self.size as i32;
        for i in 0..8 {
            self.set_function_module(size - 1 - i, 8, get_bit(bits, i as u32));
        }
        for i in 8..15 {
            self.set_function_module(8, size - 15 + i, get_bit(bits, i as u32));
        }
        self.set_function_module(8, size - 8, true);
    }

    fn draw_version(&mut self) {
        if self.version < 7 {
            return;
        }
        let bits = compute_version_bits(self.version as u32);
        let size = self.size as i32;
        for i in 0..18 {
            let bit = get_bit(bits, i as u32);
            let a = size - 11 + (i % 3) as i32;
            let b = (i / 3) as i32;
            self.set_function_module(a, b, bit);
            self.set_function_module(b, a, bit);
        }
    }

    fn draw_finder_pattern(&mut self, x: i32, y: i32) {
        for dy in -4..=4 {
            for dx in -4..=4 {
                let xx = x + dx;
                let yy = y + dy;
                if (0..self.size as i32).contains(&xx) && (0..self.size as i32).contains(&yy) {
                    let dist = dx.abs().max(dy.abs());
                    self.set_function_module(xx, yy, dist != 2 && dist != 4);
                }
            }
        }
    }

    fn draw_alignment_pattern(&mut self, x: i32, y: i32) {
        for dy in -2..=2 {
            for dx in -2..=2 {
                self.set_function_module(x + dx, y + dy, dx.abs().max(dy.abs()) != 1);
            }
        }
    }

    fn set_function_module(&mut self, x: i32, y: i32, is_dark: bool) {
        let idx = (y * self.size as i32 + x) as usize;
        self.modules[idx] = is_dark;
        self.is_function[idx] = true;
    }

    fn draw_codewords(&mut self, data: &[u8]) {
        let total_bits = num_raw_data_modules(self.version);
        assert_eq!(data.len(), total_bits / 8);

        let size = self.size as i32;
        let mut i = 0usize;
        let mut right = size - 1;
        while right >= 1 {
            if right == 6 {
                right -= 1;
            }
            for vert in 0..size {
                for j in 0..2 {
                    let x = right - j;
                    let upward = ((right + 1) & 2) == 0;
                    let y = if upward { size - 1 - vert } else { vert };
                    let idx = (y * size + x) as usize;
                    if !self.is_function[idx] && i < data.len() * 8 {
                        self.modules[idx] = get_bit(data[i >> 3] as u32, 7 - (i as u32 & 7));
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
        debug_assert_eq!(i, data.len() * 8);
    }

    fn apply_mask(&mut self) {
        let size = self.size as i32;
        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) as usize;
                if !self.is_function[idx] && (x + y) % 2 == 0 {
                    self.modules[idx] = !self.modules[idx];
                }
            }
        }
    }
}

fn pick_version(data_len: usize) -> Result<usize, String> {
    for ver in 1..=40 {
        let cap_bytes = num_data_codewords(ver);
        let cap_bits = cap_bytes * 8;
        let char_count_bits = if ver <= 9 { 8 } else { 16 };
        let mut needed = 4 + char_count_bits + data_len * 8;
        if needed > cap_bits {
            continue;
        }
        let terminator = (cap_bits - needed).min(4);
        needed += terminator;
        let padding = (8 - needed % 8) % 8;
        needed += padding;
        if needed / 8 <= cap_bytes {
            return Ok(ver);
        }
    }
    Err("Données trop longues pour un QR code (limite de la norme atteinte)".to_string())
}

fn alignment_pattern_positions(version: usize, size: usize) -> Vec<usize> {
    if version == 1 {
        return vec![];
    }
    let num_align = version / 7 + 2;
    let step = (version * 8 + num_align * 3 + 5) / (num_align * 4 - 4) * 2;
    let mut positions: Vec<usize> = (0..num_align - 1).map(|i| size - 7 - i * step).collect();
    positions.push(6);
    positions.reverse();
    positions
}

fn compute_format_bits() -> u32 {
    let data = 0u32; // Medium ECC (0) and mask 0
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    (data << 10 | rem) ^ 0x5412
}

fn compute_version_bits(version: u32) -> u32 {
    let mut rem = version;
    for _ in 0..12 {
        rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
    }
    (version << 12) | rem
}

fn get_bit(data: u32, i: u32) -> bool {
    ((data >> i) & 1) != 0
}

fn num_raw_data_modules(ver: usize) -> usize {
    let mut result: usize = (16 * ver + 128) * ver + 64;
    if ver >= 2 {
        let num_align = ver / 7 + 2;
        result -= (25 * num_align - 10) * num_align - 55;
        if ver >= 7 {
            result -= 36;
        }
    }
    result
}

fn num_data_codewords(ver: usize) -> usize {
    num_raw_data_modules(ver) / 8 - ecc_codewords_per_block(ver) * num_error_correction_blocks(ver)
}

fn ecc_codewords_per_block(ver: usize) -> usize {
    MEDIUM_ECC_CODEWORDS_PER_BLOCK[ver] as usize
}

fn num_error_correction_blocks(ver: usize) -> usize {
    MEDIUM_BLOCK_COUNT[ver] as usize
}

fn reed_solomon_compute_divisor(degree: usize) -> Vec<u8> {
    assert!((1..=255).contains(&degree));
    let mut result = vec![0u8; degree - 1];
    result.push(1);
    let mut root: u8 = 1;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = reed_solomon_multiply(result[j], root);
            if j + 1 < result.len() {
                result[j] ^= result[j + 1];
            }
        }
        root = reed_solomon_multiply(root, 0x02);
    }
    result
}

fn reed_solomon_compute_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result.remove(0);
        result.push(0);
        for (x, &y) in result.iter_mut().zip(divisor.iter()) {
            *x ^= reed_solomon_multiply(y, factor);
        }
    }
    result
}

fn reed_solomon_multiply(x: u8, y: u8) -> u8 {
    let mut z: u8 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x1D);
        z ^= ((y >> i) & 1) * x;
    }
    z
}

struct BitBuffer {
    bits: Vec<bool>,
}

impl BitBuffer {
    fn new() -> Self {
        Self { bits: Vec::new() }
    }

    fn push_bit(&mut self, bit: bool) {
        self.bits.push(bit);
    }

    fn push_bits(&mut self, value: u32, length: u8) {
        for i in (0..length).rev() {
            self.bits.push(((value >> i) & 1) != 0);
        }
    }

    fn len(&self) -> usize {
        self.bits.len()
    }

    fn to_bytes(self) -> Vec<u8> {
        let len = self.bits.len();
        let mut out = Vec::with_capacity((len + 7) / 8);
        let mut acc = 0u8;
        for (i, bit) in self.bits.into_iter().enumerate() {
            acc = (acc << 1) | u8::from(bit);
            if i % 8 == 7 {
                out.push(acc);
                acc = 0;
            }
        }
        if len % 8 != 0 {
            acc <<= 8 - (len % 8);
            out.push(acc);
        }
        out
    }
}

const MEDIUM_ECC_CODEWORDS_PER_BLOCK: [i8; 41] = [
    -1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28,
    28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
];

const MEDIUM_BLOCK_COUNT: [i8; 41] = [
    -1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23,
    25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
];
