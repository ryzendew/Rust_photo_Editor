use image::{ImageBuffer, Rgba};
use imageproc::filter::{gaussian_blur_f32, box_filter};
use std::f32::consts::PI;
use crate::filters::Filter;
use log::{debug, info, trace, warn};

/// Gaussian blur filter
#[derive(Clone)]
pub struct GaussianBlur {
    /// The radius of the blur (standard deviation)
    pub radius: f32,
    name: String,
    description: String,
}

impl GaussianBlur {
    /// Create a new Gaussian blur filter with the specified radius
    pub fn new(radius: f32) -> Self {
        let radius = radius.max(0.1); // Ensure minimum radius
        info!("Creating new Gaussian blur filter with radius {}", radius);
        Self {
            radius,
            name: "Gaussian Blur".to_string(),
            description: "Applies a Gaussian blur to the image".to_string(),
        }
    }
}

impl Filter for GaussianBlur {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        debug!("Applying Gaussian blur with radius {} to {}x{} image", 
               self.radius, image.width(), image.height());
        
        let start_time = std::time::Instant::now();
        let result = gaussian_blur_f32(image, self.radius);
        let duration = start_time.elapsed();
        
        debug!("Gaussian blur completed in {:.2?}", duration);
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        trace!("Cloning Gaussian blur filter");
        Box::new(self.clone())
    }
}

/// Box blur filter
#[derive(Clone)]
pub struct BoxBlur {
    /// The radius of the blur
    pub radius: u32,
    name: String,
    description: String,
}

impl BoxBlur {
    /// Create a new box blur filter with the specified radius
    pub fn new(radius: u32) -> Self {
        let radius = radius.max(1); // Ensure minimum radius
        info!("Creating new Box blur filter with radius {}", radius);
        Self {
            radius,
            name: "Box Blur".to_string(),
            description: "Applies a box blur to the image".to_string(),
        }
    }
}

impl Filter for BoxBlur {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Convert to grayscale for blur
        let mut gray = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let gray_val = ((pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0) as u8;
                gray.put_pixel(x, y, Luma([gray_val]));
            }
        }
        
        // Apply box blur to grayscale image
        let blurred = box_filter(&gray, self.radius, self.radius);
        
        // Convert back to RGBA, preserving original alpha channel
        for y in 0..height {
            for x in 0..width {
                let gray_val = blurred.get_pixel(x, y)[0];
                let alpha = image.get_pixel(x, y)[3];
                output.put_pixel(x, y, Rgba([gray_val, gray_val, gray_val, alpha]));
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
        trace!("Cloning Box blur filter");
        Box::new(self.clone())
    }
}

/// Motion blur filter
#[derive(Clone)]
pub struct MotionBlur {
    /// The angle of the motion blur in degrees
    pub angle: f32,
    /// The distance of the motion blur
    pub distance: u32,
    name: String,
    description: String,
}

impl MotionBlur {
    /// Create a new motion blur filter with the specified angle and distance
    pub fn new(angle: f32, distance: u32) -> Self {
        let distance = distance.max(1); // Ensure minimum distance
        info!("Creating new Motion blur filter with angle {}° and distance {}", angle, distance);
        Self {
            angle,
            distance,
            name: "Motion Blur".to_string(),
            description: "Applies a directional motion blur to the image".to_string(),
        }
    }
}

impl Filter for MotionBlur {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        debug!("Applying Motion blur with angle {}° and distance {} to {}x{} image", 
               self.angle, self.distance, image.width(), image.height());
        
        let start_time = std::time::Instant::now();
        
        let width = image.width();
        let height = image.height();
        let mut result = ImageBuffer::new(width, height);
        
        // Convert angle to radians
        let angle_rad = self.angle * PI / 180.0;
        
        // Calculate the delta x and y for each step
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();
        trace!("Motion vector: dx={:.4}, dy={:.4}", dx, dy);
        
        // For each pixel in the output image
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut a_sum = 0.0;
                let mut weight_sum = 0.0;
                
