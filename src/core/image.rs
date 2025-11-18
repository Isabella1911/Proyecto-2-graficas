// src/core/image.rs

use std::fs::File;
use std::io::{BufWriter, Write};

use crate::core::vec3::Color;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub data: Vec<Color>, // RGB en float [0, +inf), se clamp a [0,1] al guardar
}

impl Image {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            data: vec![Color::new(0.0, 0.0, 0.0); w * h],
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, c: Color) {
        if x < self.w && y < self.h {
            self.data[y * self.w + x] = c;
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Color {
        self.data[y * self.w + x]
    }

    /// Guarda como BMP 24-bit (BGR), **bottom-up** con padding de filas a múltiplos de 4 bytes.
    pub fn save_bmp(&self, path: &str) {
        save_bmp24(self, path).expect("No se pudo escribir el BMP");
    }
}

#[inline]
fn f2u8(v: f64) -> u8 {
    let c = if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v };
    (c * 255.0 + 0.5).floor() as u8
}

fn save_bmp24(img: &Image, path: &str) -> std::io::Result<()> {
    let w = img.w as u32;
    let h = img.h as i32; // positivo => bottom-up
    let row_stride = ((w as usize * 3 + 3) / 4) * 4; // múltiplo de 4
    let img_size = row_stride * (h as usize);
    let file_size = 14 + 40 + img_size;

    let mut f = BufWriter::new(File::create(path)?);

    // --- File header (14 bytes) ---
    // Signature "BM"
    f.write_all(b"BM")?;
    // File size (u32 LE)
    f.write_all(&(file_size as u32).to_le_bytes())?;
    // Reserved1 + Reserved2
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    // Pixel data offset = 14 + 40
    let pixel_offset: u32 = 54;
    f.write_all(&pixel_offset.to_le_bytes())?;

    // --- DIB header BITMAPINFOHEADER (40 bytes) ---
    let dib_size: u32 = 40;
    f.write_all(&dib_size.to_le_bytes())?;    // header size
    f.write_all(&w.to_le_bytes())?;           // width
    f.write_all(&h.to_le_bytes())?;           // height (positivo = bottom-up)
    f.write_all(&(1u16).to_le_bytes())?;      // planes
    f.write_all(&(24u16).to_le_bytes())?;     // bpp = 24
    f.write_all(&0u32.to_le_bytes())?;        // compression = BI_RGB (0)
    f.write_all(&(img_size as u32).to_le_bytes())?; // image size
    f.write_all(&2835u32.to_le_bytes())?;     // X ppm (~72 dpi => 2835)
    f.write_all(&2835u32.to_le_bytes())?;     // Y ppm
    f.write_all(&0u32.to_le_bytes())?;        // colors in palette
    f.write_all(&0u32.to_le_bytes())?;        // important colors

    // --- Pixel data (bottom-up, BGR, con padding) ---
    let mut row = vec![0u8; row_stride];
    for y in 0..(h as usize) {
        // bottom-up => fila src = (h-1 - y)
        let sy = (h as usize - 1) - y;
        let mut pos = 0;
        for x in 0..(w as usize) {
            let c = img.get(x, sy);
            // almacenamos BGR
            row[pos] = f2u8(c.z);     // B
            row[pos + 1] = f2u8(c.y); // G
            row[pos + 2] = f2u8(c.x); // R
            pos += 3;
        }
        // padding ya está en 0
        f.write_all(&row)?;
    }

    f.flush()?;
    Ok(())
}
