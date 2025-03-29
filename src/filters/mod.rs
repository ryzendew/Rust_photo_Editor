// Filters module: Contains image processing filters

use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::contrast::threshold;
use imageproc::filter::gaussian_blur_f32;
use imageproc::gradients::sobel_gradients;
use imageproc::map::map_colors;
use imageproc::pixelops::weighted_sum;
use log::{debug, error, info, trace, warn};

pub mod blur;
pub mod sharpen;
pub mod color;
pub mod artistic;
pub mod distort;

pub use blur::*;
pub use sharpen::*;
pub use color::*;
pub use artistic::*;
pub use distort::*;

use std::sync::{Arc, Mutex};
use std::thread;
use rayon::prelude::*;

/// Represents an image filter that can be applied to an image
pub trait Filter {
    /// Apply the filter to the given image
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    
    /// Get the name of the filter
    fn name(&self) -> &str;
    
    /// Get a description of the filter
    fn description(&self) -> &str;
    
    /// Clone the filter into a boxed trait object
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync>;
}

/// Trait for filters that can be applied with a specified intensity
pub trait IntensityFilter: Filter {
    /// Set the intensity of the filter (usually 0.0 to 1.0)
    fn set_intensity(&mut self, intensity: f32);
    
    /// Get the current intensity of the filter
    fn intensity(&self) -> f32;
}

/// Applies a filter to an image
pub fn apply_filter(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter: &dyn Filter,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    info!("Applying filter '{}' to image: {}x{}", 
          filter.name(), image.width(), image.height());
    debug!("Filter description: {}", filter.description());
    
    let result = filter.apply(image);
    debug!("Filter '{}' applied successfully", filter.name());
    result
}

/// Applies a filter to a specific region of an image
pub fn apply_filter_to_region(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter: &dyn Filter,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    info!("Applying filter '{}' to region: x={}, y={}, width={}, height={}", 
          filter.name(), x, y, width, height);
    
    let mut result = image.clone();
    
    // Extract the region
    let region = extract_region(image, x, y, width, height);
    debug!("Extracted region of size {}x{}", region.width(), region.height());
    
    // Apply the filter to the region
    let filtered_region = filter.apply(&region);
    debug!("Filter applied to region");
    
    // Put the filtered region back into the result
    debug!("Merging filtered region back into result image");
    for j in 0..height {
        for i in 0..width {
            if i + x < image.width() && j + y < image.height() {
                let pixel = filtered_region.get_pixel(i, j);
                result.put_pixel(i + x, j + y, *pixel);
            }
        }
    }
    
    debug!("Region filter applied successfully");
    result
}

/// Extract a region from an image
fn extract_region(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    trace!("Extracting region: x={}, y={}, width={}, height={}", x, y, width, height);
    
    let mut region = ImageBuffer::new(width, height);
    
    for j in 0..height {
        for i in 0..width {
            if i + x < image.width() && j + y < image.height() {
                let pixel = image.get_pixel(i + x, j + y);
                region.put_pixel(i, j, *pixel);
            } else {
                // Out of bounds
                region.put_pixel(i, j, Rgba([0, 0, 0, 0]));
                trace!("Pixel at {},{} is out of bounds", i + x, j + y);
            }
        }
    }
    
    trace!("Region extraction complete: {}x{}", region.width(), region.height());
    region
}

