// Rust Photo - Main Application
use rust_photo::{core::init_logging, ui::MainWindow, APP_ID};
use gtk4::prelude::*;
use std::env;
use log::{info, warn, debug, error};
use std::path::PathBuf;
use gtk4::{Application, ApplicationWindow};
use libadwaita::prelude::*;
use crate::core::{Document, Layer};
use crate::filters::{Filter, GaussianBlur};

mod core;
mod filters;
mod tools;
mod ui;
mod vector;

#[cfg(feature = "gpu-cuda")]
fn init_gpu() {
    info!("Initializing CUDA GPU support");
    // CUDA initialization is handled by the runtime automatically
    // We don't need to explicitly initialize it
}

#[cfg(not(feature = "gpu-cuda"))]
fn init_gpu() {
    info!("No GPU acceleration enabled");
}

fn main() {
    // Initialize logging
    env_logger::init();
    info!("Starting Rust Photo v0.1");
    
    // Initialize GPU if available
    init_gpu();
    
    // Create GTK application
    let app = Application::builder()
        .application_id("com.example.rust_photo")
        .build();
    
    // Initialize styles when the application is activated
    app.connect_startup(|_| {
        ui::init_styles();
    });
    
    app.connect_activate(|app| {
        let main_window = MainWindow::new(app);
        main_window.window.show();
    });
    
    // Run the application
    app.run();
    
    info!("Shutting down Rust Photo");
}

/// Initialize a new document for testing
fn create_test_document(app_state: &mut core::AppState) -> core::Document {
    info!("Creating test document");
    
    let width = 1920;
    let height = 1080;
    
    info!("Creating new document: {}x{}", width, height);
    let mut document = app_state.new_document(
        width, 
        height, 
        core::ColorSpace::SRGB, 
        core::BitDepth::Bit8
    );
    
    // Create a few layers for testing
    info!("Adding test layers");
    
    // Background layer was already created by new_document
    
    // Add a second layer
    let mut layer2 = core::Layer::new(width, height, "Layer 2".to_string());
    layer2.opacity = 0.8;
    document.add_layer(layer2);
    
    // Add a third layer
    let mut layer3 = core::Layer::new(width, height, "Text Layer".to_string());
    layer3.opacity = 1.0;
    document.add_layer(layer3);
    
    // Apply a test filter to one of the layers
    info!("Applying test filter");
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