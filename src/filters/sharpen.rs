use image::{ImageBuffer, Rgba};
use imageproc::filter::{gaussian_blur_f32};
use crate::filters::{Filter, IntensityFilter};

/// Unsharp mask filter
#[derive(Clone)]
pub struct UnsharpMask {
    /// Radius of the blur used for the mask
    pub radius: f32,
    /// Amount of sharpening to apply (0.0 to 10.0)
    pub amount: f32,
    /// Threshold below which differences are not sharpened (0-255)
    pub threshold: u8,
    name: String,
    description: String,
}

impl UnsharpMask {
    /// Create a new unsharp mask filter
    pub fn new(radius: f32, amount: f32, threshold: u8) -> Self {
        Self {
            radius: radius.max(0.1),
            amount: amount.max(0.0).min(10.0),
            threshold,
            name: "Unsharp Mask".to_string(),
            description: "Sharpens an image by subtracting a blurred version".to_string(),
        }
    }
}

impl Filter for UnsharpMask {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let width = image.width();
        let height = image.height();
        let mut result = image.clone();
        
        // Create a blurred version of the image
        let blurred = gaussian_blur_f32(image, self.radius);
        
        // For each pixel in the image
        for y in 0..height {
            for x in 0..width {
                let original = image.get_pixel(x, y);
                let blur = blurred.get_pixel(x, y);
                
                // For each color channel
                let mut new_pixel = [0u8; 4];
                
                for c in 0..3 {  // Only process RGB, preserve alpha
                    let orig_val = original[c] as i32;
                    let blur_val = blur[c] as i32;
                    let diff = orig_val - blur_val;
                    
                    // Only sharpen if the difference is greater than the threshold
                    if diff.abs() > self.threshold as i32 {
                        // Apply the unsharp mask formula: original + amount * (original - blurred)
                        let sharpened = orig_val + (self.amount * diff as f32) as i32;
                        new_pixel[c] = sharpened.clamp(0, 255) as u8;
                    } else {
                        new_pixel[c] = original[c];
                    }
                }
                
                // Preserve alpha
                new_pixel[3] = original[3];
                
                result.put_pixel(x, y, Rgba(new_pixel));
            }
        }
        
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(self.clone())
    }
}

impl IntensityFilter for UnsharpMask {
    fn set_intensity(&mut self, intensity: f32) {
        self.amount = intensity.max(0.0).min(10.0);
    }
    
    fn intensity(&self) -> f32 {
        self.amount
    }
}

/// High pass filter
#[derive(Clone)]
pub struct HighPass {
    /// Radius of the filter
    pub radius: f32,
    /// Amount of filtering to apply (0.0 to 1.0)
    pub amount: f32,
    /// Whether to preserve the original brightness
    pub preserve_brightness: bool,
    name: String,
    description: String,
}

impl HighPass {
    /// Create a new high pass filter
    pub fn new(radius: f32, amount: f32, preserve_brightness: bool) -> Self {
        Self {
            radius: radius.max(0.1),
            amount: amount.max(0.0).min(1.0),
            preserve_brightness,
            name: "High Pass".to_string(),
            description: "Enhances edges while reducing low frequency details".to_string(),
        }
    }
}

impl Filter for HighPass {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let width = image.width();
        let height = image.height();
        let mut result = ImageBuffer::new(width, height);
        
        // Create a blurred version of the image (low-pass)
        let blurred = gaussian_blur_f32(image, self.radius);
        
        // For each pixel in the image
        for y in 0..height {
            for x in 0..width {
                let original = image.get_pixel(x, y);
                let blur = blurred.get_pixel(x, y);
                
                let mut new_pixel = [0u8; 4];
                
                if self.preserve_brightness {
                    // Calculate average brightness of original pixel
                    let avg_brightness = (
                        original[0] as f32 + 
                        original[1] as f32 + 
                        original[2] as f32
                    ) / 3.0;
                    
                    // For each color channel (except alpha)
                    for c in 0..3 {
                        // High pass = original - blurred (low pass)
                        let high_pass_val = 
                            (original[c] as f32 - blur[c] as f32) * self.amount + 128.0;
                            
                        // Preserve the original brightness
                        let normalized = high_pass_val - 128.0; // Center around 0
                        let result_val = avg_brightness + normalized;
                        
                        new_pixel[c] = result_val.round().clamp(0.0, 255.0) as u8;
                    }
                } else {
                    // Standard high pass implementation
                    for c in 0..3 {
                        // High pass = original - blurred (low pass)
                        let high_pass_val = 
                            (original[c] as f32 - blur[c] as f32) * self.amount + 128.0;
                        
                        new_pixel[c] = high_pass_val.round().clamp(0.0, 255.0) as u8;
                    }
                }
                
                // Preserve alpha
                new_pixel[3] = original[3];
                
                result.put_pixel(x, y, Rgba(new_pixel));
            }
        }
        
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(self.clone())
    }
}

impl IntensityFilter for HighPass {
    fn set_intensity(&mut self, intensity: f32) {
        self.amount = intensity.max(0.0).min(1.0);
    }
    
    fn intensity(&self) -> f32 {
        self.amount
    }
}

/// Sharpen filter
#[derive(Clone)]
pub struct Sharpen {
    /// Amount of sharpening to apply (0.0 to 10.0)
    pub amount: f32,
    name: String,
    description: String,
}

impl Sharpen {
    /// Create a new sharpen filter
    pub fn new(amount: f32) -> Self {
        Self {
            amount: amount.max(0.0).min(10.0),
            name: "Sharpen".to_string(),
            description: "Sharpens an image using a convolution kernel".to_string(),
        }
    }
}

impl Filter for Sharpen {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let width = image.width();
        let height = image.height();
        let mut result = ImageBuffer::new(width, height);
        
        // Create a sharpening kernel based on the amount
        // [0, -1, 0]
        // [-1, 4+amount, -1]
        // [0, -1, 0]
        let center_weight = 4.0 + self.amount;
        let edge_weight = -1.0;
        
        // Apply the convolution
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                
                // Apply the kernel
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                        let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                        
                        let pixel = image.get_pixel(nx, ny);
                        
                        // Determine the weight based on position
                        let weight = if dx == 0 && dy == 0 {
                            center_weight
                        } else if (dx == 0 && dy.abs() == 1) || (dy == 0 && dx.abs() == 1) {
                            edge_weight
                        } else {
                            0.0 // Corners have zero weight
                        };
                        
                        r_sum += pixel[0] as f32 * weight;
                        g_sum += pixel[1] as f32 * weight;
                        b_sum += pixel[2] as f32 * weight;
                    }
                }
                
                // Normalize the result
                let r = r_sum.round().clamp(0.0, 255.0) as u8;
                let g = g_sum.round().clamp(0.0, 255.0) as u8;
                let b = b_sum.round().clamp(0.0, 255.0) as u8;
                
                // Preserve the alpha channel
                let a = image.get_pixel(x, y)[3];
                
                result.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
        
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(self.clone())
    }
}

impl IntensityFilter for Sharpen {
    fn set_intensity(&mut self, intensity: f32) {
        self.amount = intensity.max(0.0).min(10.0);
    }
    
    fn intensity(&self) -> f32 {
        self.amount
    }
} 