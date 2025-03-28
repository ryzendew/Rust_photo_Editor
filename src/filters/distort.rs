use image::{DynamicImage, Rgba, GenericImageView, ImageBuffer};
use crate::filters::Filter;
use std::f32::consts::PI;

/// Applies a wave distortion to the image
pub struct WaveFilter {
    pub amplitude: f32,
    pub wavelength: f32,
    pub direction: WaveDirection,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Copy)]
pub enum WaveDirection {
    Horizontal,
    Vertical,
    Both,
}

impl WaveFilter {
    pub fn new(amplitude: f32, wavelength: f32, direction: WaveDirection) -> Self {
        Self {
            amplitude,
            wavelength,
            direction,
            name: "Wave".to_string(),
            description: "Applies a wave distortion to the image".to_string(),
        }
    }
}

impl Filter for WaveFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Ensure we have valid parameters
        let amplitude = self.amplitude.max(0.0);
        let wavelength = self.wavelength.max(1.0);
        
        for y in 0..height {
            for x in 0..width {
                // Calculate source coordinates with wave distortion
                let mut src_x = x as f32;
                let mut src_y = y as f32;
                
                match self.direction {
                    WaveDirection::Horizontal => {
                        // Apply horizontal wave (distort y based on x)
                        src_y += amplitude * (2.0 * PI * src_x / wavelength).sin();
                    },
                    WaveDirection::Vertical => {
                        // Apply vertical wave (distort x based on y)
                        src_x += amplitude * (2.0 * PI * src_y / wavelength).sin();
                    },
                    WaveDirection::Both => {
                        // Apply both horizontal and vertical waves
                        src_y += amplitude * (2.0 * PI * src_x / wavelength).sin();
                        src_x += amplitude * (2.0 * PI * src_y / wavelength).sin();
                    }
                }
                
                // Ensure source coordinates are within bounds
                src_x = src_x.max(0.0).min(width as f32 - 1.0);
                src_y = src_y.max(0.0).min(height as f32 - 1.0);
                
                // Bilinear interpolation for smooth sampling
                let x0 = src_x.floor() as u32;
                let y0 = src_y.floor() as u32;
                let x1 = (x0 + 1).min(width - 1);
                let y1 = (y0 + 1).min(height - 1);
                
                let dx = src_x - x0 as f32;
                let dy = src_y - y0 as f32;
                
                let p00 = image.get_pixel(x0, y0);
                let p01 = image.get_pixel(x0, y1);
                let p10 = image.get_pixel(x1, y0);
                let p11 = image.get_pixel(x1, y1);
                
                // Interpolate each color channel
                let mut rgba = [0; 4];
                for i in 0..4 {
                    let top = p00.0[i] as f32 * (1.0 - dx) + p10.0[i] as f32 * dx;
                    let bottom = p01.0[i] as f32 * (1.0 - dx) + p11.0[i] as f32 * dx;
                    rgba[i] = (top * (1.0 - dy) + bottom * dy) as u8;
                }
                
                output.put_pixel(x, y, Rgba(rgba));
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
        Box::new(WaveFilter {
            amplitude: self.amplitude,
            wavelength: self.wavelength,
            direction: self.direction,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

/// Applies a whirl (swirl) distortion to the image
pub struct WhirlFilter {
    pub strength: f32,
    pub radius: f32,
    name: String,
    description: String,
}

impl WhirlFilter {
    pub fn new(strength: f32, radius: f32) -> Self {
        Self {
            strength,
            radius,
            name: "Whirl".to_string(),
            description: "Applies a whirl (swirl) distortion to the image".to_string(),
        }
    }
}

impl Filter for WhirlFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Calculate center point
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        
        // Ensure we have valid parameters
        let strength = self.strength.max(0.0).min(1.0);
        let radius = self.radius.max(1.0);
        
        for y in 0..height {
            for x in 0..width {
                // Calculate distance from center
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < radius {
                    // Calculate angle based on distance
                    let angle = strength * (1.0 - dist / radius) * 2.0 * PI;
                    
                    // Calculate new position
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();
                    let src_x = center_x + dx * cos_a - dy * sin_a;
                    let src_y = center_y + dx * sin_a + dy * cos_a;
                    
                    // Ensure source coordinates are within bounds
                    let src_x = src_x.max(0.0).min(width as f32 - 1.0);
                    let src_y = src_y.max(0.0).min(height as f32 - 1.0);
                    
                    // Bilinear interpolation for smooth sampling
                    let x0 = src_x.floor() as u32;
                    let y0 = src_y.floor() as u32;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);
                    
                    let dx = src_x - x0 as f32;
                    let dy = src_y - y0 as f32;
                    
                    let p00 = image.get_pixel(x0, y0);
                    let p01 = image.get_pixel(x0, y1);
                    let p10 = image.get_pixel(x1, y0);
                    let p11 = image.get_pixel(x1, y1);
                    
                    // Interpolate each color channel
                    let mut rgba = [0; 4];
                    for i in 0..4 {
                        let top = p00.0[i] as f32 * (1.0 - dx) + p10.0[i] as f32 * dx;
                        let bottom = p01.0[i] as f32 * (1.0 - dx) + p11.0[i] as f32 * dx;
                        rgba[i] = (top * (1.0 - dy) + bottom * dy) as u8;
                    }
                    
                    output.put_pixel(x, y, Rgba(rgba));
                } else {
                    // Outside the whirl radius, copy the original pixel
                    output.put_pixel(x, y, *image.get_pixel(x, y));
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
        Box::new(WhirlFilter {
            strength: self.strength,
            radius: self.radius,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

/// Applies a bulge or pinch distortion to the image
pub struct BulgeFilter {
    pub strength: f32,  // Positive for bulge, negative for pinch
    pub radius: f32,
    name: String,
    description: String,
}

impl BulgeFilter {
    pub fn new(strength: f32, radius: f32) -> Self {
        Self {
            strength,
            radius,
            name: "Bulge".to_string(),
            description: "Applies a bulge or pinch distortion to the image".to_string(),
        }
    }
}

impl Filter for BulgeFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Calculate center point
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        
        // Ensure we have valid parameters
        let strength = self.strength.max(-1.0).min(1.0);
        let radius = self.radius.max(1.0);
        
        for y in 0..height {
            for x in 0..width {
                // Calculate distance from center
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < radius {
                    // Calculate new distance based on strength
                    let factor = 1.0 - (dist / radius).powi(2);
                    let new_dist = dist * (1.0 + strength * factor);
                    
                    // Calculate new position
                    let angle = dy.atan2(dx);
                    let src_x = center_x + new_dist * angle.cos();
                    let src_y = center_y + new_dist * angle.sin();
                    
                    // Ensure source coordinates are within bounds
                    let src_x = src_x.max(0.0).min(width as f32 - 1.0);
                    let src_y = src_y.max(0.0).min(height as f32 - 1.0);
                    
                    // Bilinear interpolation for smooth sampling
                    let x0 = src_x.floor() as u32;
                    let y0 = src_y.floor() as u32;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);
                    
                    let dx = src_x - x0 as f32;
                    let dy = src_y - y0 as f32;
                    
                    let p00 = image.get_pixel(x0, y0);
                    let p01 = image.get_pixel(x0, y1);
                    let p10 = image.get_pixel(x1, y0);
                    let p11 = image.get_pixel(x1, y1);
                    
                    // Interpolate each color channel
                    let mut rgba = [0; 4];
                    for i in 0..4 {
                        let top = p00.0[i] as f32 * (1.0 - dx) + p10.0[i] as f32 * dx;
                        let bottom = p01.0[i] as f32 * (1.0 - dx) + p11.0[i] as f32 * dx;
                        rgba[i] = (top * (1.0 - dy) + bottom * dy) as u8;
                    }
                    
                    output.put_pixel(x, y, Rgba(rgba));
                } else {
                    // Outside the bulge radius, copy the original pixel
                    output.put_pixel(x, y, *image.get_pixel(x, y));
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
        Box::new(BulgeFilter {
            strength: self.strength,
            radius: self.radius,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
} 