/// Helper function to apply a filter in parallel using multiple threads
pub fn apply_filter_parallel<F>(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter_factory: F,
    num_threads: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>>
where
    F: Fn() -> Box<dyn Filter + Send + Sync>,
{
    info!("Applying filter in parallel using {} threads", num_threads);
    
    if num_threads <= 1 {
        // Single-threaded case
        debug!("Using single-threaded execution due to thread count <= 1");
        return filter_factory().apply(image);
    }
    
    let width = image.width();
    let height = image.height();
    debug!("Processing image of size {}x{}", width, height);
    
    let mut result = ImageBuffer::new(width, height);
    
    // Split the image into horizontal strips
    let strip_height = height / num_threads as u32;
    let remainder = height % num_threads as u32;
    debug!("Strip height: {}, remainder: {}", strip_height, remainder);
    
    let result_arc = Arc::new(Mutex::new(result));
    
    // Create and launch threads
    let mut handles = vec![];
    
    for i in 0..num_threads {
        let start_y = i as u32 * strip_height;
        let mut end_y = start_y + strip_height;
        
        // Add remainder to the last strip
        if i == num_threads - 1 {
            end_y += remainder;
        }
        
        debug!("Thread {} processing strip y={}..{}", i, start_y, end_y);
        
        let image_clone = image.clone();
        let result_clone = Arc::clone(&result_arc);
        let filter = filter_factory();
        
        let handle = thread::spawn(move || {
            // Extract the strip
            let strip = extract_region(&image_clone, 0, start_y, width, end_y - start_y);
            
            // Apply the filter to the strip
            debug!("Thread {} applying filter to strip", i);
            let filtered_strip = filter.apply(&strip);
            
            // Put the filtered strip back into the result
            debug!("Thread {} merging results", i);
            let mut result = result_clone.lock().unwrap();
            for y in 0..(end_y - start_y) {
                for x in 0..width {
                    let pixel = filtered_strip.get_pixel(x, y);
                    result.put_pixel(x, start_y + y, *pixel);
                }
            }
            debug!("Thread {} completed processing", i);
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    debug!("Waiting for all threads to complete");
    for (i, handle) in handles.into_iter().enumerate() {
        if let Err(e) = handle.join() {
            error!("Thread {} panicked: {:?}", i, e);
        }
    }
    
    info!("Parallel filter application completed");
    Arc::try_unwrap(result_arc)
        .unwrap()
        .into_inner()
        .unwrap()
}

/// Apply a filter using Rayon's parallel processing
pub fn apply_filter_rayon<F>(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter: F,
) -> ImageBuffer<Rgba<u8>, Vec<u8>>
where
    F: Fn(&Rgba<u8>) -> Rgba<u8> + Send + Sync,
{
    info!("Applying filter using Rayon parallel iterator");
    debug!("Image size: {}x{}", image.width(), image.height());
    
    let width = image.width();
    let height = image.height();
    let mut result = ImageBuffer::new(width, height);
    
    // Process the image in parallel by chunks
    debug!("Starting parallel processing");
    result.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
        let src_pixel = image.get_pixel(x, y);
        *pixel = filter(src_pixel);
    });
    
    info!("Rayon parallel filter completed");
    result
}

/// Create a grayscale preview of a filter application
pub fn preview_filter(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter: &dyn Filter,
    scale: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    info!("Creating filter preview at 1/{} scale", scale);
    debug!("Original image: {}x{}, filter: {}", 
           image.width(), image.height(), filter.name());
    
    let width = image.width() / scale;
    let height = image.height() / scale;
    debug!("Preview size: {}x{}", width, height);
    
    // Create a downscaled version of the image
    let mut small = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let src_x = x * scale;
            let src_y = y * scale;
            let pixel = image.get_pixel(src_x, src_y);
            small.put_pixel(x, y, *pixel);
        }
    }
    
    // Apply the filter
    debug!("Applying filter to downscaled image");
    let filtered = filter.apply(&small);
    
    // Convert to grayscale
    debug!("Converting to grayscale");
    let mut grayscale = ImageBuffer::new(width, height);
    for (x, y, pixel) in filtered.enumerate_pixels() {
        let luma = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
        grayscale.put_pixel(x, y, Rgba([luma, luma, luma, pixel[3]]));
    }
    
    info!("Preview created successfully");
    grayscale
}

#[derive(Debug, Clone)]
pub struct FilterSettings {
    pub filter_type: FilterType,
    pub strength: f32,
    pub radius: f32,
    pub threshold: f32,
    pub parameters: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    Blur,
    Sharpen,
    EdgeDetect,
    Emboss,
    Noise,
    Threshold,
    Brightness,
    Contrast,
    Saturation,
    HueSaturation,
    ColorBalance,
    Levels,
    Curves,
    Invert,
    Custom,
}

pub struct ImageFilter {
    settings: FilterSettings,
}

impl ImageFilter {
    pub fn new(settings: FilterSettings) -> Self {
        Self { settings }
    }
    
    pub fn apply(&self, image: &DynamicImage) -> DynamicImage {
        // The actual filter implementation would go here
        // For now, we'll just return a clone of the input image
        match self.settings.filter_type {
            FilterType::Blur => {
                // Apply Gaussian blur
                image.blur(self.settings.radius)
            },
            FilterType::Sharpen => {
                // Apply sharpening filter
                image.clone()
            },
            FilterType::EdgeDetect => {
                // Apply edge detection
                image.clone()
            },
            // ... other filter implementations
            _ => image.clone(),
        }
    }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Blur,
            strength: 1.0,
            radius: 5.0,
            threshold: 0.5,
            parameters: Vec::new(),
        }
    }
}

