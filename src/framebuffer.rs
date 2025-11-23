// src/framebuffer.rs
use crate::core::math::Color;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            data: vec![Color::new(0.0, 0.0, 0.0); len],
        }
    }

    #[inline]
    pub fn clear(&mut self, color: Color) {
        self.data.fill(color);
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        self.data[idx] = color;
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Color {
        let idx = y * self.width + x;
        self.data[idx]
    }
}
