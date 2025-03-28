use image::{DynamicImage, Rgba, GenericImageView, ImageBuffer};
use crate::filters::Filter;

/// Color filters for adjusting brightness, contrast, and other color attributes
pub struct BrightnessFilter {
    pub amount: f32,
    name: String,
    description: String,
}

impl BrightnessFilter {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            name: format!("Brightness ({:+.1})", amount),
            description: format!("Adjusts image brightness by {}", amount),
        }
    }
}

impl Filter for BrightnessFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let r = (pixel[0] as f32 + self.amount * 255.0).clamp(0.0, 255.0) as u8;
                let g = (pixel[1] as f32 + self.amount * 255.0).clamp(0.0, 255.0) as u8;
                let b = (pixel[2] as f32 + self.amount * 255.0).clamp(0.0, 255.0) as u8;
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
        Box::new(Self {
            amount: self.amount,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

pub struct ContrastFilter {
    pub amount: f32,
    name: String,
    description: String,
}

impl ContrastFilter {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            name: format!("Contrast ({:+.1})", amount),
            description: format!("Adjusts image contrast by {}", amount),
        }
    }
}

impl Filter for ContrastFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let r = ((pixel[0] as f32 - 128.0) * self.amount + 128.0).clamp(0.0, 255.0) as u8;
                let g = ((pixel[1] as f32 - 128.0) * self.amount + 128.0).clamp(0.0, 255.0) as u8;
                let b = ((pixel[2] as f32 - 128.0) * self.amount + 128.0).clamp(0.0, 255.0) as u8;
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
        Box::new(Self {
            amount: self.amount,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

pub struct SaturationFilter {
    pub amount: f32,
    name: String,
    description: String,
}

impl SaturationFilter {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            name: format!("Saturation ({:+.1})", amount),
            description: format!("Adjusts image saturation by {}", amount),
        }
    }
}

impl Filter for SaturationFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let gray = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
                let r = (gray + (pixel[0] as f32 - gray) * self.amount).clamp(0.0, 255.0) as u8;
                let g = (gray + (pixel[1] as f32 - gray) * self.amount).clamp(0.0, 255.0) as u8;
                let b = (gray + (pixel[2] as f32 - gray) * self.amount).clamp(0.0, 255.0) as u8;
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
        Box::new(Self {
            amount: self.amount,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

pub struct HueFilter {
    pub amount: f32,
    name: String,
    description: String,
}

impl HueFilter {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            name: format!("Hue ({:+.1}°)", amount),
            description: format!("Adjusts image hue by {} degrees", amount),
        }
    }
}

impl Filter for HueFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        let angle = self.amount * std::f32::consts::PI / 180.0;
        let cos_h = angle.cos();
        let sin_h = angle.sin();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                
                let matrix = [
                    0.213 + cos_h * 0.787 - sin_h * 0.213,
                    0.213 - cos_h * 0.213 + sin_h * 0.143,
                    0.213 - cos_h * 0.213 - sin_h * 0.787,
                    
                    0.715 - cos_h * 0.715 - sin_h * 0.715,
                    0.715 + cos_h * 0.285 + sin_h * 0.140,
                    0.715 - cos_h * 0.715 + sin_h * 0.715,
                    
                    0.072 - cos_h * 0.072 + sin_h * 0.928,
                    0.072 - cos_h * 0.072 - sin_h * 0.283,
                    0.072 + cos_h * 0.928 + sin_h * 0.072
                ];
                
                let new_r = (matrix[0] * r + matrix[1] * g + matrix[2] * b) * 255.0;
                let new_g = (matrix[3] * r + matrix[4] * g + matrix[5] * b) * 255.0;
                let new_b = (matrix[6] * r + matrix[7] * g + matrix[8] * b) * 255.0;
                
                output.put_pixel(x, y, Rgba([
                    new_r.clamp(0.0, 255.0) as u8,
                    new_g.clamp(0.0, 255.0) as u8,
                    new_b.clamp(0.0, 255.0) as u8,
                    pixel[3]
                ]));
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
        Box::new(Self {
            amount: self.amount,
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}

pub struct InvertFilter {
    name: String,
    description: String,
}

impl InvertFilter {
    pub fn new() -> Self {
        Self {
            name: "Invert".to_string(),
            description: "Inverts the colors of the image".to_string(),
        }
    }
}

impl Filter for InvertFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = image.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                output.put_pixel(x, y, Rgba([
                    255 - pixel[0],
                    255 - pixel[1],
                    255 - pixel[2],
                    pixel[3]
                ]));
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
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
}