impl ImageFilter {
    pub fn apply_brightness(&self, image: &DynamicImage) -> DynamicImage {
        let brightness = self.settings.strength;
        let factor = (brightness + 100.0) / 100.0;
        let img = image.to_rgba8();
        let (width, height) = img.dimensions();
        let mut output = img.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let r = (pixel[0] as f32 * factor).clamp(0.0, 255.0) as u8;
                let g = (pixel[1] as f32 * factor).clamp(0.0, 255.0) as u8;
                let b = (pixel[2] as f32 * factor).clamp(0.0, 255.0) as u8;
                output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
            }
        }
        
        DynamicImage::ImageRgba8(output)
    }

    pub fn apply_contrast(&self, image: &DynamicImage) -> DynamicImage {
        let contrast = self.settings.strength;
        let factor = (contrast + 100.0) / 100.0;
        let img = image.to_rgba8();
        let (width, height) = img.dimensions();
        let mut output = img.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let r = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
                let g = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
                let b = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
                output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
            }
        }
        
        DynamicImage::ImageRgba8(output)
    }

    pub fn apply_saturation(&self, image: &DynamicImage) -> DynamicImage {
        let saturation = self.settings.strength;
        let factor = (saturation + 100.0) / 100.0;
        let img = image.to_rgba8();
        let (width, height) = img.dimensions();
        let mut output = img.clone();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let gray = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
                let r = (gray + (pixel[0] as f32 - gray) * factor).clamp(0.0, 255.0) as u8;
                let g = (gray + (pixel[1] as f32 - gray) * factor).clamp(0.0, 255.0) as u8;
                let b = (gray + (pixel[2] as f32 - gray) * factor).clamp(0.0, 255.0) as u8;
                output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
            }
        }
        
        DynamicImage::ImageRgba8(output)
    }

    pub fn apply_hue(&self, image: &DynamicImage) -> DynamicImage {
        let hue = self.settings.strength;
        let angle = hue * std::f32::consts::PI / 180.0;
        let img = image.to_rgba8();
        let (width, height) = img.dimensions();
        let mut output = img.clone();
        
        let cos_h = angle.cos();
        let sin_h = angle.sin();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
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
        
        DynamicImage::ImageRgba8(output)
    }

    pub fn apply_blur(&self, image: &DynamicImage) -> DynamicImage {
        let radius = self.settings.radius;
        let sigma = radius / 3.0;

        let img = image.to_rgba8();
        let blurred = gaussian_blur_f32(&img, sigma);
        DynamicImage::ImageRgba8(blurred)
    }

    pub fn apply_sharpen(&self, img: &DynamicImage) -> DynamicImage {
        let kernel = [
            -1.0, -1.0, -1.0,
            -1.0,  9.0, -1.0,
            -1.0, -1.0, -1.0
        ];
        image::DynamicImage::ImageRgb8(imageproc::filter::filter3x3(&img.to_rgb8(), &kernel))
    }

    pub fn apply_noise_reduction(&self, image: &DynamicImage) -> DynamicImage {
        let sigma = self.settings.radius;
        if sigma <= 0.0 {
            return image.clone();
        }
        
        // Convert to grayscale and apply gaussian blur
        let img = image.to_rgba8();
        let denoised = gaussian_blur_f32(&img, sigma);
        DynamicImage::ImageRgba8(denoised)
    }
}

fn apply_brightness(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, amount: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut output = image.clone();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let r = (pixel[0] as f32 + amount * 255.0).clamp(0.0, 255.0) as u8;
            let g = (pixel[1] as f32 + amount * 255.0).clamp(0.0, 255.0) as u8;
            let b = (pixel[2] as f32 + amount * 255.0).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
        }
    }
    
    output
}

fn apply_contrast(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, amount: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut output = image.clone();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let r = ((pixel[0] as f32 - 128.0) * amount + 128.0).clamp(0.0, 255.0) as u8;
            let g = ((pixel[1] as f32 - 128.0) * amount + 128.0).clamp(0.0, 255.0) as u8;
            let b = ((pixel[2] as f32 - 128.0) * amount + 128.0).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
        }
    }
    
    output
}

fn apply_saturation(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, amount: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut output = image.clone();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let gray = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
            let r = (gray + (pixel[0] as f32 - gray) * amount).clamp(0.0, 255.0) as u8;
            let g = (gray + (pixel[1] as f32 - gray) * amount).clamp(0.0, 255.0) as u8;
            let b = (gray + (pixel[2] as f32 - gray) * amount).clamp(0.0, 255.0) as u8;
            output.put_pixel(x, y, Rgba([r, g, b, pixel[3]]));
        }
    }
    
    output
}

fn apply_hue(image: &ImageBuffer<Rgba<u8>, Vec<u8>>, hue: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut output = image.clone();
    
    let angle = hue * std::f32::consts::PI / 180.0;
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

fn apply_invert(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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

pub struct BrightnessFilter {
    pub amount: f32,
    name: String,
    description: String,
}

impl BrightnessFilter {
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            name: "Brightness".to_string(),
            description: "Adjusts the brightness of the image".to_string(),
        }
    }
}

impl Filter for BrightnessFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        apply_brightness(image, self.amount)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(BrightnessFilter {
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
            name: "Contrast".to_string(),
            description: "Adjusts the contrast of the image".to_string(),
        }
    }
}

impl Filter for ContrastFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        apply_contrast(image, self.amount)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(ContrastFilter {
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
            name: "Saturation".to_string(),
            description: "Adjusts the saturation of the image".to_string(),
        }
    }
}

impl Filter for SaturationFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        apply_saturation(image, self.amount)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(SaturationFilter {
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
            name: "Hue".to_string(),
            description: "Adjusts the hue of the image".to_string(),
        }
    }
}

impl Filter for HueFilter {
    fn apply(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        apply_hue(image, self.amount)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(HueFilter {
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
        apply_invert(image)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn box_clone(&self) -> Box<dyn Filter + Send + Sync> {
        Box::new(InvertFilter {
            name: self.name.clone(),
            description: self.description.clone(),
        })
    }
} 