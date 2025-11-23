

use std::fs::File;
use std::io::{BufWriter, Write};

use crate::core::math::Color;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub data: Vec<Color>, 
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
    let h = img.h as i32; 
    let row_stride = ((w as usize * 3 + 3) / 4) * 4; // múltiplo de 4
    let img_size = row_stride * (h as usize);
    let file_size = 14 + 40 + img_size;

    let mut f = BufWriter::new(File::create(path)?);

  
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    let pixel_offset: u32 = 54;
    f.write_all(&pixel_offset.to_le_bytes())?;

    let dib_size: u32 = 40;
    f.write_all(&dib_size.to_le_bytes())?;    
    f.write_all(&w.to_le_bytes())?;          
    f.write_all(&h.to_le_bytes())?;           
    f.write_all(&(1u16).to_le_bytes())?;      
    f.write_all(&(24u16).to_le_bytes())?;     
    f.write_all(&0u32.to_le_bytes())?;        
    f.write_all(&(img_size as u32).to_le_bytes())?; 
    f.write_all(&2835u32.to_le_bytes())?;     
    f.write_all(&2835u32.to_le_bytes())?;    
    f.write_all(&0u32.to_le_bytes())?;        
    f.write_all(&0u32.to_le_bytes())?;        

    
    let mut row = vec![0u8; row_stride];
    for y in 0..(h as usize) {
        let sy = (h as usize - 1) - y;
        let mut pos = 0;
        for x in 0..(w as usize) {
            let c = img.get(x, sy);
            row[pos] = f2u8(c.z);     // B
            row[pos + 1] = f2u8(c.y); // G
            row[pos + 2] = f2u8(c.x); // R
            pos += 3;
        }
        
        f.write_all(&row)?;
    }

    f.flush()?;
    Ok(())
}