                // Sample along the motion path
                for i in 0..=self.distance {
                    let t = (i as f32 - self.distance as f32 / 2.0) / self.distance as f32;
                    let sample_x = x as f32 + t * dx * self.distance as f32;
                    let sample_y = y as f32 + t * dy * self.distance as f32;
                    
                    // Only sample within image bounds
                    if sample_x >= 0.0 && sample_x < width as f32 &&
                       sample_y >= 0.0 && sample_y < height as f32 {
                        // Bilinear interpolation
                        let x0 = sample_x.floor() as u32;
                        let y0 = sample_y.floor() as u32;
                        let x1 = (x0 + 1).min(width - 1);
                        let y1 = (y0 + 1).min(height - 1);
                        
                        let dx = sample_x - x0 as f32;
                        let dy = sample_y - y0 as f32;
                        
                        let w00 = (1.0 - dx) * (1.0 - dy);
                        let w01 = (1.0 - dx) * dy;
                        let w10 = dx * (1.0 - dy);
                        let w11 = dx * dy;
                        
                        let p00 = image.get_pixel(x0, y0);
                        let p01 = image.get_pixel(x0, y1);
                        let p10 = image.get_pixel(x1, y0);
                        let p11 = image.get_pixel(x1, y1);
                        
                        let r = (p00[0] as f32 * w00 + p01[0] as f32 * w01 + 
                                 p10[0] as f32 * w10 + p11[0] as f32 * w11);
                        let g = (p00[1] as f32 * w00 + p01[1] as f32 * w01 + 
                                 p10[1] as f32 * w10 + p11[1] as f32 * w11);
                        let b = (p00[2] as f32 * w00 + p01[2] as f32 * w01 + 
                                 p10[2] as f32 * w10 + p11[2] as f32 * w11);
                        let a = (p00[3] as f32 * w00 + p01[3] as f32 * w01 + 
                                 p10[3] as f32 * w10 + p11[3] as f32 * w11);
                        
                        let weight = 1.0;
                        
                        r_sum += r * weight;
                        g_sum += g * weight;
                        b_sum += b * weight;
                        a_sum += a * weight;
                        weight_sum += weight;
                    }
                }
                
                // Normalize and set the result
                if weight_sum > 0.0 {
                    let r = (r_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let g = (g_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let b = (b_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let a = (a_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    
                    result.put_pixel(x, y, Rgba([r, g, b, a]));
                } else {
                    // If no samples were taken, copy the original pixel
                    trace!("No samples for pixel ({}, {}), using original", x, y);
                    result.put_pixel(x, y, *image.get_pixel(x, y));
                }
            }
        }
        
        let duration = start_time.elapsed();
        debug!("Motion blur completed in {:.2?}", duration);
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        trace!("Cloning Motion blur filter");
        Box::new(self.clone())
    }
}

/// Radial blur filter
#[derive(Clone)]
pub struct RadialBlur {
    /// The center x-coordinate of the blur
    pub center_x: f32,
    /// The center y-coordinate of the blur
    pub center_y: f32,
    /// The amount of blur
    pub amount: f32,
    name: String,
    description: String,
}

impl RadialBlur {
    /// Create a new radial blur filter
    pub fn new(center_x: f32, center_y: f32, amount: f32) -> Self {
        let amount = amount.max(0.0).min(1.0); // Clamp amount to 0.0-1.0
        info!("Creating new Radial blur filter at position ({}, {}) with amount {}", 
              center_x, center_y, amount);
        
        Self {
            center_x,
            center_y,
            amount,
            name: "Radial Blur".to_string(),
            description: "Applies a radial blur effect to the image".to_string(),
        }
    }
}

impl Filter for RadialBlur {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        debug!("Applying Radial blur at ({}, {}) with amount {} to {}x{} image", 
               self.center_x, self.center_y, self.amount, image.width(), image.height());
        
        let start_time = std::time::Instant::now();
        
