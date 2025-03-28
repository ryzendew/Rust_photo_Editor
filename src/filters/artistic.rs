use image::{DynamicImage, Rgba, GenericImageView, ImageBuffer};
use crate::filters::Filter;
use imageproc::noise::{gaussian_noise_mut, salt_and_pepper_noise_mut};
use rand::distributions::Uniform;
use rand::{Rng, thread_rng};

/// Applies a pixelation (mosaic) effect to the image
pub struct PixelateFilter {
    pub block_size: u32,
    name: String,
    description: String,
}

impl PixelateFilter {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size,
            name: "Pixelate".to_string(),
            description: "Creates a pixelated effect by averaging blocks of pixels".to_string(),
        }
    }
}

impl Filter for PixelateFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in (0..height).step_by(self.block_size as usize) {
            for x in (0..width).step_by(self.block_size as usize) {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;
                
                // Calculate average color for block
                for by in y..std::cmp::min(y + self.block_size, height) {
                    for bx in x..std::cmp::min(x + self.block_size, width) {
                        let pixel = image.get_pixel(bx, by);
                        r_sum += pixel[0] as u32;
                        g_sum += pixel[1] as u32;
                        b_sum += pixel[2] as u32;
                        a_sum += pixel[3] as u32;
                        count += 1;
                    }
                }
                
                let r = (r_sum / count) as u8;
                let g = (g_sum / count) as u8;
                let b = (b_sum / count) as u8;
                let a = (a_sum / count) as u8;
                
                // Fill block with average color
                for by in y..std::cmp::min(y + self.block_size, height) {
                    for bx in x..std::cmp::min(x + self.block_size, width) {
                        output.put_pixel(bx, by, Rgba([r, g, b, a]));
                    }
                }
            }
        }
        
        output
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(PixelateFilter {
            block_size: self.block_size,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

/// Applies Oil Painting effect
pub struct OilPaintingFilter {
    pub radius: u32,
    pub intensity_levels: u32,
    name: String,
    description: String,
}

impl OilPaintingFilter {
    pub fn new(radius: u32, intensity_levels: u32) -> Self {
        Self {
            radius,
            intensity_levels,
            name: "Oil Painting".to_string(),
            description: "Creates an oil painting effect by analyzing color intensities".to_string(),
        }
    }
}

impl Filter for OilPaintingFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in 0..height {
            for x in 0..width {
                let mut intensity_count = vec![0u32; self.intensity_levels as usize];
                let mut r_sum = vec![0u32; self.intensity_levels as usize];
                let mut g_sum = vec![0u32; self.intensity_levels as usize];
                let mut b_sum = vec![0u32; self.intensity_levels as usize];
                let mut a_sum = vec![0u32; self.intensity_levels as usize];
                
                // Analyze neighborhood
                for ny in y.saturating_sub(self.radius)..std::cmp::min(y + self.radius + 1, height) {
                    for nx in x.saturating_sub(self.radius)..std::cmp::min(x + self.radius + 1, width) {
                        let pixel = image.get_pixel(nx, ny);
                        let intensity = ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as usize;
                        let intensity_level = (intensity * (self.intensity_levels as usize - 1) / 255) as usize;
                        
                        intensity_count[intensity_level] += 1;
                        r_sum[intensity_level] += pixel[0] as u32;
                        g_sum[intensity_level] += pixel[1] as u32;
                        b_sum[intensity_level] += pixel[2] as u32;
                        a_sum[intensity_level] += pixel[3] as u32;
                    }
                }
                
                // Find most frequent intensity level
                let mut max_count = 0;
                let mut max_intensity = 0;
                for i in 0..self.intensity_levels as usize {
                    if intensity_count[i] > max_count {
                        max_count = intensity_count[i];
                        max_intensity = i;
                    }
                }
                
                if max_count > 0 {
                    let r = (r_sum[max_intensity] / max_count) as u8;
                    let g = (g_sum[max_intensity] / max_count) as u8;
                    let b = (b_sum[max_intensity] / max_count) as u8;
                    let a = (a_sum[max_intensity] / max_count) as u8;
                    output.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }
        
        output
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(OilPaintingFilter {
            radius: self.radius,
            intensity_levels: self.intensity_levels,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

/// Applies a cartoon-like effect
pub struct CartoonFilter {
    pub edge_threshold: f32,
    pub color_levels: u32,
    name: String,
    description: String,
}

impl CartoonFilter {
    pub fn new(edge_threshold: f32, color_levels: u32) -> Self {
        Self {
            edge_threshold,
            color_levels,
            name: "Cartoon".to_string(),
            description: "Creates a cartoon effect by combining edge detection with color quantization".to_string(),
        }
    }
}

impl Filter for CartoonFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        // First pass: detect edges using Sobel operator
        let mut edge_map = ImageBuffer::new(width, height);
        for y in 1..height-1 {
            for x in 1..width-1 {
                let gx = self.sobel_x(image, x, y);
                let gy = self.sobel_y(image, x, y);
                let edge_strength = ((gx * gx + gy * gy) as f32).sqrt();
                let is_edge = if edge_strength > self.edge_threshold { 0 } else { 255 };
                edge_map.put_pixel(x, y, Rgba([is_edge, is_edge, is_edge, 255]));
            }
        }
        
        // Second pass: quantize colors
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let edge_pixel = edge_map.get_pixel(x, y);
                
                let r = self.quantize(pixel[0], self.color_levels);
                let g = self.quantize(pixel[1], self.color_levels);
                let b = self.quantize(pixel[2], self.color_levels);
                
                // If this is an edge pixel, darken it
                let edge_factor = edge_pixel[0] as f32 / 255.0;
                let r = (r as f32 * edge_factor) as u8;
                let g = (g as f32 * edge_factor) as u8;
                let b = (b as f32 * edge_factor) as u8;
                
                output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
            }
        }
        
        output
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(CartoonFilter {
            edge_threshold: self.edge_threshold,
            color_levels: self.color_levels,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

impl CartoonFilter {
    fn sobel_x(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32) -> i32 {
        let p00 = self.intensity(image.get_pixel(x-1, y-1));
        let p01 = self.intensity(image.get_pixel(x-1, y));
        let p02 = self.intensity(image.get_pixel(x-1, y+1));
        let p20 = self.intensity(image.get_pixel(x+1, y-1));
        let p21 = self.intensity(image.get_pixel(x+1, y));
        let p22 = self.intensity(image.get_pixel(x+1, y+1));
        
        -p00 - 2*p01 - p02 + p20 + 2*p21 + p22
    }
    
    fn sobel_y(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32) -> i32 {
        let p00 = self.intensity(image.get_pixel(x-1, y-1));
        let p10 = self.intensity(image.get_pixel(x, y-1));
        let p20 = self.intensity(image.get_pixel(x+1, y-1));
        let p02 = self.intensity(image.get_pixel(x-1, y+1));
        let p12 = self.intensity(image.get_pixel(x, y+1));
        let p22 = self.intensity(image.get_pixel(x+1, y+1));
        
        -p00 - 2*p10 - p20 + p02 + 2*p12 + p22
    }
    
    fn intensity(&self, pixel: &Rgba<u8>) -> i32 {
        ((pixel[0] as i32 + pixel[1] as i32 + pixel[2] as i32) / 3) as i32
    }
    
    fn quantize(&self, value: u8, levels: u32) -> u8 {
        let step = 256u32 / levels;
        let level = (value as u32 / step) * step;
        level.min(255) as u8
    }
}

/// Applies noise to an image
pub struct NoiseFilter {
    pub noise_type: NoiseType,
    pub amount: f32,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Copy)]
pub enum NoiseType {
    Gaussian,
    SaltAndPepper,
    Speckle,
}

impl NoiseFilter {
    pub fn new(noise_type: NoiseType, amount: f32) -> Self {
        Self {
            noise_type,
            amount,
            name: "Noise".to_string(),
            description: "Applies various types of noise to the image".to_string(),
        }
    }
}

impl Filter for NoiseFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut output = image.clone();
        let (width, height) = output.dimensions();
        let mut rng = thread_rng();
        
        match self.noise_type {
            NoiseType::Gaussian => {
                let stddev = self.amount * 50.0;
                gaussian_noise_mut(&mut output, 0.0, stddev as f64, 42);
            }
            NoiseType::SaltAndPepper => {
                let probability = self.amount * 0.1;
                salt_and_pepper_noise_mut(&mut output, probability as f64, 42);
            }
            NoiseType::Speckle => {
                let stddev = self.amount * 30.0;
                for y in 0..height {
                    for x in 0..width {
                        let pixel = output.get_pixel(x, y);
                        let noise = rng.gen::<f32>() * stddev;
                        let r = (pixel[0] as f32 * (1.0 + noise)).clamp(0.0, 255.0) as u8;
                        let g = (pixel[1] as f32 * (1.0 + noise)).clamp(0.0, 255.0) as u8;
                        let b = (pixel[2] as f32 * (1.0 + noise)).clamp(0.0, 255.0) as u8;
                        output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
                    }
                }
            }
        }
        
        output
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(NoiseFilter {
            noise_type: self.noise_type,
            amount: self.amount,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
} 