// Rust Photo - Main Application
use rust_photo::{core::init_logging, ui::MainWindow, APP_ID};
use gtk4::prelude::*;
use std::env;
use log::{info, warn, debug, error};

mod core;
mod filters;
mod tools;
mod ui;
mod vector;

fn init_gpu() {
    // Check for CUDA
    if cfg!(feature = "gpu-cuda") {
        match cuda_runtime_sys::init() {
            Ok(_) => info!("CUDA initialization successful"),
            Err(e) => warn!("Failed to initialize CUDA: {}", e),
        }
    }

    // Check for ROCm
    if cfg!(feature = "gpu-rocm") {
        match rocm_runtime_sys::init() {
            Ok(_) => info!("ROCm initialization successful"),
            Err(e) => warn!("Failed to initialize ROCm: {}", e),
        }
    }

    // Initialize OpenCL as fallback
    match ocl::Platform::list() {
        Ok(platforms) => {
            if !platforms.is_empty() {
                info!("OpenCL platforms available: {}", platforms.len());
            } else {
                warn!("No OpenCL platforms found");
            }
        }
        Err(e) => warn!("Failed to query OpenCL platforms: {}", e),
    }
}

fn main() {
    // Initialize logging first thing
    init_logging();
    
    info!("Starting Rust Photo v0.1");
    
    // Initialize GPU support
    if let Err(e) = rust_photo::init_gpu() {
        error!("Failed to initialize GPU support: {}", e);
    }
    
    // Create GTK application
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .build();

    // Connect to activate signal
    app.connect_activate(|app| {
        // Create and show the main window
        let window = MainWindow::new(app);
        window.window.show();
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
    
    info!("Shutting down Rust Photo");
}

/// Initialize a new document for testing
fn create_test_document(app_state: &mut core::AppState) -> core::Document {
    debug!("Creating test document");
    
    let width = 1920;
    let height = 1080;
    
    info!("Creating new document: {}x{}", width, height);
    let document = app_state.new_document(
        width, 
        height, 
        core::ColorSpace::SRGB, 
        core::BitDepth::Bit8
    );
    
    // Create a few layers for testing
    debug!("Adding test layers");
    
    // Background layer was already created by new_document
    
    // Add a second layer
    let mut layer2 = core::Layer::new(width, height, "Layer 2".to_string());
    layer2.opacity = 0.8;
    let layer2_idx = document.add_layer(layer2);
    debug!("Added Layer 2 at index {}", layer2_idx);
    
    // Add a third layer
    let mut layer3 = core::Layer::new(width, height, "Text Layer".to_string());
    layer3.opacity = 1.0;
    let layer3_idx = document.add_layer(layer3);
    debug!("Added Layer 3 at index {}", layer3_idx);
    
    // Apply a test filter to one of the layers
    debug!("Applying test filter");
    let gaussian_blur = filters::GaussianBlur::new(5.0);
    if let Some(layer) = document.layer_manager.get_layer_mut(1) {
        let width = layer.image.width();
        let height = layer.image.height();
        layer.image = gaussian_blur.apply(&layer.image);
        info!("Applied Gaussian blur to layer at index 1 ({}x{})", width, height);
    }
    
    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::{GaussianBlur, BoxBlur, Filter};
    use crate::core::{Layer, Canvas, Point};
    use image::{ImageBuffer, Rgba};
    
    #[test]
    fn test_gaussian_blur() {
        // Create a test image
        let width = 100;
        let height = 100;
        let mut image = ImageBuffer::new(width, height);
        
        // Fill with test pattern
        for y in 0..height {
            for x in 0..width {
                let r = (x as f32 / width as f32 * 255.0) as u8;
                let g = (y as f32 / height as f32 * 255.0) as u8;
                let b = 128u8;
                image.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
        
        // Apply filter
        let filter = GaussianBlur::new(2.0);
        let result = filter.apply(&image);
        
        // Basic assertions
        assert_eq!(result.width(), width);
        assert_eq!(result.height(), height);
        
        // Center pixel should be smoothed
        let center_pixel = result.get_pixel(width/2, height/2);
        println!("Center pixel: {:?}", center_pixel);
    }
    
    #[test]
    fn test_canvas_operations() {
        // Create a canvas
        let width = 800;
        let height = 600;
        let mut canvas = Canvas::new(width, height);
        
        // Test zoom and pan
        canvas.set_zoom(1.5);
        assert_eq!(canvas.zoom, 1.5);
        
        canvas.pan(10.0, 20.0);
        assert_eq!(canvas.offset_x, 10.0);
        assert_eq!(canvas.offset_y, 20.0);
        
        // Test coordinate conversion
        let screen_point = Point::new(100.0, 100.0);
        let canvas_point = canvas.screen_to_canvas(screen_point.x, screen_point.y);
        let screen_point2 = canvas.canvas_to_screen(canvas_point.x, canvas_point.y);
        
        assert!((screen_point.x - screen_point2.x).abs() < 0.001);
        assert!((screen_point.y - screen_point2.y).abs() < 0.001);
    }
} 