        let width = image.width();
        let height = image.height();
        let mut result = ImageBuffer::new(width, height);
        
        // Normalize center coordinates to 0.0-1.0
        let center_x = self.center_x.clamp(0.0, 1.0) * width as f32;
        let center_y = self.center_y.clamp(0.0, 1.0) * height as f32;
        trace!("Normalized center coordinates: ({}, {})", center_x, center_y);
        
        let num_samples = 10; // Number of samples to take
        let max_offset = self.amount * 20.0; // Maximum offset for sampling
        trace!("Using {} samples with max offset {:.2}", num_samples, max_offset);
        
        // For each pixel in the output image
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut a_sum = 0.0;
                let mut weight_sum = 0.0;
                
                // Calculate vector from center to current pixel
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                // Normalize the vector
                let length = (dx * dx + dy * dy).sqrt();
                let norm_dx = if length > 0.0 { dx / length } else { 0.0 };
                let norm_dy = if length > 0.0 { dy / length } else { 0.0 };
                
                // Sample along the radial path
                for i in 0..=num_samples {
                    let t = (i as f32 / num_samples as f32) * max_offset;
                    let sample_x = x as f32 - norm_dx * t;
                    let sample_y = y as f32 - norm_dy * t;
                    
                    // Only sample within image bounds
                    if sample_x >= 0.0 && sample_x < width as f32 &&
                       sample_y >= 0.0 && sample_y < height as f32 {
                        // Bilinear interpolation
                        let x0 = sample_x.floor() as u32;
                        let y0 = sample_y.floor() as u32;
                        let x1 = (x0 + 1).min(width - 1);
                        let y1 = (y0 + 1).min(height - 1);
                        
                        let dx = sample_x - x0 as f32;
                        let dy = sample_y - y0 as f32;
                        
                        let w00 = (1.0 - dx) * (1.0 - dy);
                        let w01 = (1.0 - dx) * dy;
                        let w10 = dx * (1.0 - dy);
                        let w11 = dx * dy;
                        
                        let p00 = image.get_pixel(x0, y0);
                        let p01 = image.get_pixel(x0, y1);
                        let p10 = image.get_pixel(x1, y0);
                        let p11 = image.get_pixel(x1, y1);
                        
                        let r = (p00[0] as f32 * w00 + p01[0] as f32 * w01 + 
                                 p10[0] as f32 * w10 + p11[0] as f32 * w11);
                        let g = (p00[1] as f32 * w00 + p01[1] as f32 * w01 + 
                                 p10[1] as f32 * w10 + p11[1] as f32 * w11);
                        let b = (p00[2] as f32 * w00 + p01[2] as f32 * w01 + 
                                 p10[2] as f32 * w10 + p11[2] as f32 * w11);
                        let a = (p00[3] as f32 * w00 + p01[3] as f32 * w01 + 
                                 p10[3] as f32 * w10 + p11[3] as f32 * w11);
                        
                        // Weight decreases with distance from original point
                        let weight = 1.0 - (i as f32 / num_samples as f32) * 0.5;
                        
                        r_sum += r * weight;
                        g_sum += g * weight;
                        b_sum += b * weight;
                        a_sum += a * weight;
                        weight_sum += weight;
                    }
                }
                
                // Normalize and set the result
                if weight_sum > 0.0 {
                    let r = (r_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let g = (g_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let b = (b_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    let a = (a_sum / weight_sum).round().clamp(0.0, 255.0) as u8;
                    
                    result.put_pixel(x, y, Rgba([r, g, b, a]));
                } else {
                    // If no samples were taken, copy the original pixel
                    trace!("No samples for pixel ({}, {}), using original", x, y);
                    result.put_pixel(x, y, *image.get_pixel(x, y));
                }
            }
        }
        
        let duration = start_time.elapsed();
        debug!("Radial blur completed in {:.2?}", duration);
        result
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        trace!("Cloning Radial blur filter");
        Box::new(self.clone())
    }
} 